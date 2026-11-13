#![allow(unused_variables, unused_imports, unused_parens)]
#![allow(non_camel_case_types, non_snake_case, dead_code)]

#![no_std]

pub mod registers;

use registers::*;
use rs_rcu_bluepill::{Peripheral, enable};
use rs_mcu_traits::handler::HandlerTable;
use rs_mcu_traits::i2c;
use rs_mcu_traits::{Error, Result};
use core::ptr::{read_volatile, write_volatile};

/// Idiomatic struct representing an I2C master/slave hardware peripheral.
pub struct I2c {
    pub(crate) instance: u32,
}

impl I2c {
    pub fn new(instance: u32, speed: u32) -> Self {
        let periph = match instance {
            I2C_INSTANCE_0 => Peripheral::I2c0,
            I2C_INSTANCE_1 => Peripheral::I2c1,
            _ => Peripheral::I2c0,
        };
        enable(periph);

        let mut i2c = Self { instance };
        i2c.set_speed(speed);

        let regs = I2cRegisters::ptr(instance);
        unsafe {
            let mut cr1 = read_volatile(&mut (*regs).cr1);
            cr1 |= I2C_CR1_PE;
            write_volatile(&mut (*regs).cr1, cr1);
        }

        i2c
    }

    #[inline(always)]
    pub fn instance(&self) -> u32 {
        self.instance
    }

    pub fn set_speed(&mut self, _speed: u32) {
        // Native speed setting logic
    }

    pub fn set_address(&mut self, _address: u32) {}

    pub fn write(&mut self, _n: u32, _data: *const u8) -> bool {
        false
    }

    pub fn read(&mut self, _n: u32, _data: *mut u8) -> bool {
        false
    }

    pub fn write_to(&mut self, _target: u32, _n: u32, _data: *const u8) -> bool {
        false
    }

    pub fn multi_write_to(
        &mut self,
        _target: u32,
        _nb_seqn: u32,
        _seq_length: *const u32,
        _data: *mut *const u8,
    ) -> bool {
        false
    }

    pub fn read_from(&mut self, _target: u32, _n: u32, _data: *mut u8) -> bool {
        false
    }

    pub fn begin(&mut self, _target: u32) -> bool {
        false
    }

    pub fn set_dma_mode(&mut self, enable: bool) {
        unsafe {
            I2C_USE_DMA[self.instance as usize] = enable;
        }
    }
}

// --- IRQ STATE MACHINE ---

#[derive(Clone, Copy)]
struct I2cSession {
    use_dma: bool,
    target: u32,
    nb_seqn: u32,
    seq_length: *const u32,
    data: *mut *const u8,
    cur_seq: u32,
    cur_pos: u32,
    error: bool,
    done: bool,
}

static mut I2C_SESSIONS: [Option<I2cSession>; I2C_MAX_INSTANCES] = [None; I2C_MAX_INSTANCES];
static mut I2C_USE_DMA: [bool; I2C_MAX_INSTANCES] = [false; I2C_MAX_INSTANCES];

