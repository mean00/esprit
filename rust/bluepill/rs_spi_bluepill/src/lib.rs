#![allow(non_upper_case_globals)]
#![allow(unused_variables, unused_imports, unused_parens)]
#![allow(non_camel_case_types, non_snake_case, dead_code)]

#![no_std]

pub mod registers;

use registers::*;
use rs_rcu_bluepill::{Peripheral, enable};
use rs_gpio_bluepill::Pin;
use core::ffi::c_void;
use core::ptr::{read_volatile, write_volatile};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ln_spi_c {
    pub dummy: *mut c_void,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct lnSPISettings {
    pub speed: u32,
    pub data_mode: u32,
    pub bit_order: u32,
}

pub type lnSpiCallback = ::core::option::Option<unsafe extern "C" fn(arg1: *mut c_void)>;

static mut SPI_USE_DMA: [bool; SPI_MAX_INSTANCES] = [false; SPI_MAX_INSTANCES];

#[inline(always)]
fn pack_handle(instance: u32) -> *mut ln_spi_c {
    instance as *mut ln_spi_c
}

#[inline(always)]
fn unpack_handle(handle: *mut ln_spi_c) -> u32 {
    handle as usize as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_create(instance: u32, pinCs: i32) -> *mut ln_spi_c {
    let periph = match instance {
        SPI_INSTANCE_0 => Peripheral::Spi0,
        SPI_INSTANCE_1 => Peripheral::Spi1,
        SPI_INSTANCE_2 => Peripheral::Spi2,
        _ => Peripheral::Spi0,
    };
    enable(periph);
    pack_handle(instance)
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_delete(_handle: *mut ln_spi_c) {}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_begin(handle: *mut ln_spi_c, dataSize: u32) {
    let idx = unpack_handle(handle);
    let regs = SpiRegisters::ptr(idx);
    unsafe {
        let mut cr1 = read_volatile(&mut (*regs).cr1);
        cr1 |= SPI_CR1_MSTR; // Set Master Mode (MSTR)
        cr1 |= SPI_CR1_SSI | SPI_CR1_SSM; // Set SSM and SSI (Software Slave Management)
        if dataSize == SPI_DATA_SIZE_16 {
            cr1 |= SPI_CR1_DFF; // Set DFF to 16-bit
        } else {
            cr1 &= !SPI_CR1_DFF; // Set DFF to 8-bit
        }
        cr1 |= SPI_CR1_SPE; // Enable SPI (SPE)
        write_volatile(&mut (*regs).cr1, cr1);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_end(handle: *mut ln_spi_c) {
    let idx = unpack_handle(handle);
    let regs = SpiRegisters::ptr(idx);
    unsafe {
        let mut cr1 = read_volatile(&mut (*regs).cr1);
        cr1 &= !SPI_CR1_SPE; // Disable SPI (SPE)
        write_volatile(&mut (*regs).cr1, cr1);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_set_speed(handle: *mut ln_spi_c, speed: u32) {
    // Native implementation mapping requested speed to BR bits in CR1
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_set_data_mode(handle: *mut ln_spi_c, mode: u32) {
    let idx = unpack_handle(handle);
    let regs = SpiRegisters::ptr(idx);
    unsafe {
        let mut cr1 = read_volatile(&mut (*regs).cr1);
        cr1 &= !SPI_CR1_MODE_MASK; // Clear CPOL and CPHA
        cr1 |= mode & SPI_CR1_MODE_MASK; // Set them
        write_volatile(&mut (*regs).cr1, cr1);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_set_bit_order(handle: *mut ln_spi_c, order: u32) {
    let idx = unpack_handle(handle);
    let regs = SpiRegisters::ptr(idx);
    unsafe {
        let mut cr1 = read_volatile(&mut (*regs).cr1);
        if order == spiBitOrder_SPI_LSBFIRST {
            cr1 |= SPI_CR1_LSBFIRST; // LSBFIRST
        } else {
            cr1 &= !SPI_CR1_LSBFIRST; // MSBFIRST
        }
        write_volatile(&mut (*regs).cr1, cr1);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_set_ssel(handle: *mut ln_spi_c, ssel: i32) {}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_set(handle: *mut ln_spi_c, st: *const lnSPISettings) {}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_write8(handle: *mut ln_spi_c, data: u8) -> bool {
    lnspi_transfer8_old(handle, data);
    true
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_write16(handle: *mut ln_spi_c, data: u16) -> bool {
    true
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_wait_for_completion(handle: *mut ln_spi_c) -> bool {
    true
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_block_write8(handle: *mut ln_spi_c, n: u32, data: *const u8) -> bool { true }
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_block_write16(handle: *mut ln_spi_c, n: u32, data: *const u16) -> bool { true }
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_block_write8_repeat(handle: *mut ln_spi_c, n: u32, data: u8) -> bool { true }
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_block_write16_repeat(handle: *mut ln_spi_c, n: u32, data: u16) -> bool { true }

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_asyncWrite8(handle: *mut ln_spi_c, n: u32, data: *const u8, cb: lnSpiCallback, ctx: *mut c_void, repeat: bool) -> bool {
    // Native DMA configuration goes here
    true
}
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_nextWrite8(handle: *mut ln_spi_c, n: u32, data: *const u8, cb: lnSpiCallback, ctx: *mut c_void, repeat: bool) -> bool { true }
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_asyncWrite16(handle: *mut ln_spi_c, n: u32, data: *const u16, cb: lnSpiCallback, ctx: *mut c_void, repeat: bool) -> bool { true }
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_nextWrite16(handle: *mut ln_spi_c, n: u32, data: *const u16, cb: lnSpiCallback, ctx: *mut c_void, repeat: bool) -> bool { true }
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_finishAsyncDma(handle: *mut ln_spi_c) -> bool { true }
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_waitForAsync(handle: *mut ln_spi_c) -> bool { true }

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_transfer8(handle: *mut ln_spi_c, count: u32, tx: *const u8, rx: *mut u8) -> bool {
    return true;
}
pub extern "C" fn lnspi_transfer8_old(handle: *mut ln_spi_c, val: u8) -> u8 {
    let idx = unpack_handle(handle);
    let regs = SpiRegisters::ptr(idx);
    unsafe {
        // Wait for TXE
        while (read_volatile(&mut (*regs).sr) & SPI_SR_TXE) == 0 {}
        // Write data
        write_volatile(&mut (*regs).dr, val as u32);
        // Wait for RXNE
        while (read_volatile(&mut (*regs).sr) & SPI_SR_RXNE) == 0 {}
        // Read data
        (read_volatile(&mut (*regs).dr) & SPI_DR_DATA_8BIT_MASK) as u8
    }
}
pub type spiBitOrder = u32;
pub const spiBitOrder_SPI_LSBFIRST: spiBitOrder = 0;
pub const spiBitOrder_SPI_MSBFIRST: spiBitOrder = 1;
pub type spiDataMode = u32;
pub const spiDataMode_SPI_MODE0: spiDataMode = 0;
pub const spiDataMode_SPI_MODE1: spiDataMode = 1;
pub const spiDataMode_SPI_MODE2: spiDataMode = 2;
pub const spiDataMode_SPI_MODE3: spiDataMode = 3;

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_set_dma_mode(handle: *mut ln_spi_c, enable: bool) {
    let instance = unpack_handle(handle);
    unsafe {
        SPI_USE_DMA[instance as usize] = enable;
    }
}
