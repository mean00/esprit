//! RP2040 / RP2350 stub backend for the `lnSPI` C ABI.
//!
//! esprit has no RP2xxx implementation of this peripheral: the C++ bridge
//! (`lnSPI_c.cpp`) is only added to `esprit_c_bindings` under `LN_ENABLE_SPI`
//! and the RP2xxx build has no Rust SPI driver either — see
//! `esprit/rust/CMakeLists.txt`.
//!
//! This crate gives `rust_esprit` a linkable backend for RP2xxx anyway: it
//! exposes exactly the same public Rust entry points and types as
//! `rs_spi_bluepill` (and `c_api::rn_spi_c`), but every body is
//! `unimplemented!(...)`.  Reaching one therefore panics with a clear message
//! instead of producing an obscure undefined reference at link time.
//!
//! The type/constant names must match `c_api::rn_spi_c` exactly, because
//! `rust_esprit::raw` re-exports them (`lnSpiCallback`, `lnSPISettings`,
//! `ln_spi_c`, `spiBitOrder`, `spiDataMode`, …).

#![no_std]
#![allow(unused_variables, non_upper_case_globals)]
#![allow(non_camel_case_types, non_snake_case, dead_code)]

use core::ffi::c_void;

pub const spiDataMode_SPI_MODE0: spiDataMode = 0;
pub const spiDataMode_SPI_MODE1: spiDataMode = 1;
pub const spiDataMode_SPI_MODE2: spiDataMode = 2;
pub const spiDataMode_SPI_MODE3: spiDataMode = 3;
/// SPI clock polarity/phase combination.
pub type spiDataMode = u32;

pub const spiBitOrder_SPI_LSBFIRST: spiBitOrder = 0;
pub const spiBitOrder_SPI_MSBFIRST: spiBitOrder = 1;
/// Bit order within a byte.
pub type spiBitOrder = u32;

/// Full SPI configuration block — mirrors `lnSPISettings` from `lnSPI_c.h`.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct lnSPISettings {
    pub pinCS: i32,
    pub speed: u32,
    pub bOrder: spiBitOrder,
    pub dMode: spiDataMode,
}

/// Completion callback for the asynchronous (`*Async*`) transfers.
pub type lnSpiCallback = ::core::option::Option<unsafe extern "C" fn(cookie: *mut c_void)>;

/// Opaque SPI handle — same shape as the C++ `ln_spi_c` and as
/// `rs_spi_bluepill::ln_spi_c`.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ln_spi_c {
    pub dummy: *mut c_void,
}

/// Create an SPI bus on `instance`; `pinCs` is the C `Pin` enum value.
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_create(instance: u32, pinCs: i32) -> *mut ln_spi_c {
    unimplemented!("lnspi_create is not supported on RP2040 / RP2350")
}

/// Destroy a bus created by [`lnspi_create`].
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_delete(instance: *mut ln_spi_c) {
    unimplemented!("lnspi_delete is not supported on RP2040 / RP2350")
}

/// Initialise the bus for `dataSize`-bit frames (typically 8).
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_begin(instance: *mut ln_spi_c, dataSize: u32) {
    unimplemented!("lnspi_begin is not supported on RP2040 / RP2350")
}

/// Enable/disable DMA for transfers.
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_set_dma_mode(ptr: *mut ln_spi_c, enable: bool) {
    unimplemented!("lnspi_set_dma_mode is not supported on RP2040 / RP2350")
}

/// De-initialise the bus.
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_end(instance: *mut ln_spi_c) {
    unimplemented!("lnspi_end is not supported on RP2040 / RP2350")
}

/// Set the bit order (MSB/LSB first).
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_set_bit_order(instance: *mut ln_spi_c, order: spiBitOrder) {
    unimplemented!("lnspi_set_bit_order is not supported on RP2040 / RP2350")
}

/// Set the clock polarity/phase mode.
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_set_data_mode(instance: *mut ln_spi_c, datamode: spiDataMode) {
    unimplemented!("lnspi_set_data_mode is not supported on RP2040 / RP2350")
}

/// Set the clock speed, in Hz.
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_set_speed(instance: *mut ln_spi_c, speed: u32) {
    unimplemented!("lnspi_set_speed is not supported on RP2040 / RP2350")
}

