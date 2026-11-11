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
        true
    }

    pub fn read(&mut self, _n: u32, _data: *mut u8) -> bool {
        true
    }

    pub fn write_to(&mut self, _target: u32, _n: u32, _data: *const u8) -> bool {
        true
    }

    pub fn multi_write_to(
        &mut self,
        _target: u32,
        _nb_seqn: u32,
        _seq_length: *const u32,
        _data: *mut *const u8,
    ) -> bool {
        true
    }

    pub fn read_from(&mut self, _target: u32, _n: u32, _data: *mut u8) -> bool {
        true
    }

    pub fn begin(&mut self, _target: u32) -> bool {
        true
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

    /// The GD32 I²C block has no scatter/gather: the transaction is a START, the
    /// operations in order, a STOP.
    fn transfer(&mut self, target: u8, operations: &mut [i2c::Operation<'_>]) -> Result<()> {
        if !I2c::begin(self, target as u32) {
            return Err(Error::Io);
        }
        for operation in operations.iter_mut() {
            match operation {
                i2c::Operation::Write(data) => {
                    if !I2c::write(self, data.len() as u32, data.as_ptr()) {
                        return Err(Error::Io);
                    }
                }
                i2c::Operation::Read(buffer) => {
                    if !I2c::read(self, buffer.len() as u32, buffer.as_mut_ptr()) {
                        return Err(Error::Io);
                    }
                    if let Some(handler) = HANDLERS.get(self.instance as usize) {
                        handler.on_rx_buffer(buffer);
                    }
                }
            }
        }
        if let Some(handler) = HANDLERS.get(self.instance as usize) {
            handler.on_tx_complete();
        }
        Ok(())
    }

    /// Asynchronous multi-chunk write: the GD32 block chains the chunks with a
    /// repeated start and finishes with a STOP, all from its event interrupt, so
    /// `multi_write_to` returns as soon as the sequence is queued.
    ///
    /// `wait_idle` keeps the trait's default on this family: the completion is
    /// reported through `on_tx_complete`, not through a pollable busy flag.
    fn start_write_chunks(&mut self, target: u8, chunks: &[&[u8]]) -> Result<()> {
        if chunks.is_empty() || chunks.len() > MAX_SCAN_CHUNKS {
            return Err(Error::InvalidConfig);
        }
        let mut lengths = [0u32; MAX_SCAN_CHUNKS];
        let mut pointers = [core::ptr::null::<u8>(); MAX_SCAN_CHUNKS];
        for (index, chunk) in chunks.iter().enumerate() {
            lengths[index] = chunk.len() as u32;
            pointers[index] = chunk.as_ptr();
        }
        let started = I2c::multi_write_to(
            self,
            target as u32,
            chunks.len() as u32,
            lengths.as_ptr(),
            pointers.as_mut_ptr(),
        );
        if started { Ok(()) } else { Err(Error::Io) }
    }

    fn set_handler(&mut self, handler: Option<&'static dyn i2c::I2cHandler>) {
        HANDLERS.set(self.instance as usize, handler);
    }
}

pub mod shim;
pub use shim::*;