#[unsafe(no_mangle)]
pub extern "C" fn i2cIrqHandler(instance: i32, error: bool) {
    if instance < I2C_INSTANCE_0 as i32 || instance > I2C_INSTANCE_1 as i32 {
        return;
    }

    unsafe {
        let session = match &mut I2C_SESSIONS[instance as usize] {
            Some(s) => s,
            None => return, // Spurious interrupt
        };

        let regs = I2cRegisters::ptr(instance as u32);

        if error {
            // Handle AF, BERR, ARLO, OVR
            session.error = true;
            session.done = true;

            // Clear error flags
            let mut sr1 = read_volatile(&mut (*regs).sr1);
            sr1 &= !I2C_SR1_ERROR_MASK;
            write_volatile(&mut (*regs).sr1, sr1);

            // Send STOP condition
            let mut cr1 = read_volatile(&mut (*regs).cr1);
            cr1 |= I2C_CR1_STOP;
            write_volatile(&mut (*regs).cr1, cr1);
            return;
        }

        // --- Event State Machine ---
        let sr1 = read_volatile(&mut (*regs).sr1);

        // EV5: SB (Start Bit) sent
        if (sr1 & I2C_SR1_SB) != 0 {
            // Send 7-bit Address
            let addr = (session.target & I2C_ADDRESS_7BIT_MASK) << I2C_ADDRESS_7BIT_SHIFT;
            write_volatile(&mut (*regs).dr, addr);
        }
        // EV6: ADDR (Address sent)
        else if (sr1 & I2C_SR1_ADDR) != 0 {
            // Clear ADDR by reading SR1 (done) and SR2
            let _sr2 = read_volatile(&mut (*regs).sr2);

            // Enable event interrupt
            let mut cr2 = read_volatile(&mut (*regs).cr2);
            cr2 |= I2C_CR2_ITEVTEN;
            write_volatile(&mut (*regs).cr2, cr2);
        }
        // EV8: TXE (Data register empty)
        else if (sr1 & I2C_SR1_TXE) != 0 {
            if session.cur_seq < session.nb_seqn {
                let seq_len = *session.seq_length.add(session.cur_seq as usize);
                let seq_data = *session.data.add(session.cur_seq as usize);

                if session.cur_pos < seq_len {
                    // Send next byte
                    let byte = *seq_data.add(session.cur_pos as usize);
                    write_volatile(&mut (*regs).dr, byte as u32);
                    session.cur_pos += 1;
                } else {
                    // Sequence done, move to next
                    session.cur_seq += 1;
                    session.cur_pos = 0;

                    if session.cur_seq == session.nb_seqn {
                        // All sequences done, send STOP
                        let mut cr1 = read_volatile(&mut (*regs).cr1);
                        cr1 |= I2C_CR1_STOP;
                        write_volatile(&mut (*regs).cr1, cr1);

                        // Disable interrupts
                        let mut cr2 = read_volatile(&mut (*regs).cr2);
                        cr2 &= !(I2C_CR2_ITEVTEN | I2C_CR2_ITERREN);
                        write_volatile(&mut (*regs).cr2, cr2);

                        session.done = true;
                    }
                }
            }
        }
    }
}

const I2C_MAX_INSTANCES: usize = 2;


/// One handler slot per I²C instance.
static HANDLERS: HandlerTable<dyn i2c::I2cHandler, I2C_MAX_INSTANCES> = HandlerTable::new();

/// Chunks one asynchronous transaction can carry (the sequence registers hold a
/// handful, and the driver's own API takes up to three).
const MAX_SCAN_CHUNKS: usize = 3;
/// Bounded poll budget for [`i2c::I2c::wait_idle`] on this block.
const BUSY_POLL_LIMIT: u32 = 1_000_000;

impl i2c::I2c for I2c {
    fn configure(&mut self, config: &i2c::I2cConfig) -> Result<()> {
        I2c::set_speed(self, config.speed_hz);
        I2c::set_address(self, config.own_address as u32);
        Ok(())
    }

    fn set_speed(&mut self, speed_hz: u32) -> Result<()> {
        I2c::set_speed(self, speed_hz);
        Ok(())
    }

    /// The GD32 I²C register transactions are not yet implemented on this family.
    fn transfer(&mut self, _target: u8, _operations: &mut [i2c::Operation<'_>]) -> Result<()> {
        Err(Error::Unsupported)
    }

    /// Asynchronous multi-chunk write is not yet implemented on this family.
    fn start_write_chunks(&mut self, _target: u8, _chunks: &[&[u8]]) -> Result<()> {
        Err(Error::Unsupported)
    }

    fn set_handler(&mut self, handler: Option<&'static dyn i2c::I2cHandler>) {
        HANDLERS.set(self.instance as usize, handler);
    }
}

pub mod shim;
pub use shim::*;
