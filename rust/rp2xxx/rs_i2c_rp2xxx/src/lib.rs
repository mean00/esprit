//! RP2040 / RP2350 stub backend for the `lnI2C` C ABI.
//!
//! esprit has no RP2xxx implementation of this peripheral: the C++ bridge
//! (`lnI2C_c.cpp`) is deliberately *not* built for RP2040/RP2350 — see the
//! `if(NOT (USE_RP2040 OR USE_RP2350))` guard in `esprit/rust/CMakeLists.txt`.
//!
//! This crate gives `rust_esprit` a linkable backend for RP2xxx anyway: it
//! exposes exactly the same public Rust entry points as `rs_i2c_bluepill` (and
//! `c_api::rn_i2c_c`), but every body is `unimplemented!(...)`.  Reaching one
//! therefore panics with a clear message instead of producing an obscure
//! undefined reference at link time.
//!
//! Nothing is expected to call these on RP2xxx: no C/C++ code there references
//! the `lni2c_*` symbols (`esprit_c_bindings` only builds `lnI2C_c.cpp` for the
//! non-RP2xxx targets).

#![no_std]
#![allow(unused_variables, non_camel_case_types, non_snake_case, dead_code)]

use core::ffi::c_void;

/// Opaque I²C handle — same shape as the C++ `ln_i2c_c` and as
/// `rs_i2c_bluepill::ln_i2c_c`.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ln_i2c_c {
    pub dummy: *mut c_void,
}

/// Create an I²C bus on `instance` running at `speed` Hz.
#[unsafe(no_mangle)]
pub extern "C" fn lni2c_create(instance: u32, speed: u32) -> *mut ln_i2c_c {
    unimplemented!("lni2c_create is not supported on RP2040 / RP2350")
}

/// Destroy a bus created by [`lni2c_create`].
#[unsafe(no_mangle)]
pub extern "C" fn lni2c_delete(i2c: *mut ln_i2c_c) {
    unimplemented!("lni2c_delete is not supported on RP2040 / RP2350")
}

/// Change the bus speed, in Hz.
#[unsafe(no_mangle)]
pub extern "C" fn lni2c_setSpeed(ptr: *mut ln_i2c_c, speed: u32) {
    unimplemented!("lni2c_setSpeed is not supported on RP2040 / RP2350")
}

/// Set the default 7-bit target address.
#[unsafe(no_mangle)]
pub extern "C" fn lni2c_setAddress(ptr: *mut ln_i2c_c, address: u32) {
    unimplemented!("lni2c_setAddress is not supported on RP2040 / RP2350")
}

/// Write `n` bytes to the currently addressed target.
#[unsafe(no_mangle)]
pub extern "C" fn lni2c_write(ptr: *mut ln_i2c_c, n: u32, data: *const u8) -> bool {
    unimplemented!("lni2c_write is not supported on RP2040 / RP2350")
}

/// Read `n` bytes from the currently addressed target.
#[unsafe(no_mangle)]
pub extern "C" fn lni2c_read(ptr: *mut ln_i2c_c, n: u32, data: *mut u8) -> bool {
    unimplemented!("lni2c_read is not supported on RP2040 / RP2350")
}

/// Write `n` bytes to `target` (START/STOP included).
#[unsafe(no_mangle)]
pub extern "C" fn lni2c_write_to(ptr: *mut ln_i2c_c, target: u32, n: u32, data: *const u8) -> bool {
    unimplemented!("lni2c_write_to is not supported on RP2040 / RP2350")
}

/// Write a sequence of buffers to `target` in a single transaction.
#[unsafe(no_mangle)]
pub extern "C" fn lni2c_multi_write_to(
    ptr: *mut ln_i2c_c,
    target: u32,
    nbSeqn: u32,
    seqLength: *const u32,
    data: *mut *const u8,
) -> bool {
    unimplemented!("lni2c_multi_write_to is not supported on RP2040 / RP2350")
}

/// Read `n` bytes from `target` (START/STOP included).
#[unsafe(no_mangle)]
pub extern "C" fn lni2c_read_from(ptr: *mut ln_i2c_c, target: u32, n: u32, data: *mut u8) -> bool {
    unimplemented!("lni2c_read_from is not supported on RP2040 / RP2350")
}

/// Send a START condition followed by the `target` address.
#[unsafe(no_mangle)]
pub extern "C" fn lni2c_begin(ptr: *mut ln_i2c_c, target: u32) -> bool {
    unimplemented!("lni2c_begin is not supported on RP2040 / RP2350")
}

/// Enable/disable DMA for transfers.
#[unsafe(no_mangle)]
pub extern "C" fn lni2c_set_dma_mode(ptr: *mut ln_i2c_c, enable: bool) {
    unimplemented!("lni2c_set_dma_mode is not supported on RP2040 / RP2350")
}