/// Set the chip-select pin (C `Pin` enum value).
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_set_ssel(instance: *mut ln_spi_c, ssel: i32) {
    unimplemented!("lnspi_set_ssel is not supported on RP2040 / RP2350")
}

/// Apply a complete configuration block.
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_set(instance: *mut ln_spi_c, st: *const lnSPISettings) {
    unimplemented!("lnspi_set is not supported on RP2040 / RP2350")
}

// ---------- synchronous transfers ----------

/// Write one byte; returns `true` on success.
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_write8(instance: *mut ln_spi_c, data: u8) -> bool {
    unimplemented!("lnspi_write8 is not supported on RP2040 / RP2350")
}

/// Write one 16-bit word; returns `true` on success.
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_write16(instance: *mut ln_spi_c, data: u16) -> bool {
    unimplemented!("lnspi_write16 is not supported on RP2040 / RP2350")
}

/// Wait for the in-flight transfer to complete.
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_wait_for_completion(instance: *mut ln_spi_c) -> bool {
    unimplemented!("lnspi_wait_for_completion is not supported on RP2040 / RP2350")
}

/// Write the same 16-bit word `nbWord` times.
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_block_write16_repeat(instance: *mut ln_spi_c, nbWord: u32, data: u16) -> bool {
    unimplemented!("lnspi_block_write16_repeat is not supported on RP2040 / RP2350")
}

/// Write the same byte `nbByte` times.
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_block_write8_repeat(instance: *mut ln_spi_c, nbByte: u32, data: u8) -> bool {
    unimplemented!("lnspi_block_write8_repeat is not supported on RP2040 / RP2350")
}

/// Write `nbWord` 16-bit words from `data`.
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_block_write16(instance: *mut ln_spi_c, nbWord: u32, data: *const u16) -> bool {
    unimplemented!("lnspi_block_write16 is not supported on RP2040 / RP2350")
}

/// Write `nbByte` bytes from `data`.
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_block_write8(instance: *mut ln_spi_c, nbByte: u32, data: *const u8) -> bool {
    unimplemented!("lnspi_block_write8 is not supported on RP2040 / RP2350")
}

// ---------- asynchronous (DMA) transfers ----------

/// Start an asynchronous (DMA) byte transfer.
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_asyncWrite8(
    instance: *mut ln_spi_c,
    nbBytes: u32,
    data: *const u8,
    cb: lnSpiCallback,
    cookie: *mut c_void,
    repeat: bool,
) -> bool {
    unimplemented!("lnspi_asyncWrite8 is not supported on RP2040 / RP2350")
}

/// Queue the next chunk of an asynchronous byte transfer.
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_nextWrite8(
    instance: *mut ln_spi_c,
    nbBytes: u32,
    data: *const u8,
    cb: lnSpiCallback,
    cookie: *mut c_void,
    repeat: bool,
) -> bool {
    unimplemented!("lnspi_nextWrite8 is not supported on RP2040 / RP2350")
}

/// Start an asynchronous (DMA) 16-bit word transfer.
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_asyncWrite16(
    instance: *mut ln_spi_c,
    nbWords: u32,
    data: *const u16,
    cb: lnSpiCallback,
    cookie: *mut c_void,
    repeat: bool,
) -> bool {
    unimplemented!("lnspi_asyncWrite16 is not supported on RP2040 / RP2350")
}

/// Queue the next chunk of an asynchronous 16-bit word transfer.
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_nextWrite16(
    instance: *mut ln_spi_c,
    nbWords: u32,
    data: *const u16,
    cb: lnSpiCallback,
    cookie: *mut c_void,
    repeat: bool,
) -> bool {
    unimplemented!("lnspi_nextWrite16 is not supported on RP2040 / RP2350")
}

/// Finalise an asynchronous transfer.
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_finishAsyncDma(instance: *mut ln_spi_c) -> bool {
    unimplemented!("lnspi_finishAsyncDma is not supported on RP2040 / RP2350")
}

/// Wait for an asynchronous transfer to complete.
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_waitForAsync(instance: *mut ln_spi_c) -> bool {
    unimplemented!("lnspi_waitForAsync is not supported on RP2040 / RP2350")
}

/// Full-duplex byte transfer.
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_transfer8(instance: *mut ln_spi_c, nb: u32, tx: *const u8, rx: *mut u8) -> bool {
    unimplemented!("lnspi_transfer8 is not supported on RP2040 / RP2350")
}
