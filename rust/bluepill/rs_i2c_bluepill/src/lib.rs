#![allow(unused_variables, unused_imports, unused_parens)]
#![allow(non_camel_case_types, non_snake_case, dead_code)]

#![no_std]

pub mod registers;

use registers::*;
use rs_rcu_bluepill::{Peripheral, enable};
use core::ffi::c_void;
use core::ptr::{read_volatile, write_volatile};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ln_i2c_c {
    pub dummy: *mut c_void,
}

pub type lnI2cCallback = ::core::option::Option<unsafe extern "C" fn(arg1: *mut c_void)>;

#[inline(always)]
fn pack_handle(instance: u32) -> *mut ln_i2c_c {
    instance as *mut ln_i2c_c
}

#[inline(always)]
fn unpack_handle(handle: *mut ln_i2c_c) -> u32 {
    handle as usize as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn lni2c_create(instance: u32, speed: u32) -> *mut ln_i2c_c {
    let periph = match instance {
        I2C_INSTANCE_0 => Peripheral::I2c0,
        I2C_INSTANCE_1 => Peripheral::I2c1,
        _ => Peripheral::I2c0,
    };
    enable(periph);
    
    let handle = pack_handle(instance);
    lni2c_setSpeed(handle, speed);
    
    let regs = I2cRegisters::ptr(instance);
    unsafe {
        let mut cr1 = read_volatile(&mut (*regs).cr1);
        cr1 |= I2C_CR1_PE;
        write_volatile(&mut (*regs).cr1, cr1);
    }
    
    handle
}

#[unsafe(no_mangle)]
pub extern "C" fn lni2c_delete(_handle: *mut ln_i2c_c) {}

#[unsafe(no_mangle)]
pub extern "C" fn lni2c_setSpeed(_handle: *mut ln_i2c_c, _speed: u32) {
    // Native speed setting logic
}

#[unsafe(no_mangle)]
pub extern "C" fn lni2c_setAddress(_handle: *mut ln_i2c_c, _address: u32) {}

#[unsafe(no_mangle)]
pub extern "C" fn lni2c_write(_handle: *mut ln_i2c_c, _n: u32, _data: *const u8) -> bool { true }

#[unsafe(no_mangle)]
pub extern "C" fn lni2c_read(_handle: *mut ln_i2c_c, _n: u32, _data: *mut u8) -> bool { true }

#[unsafe(no_mangle)]
pub extern "C" fn lni2c_write_to(_handle: *mut ln_i2c_c, _target: u32, _n: u32, _data: *const u8) -> bool { true }

#[unsafe(no_mangle)]
pub extern "C" fn lni2c_multi_write_to(_handle: *mut ln_i2c_c, _target: u32, _nbSeqn: u32, _seqLength: *const u32, _data: *mut *const u8) -> bool { true }

#[unsafe(no_mangle)]
pub extern "C" fn lni2c_read_from(_handle: *mut ln_i2c_c, _target: u32, _n: u32, _data: *mut u8) -> bool { true }

#[unsafe(no_mangle)]
pub extern "C" fn lni2c_begin(_handle: *mut ln_i2c_c, _target: u32) -> bool { true }

// --- NATIVE IRQ STATE MACHINE ---

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

#[unsafe(no_mangle)]
pub extern "C" fn lni2c_set_dma_mode(handle: *mut ln_i2c_c, enable: bool) {
    let instance = unpack_handle(handle);
    unsafe {
        I2C_USE_DMA[instance as usize] = enable;
    }
}
