//! C-ABI shims for `rs_spi_rp2xxx`, providing the `lnspi_*` functions.
//!
//! Mirrors `rs_spi_bluepill/src/shim.rs` and replaces `ln_rp_spi_c.cpp`.

#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use crate::*;
use core::ffi::c_void;

/// Legacy C ABI types for `rust_esprit::raw`.
pub const spiDataMode_SPI_MODE0: spiDataMode = 0;
pub const spiDataMode_SPI_MODE1: spiDataMode = 1;
pub const spiDataMode_SPI_MODE2: spiDataMode = 2;
pub const spiDataMode_SPI_MODE3: spiDataMode = 3;
pub type spiDataMode = u32;

pub const spiBitOrder_SPI_LSBFIRST: spiBitOrder = 0;
pub const spiBitOrder_SPI_MSBFIRST: spiBitOrder = 1;
pub type spiBitOrder = u32;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ln_spi_c {
    pub dummy: *mut c_void,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct lnSPISettings {
    pub pin_cs: i32,
    pub speed: u32,
    pub bit_order: u32,
    pub data_mode: u32,
}

pub type lnSpiCallback = Option<unsafe extern "C" fn(cookie: *mut c_void)>;

pub(crate) static mut SPI_INSTANCES: [Option<Spi>; SPI_MAX_INSTANCES] = [None, None];
static mut SPI_HANDLES: [u32; SPI_MAX_INSTANCES] = [0; SPI_MAX_INSTANCES];

const HANDLE_BIAS: usize = 1;

#[inline(always)]
pub fn pack_handle(instance: u32) -> *mut ln_spi_c {
    (instance as usize + HANDLE_BIAS) as *mut ln_spi_c
}

#[inline(always)]
pub fn unpack_handle(handle: *mut ln_spi_c) -> u32 {
    (handle as usize).saturating_sub(HANDLE_BIAS) as u32
}

#[inline(always)]
fn with_spi<R>(handle: *mut ln_spi_c, default: R, f: impl FnOnce(&mut Spi) -> R) -> R {
    let index = unpack_handle(handle) as usize;
    if index < SPI_MAX_INSTANCES {
        unsafe {
            if let Some(spi) = SPI_INSTANCES[index].as_mut() {
                return f(spi);
            }
        }
    }
    default
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_create(instance: u32, pinCs: i32) -> *mut ln_spi_c {
    let index = instance as usize;
    if index >= SPI_MAX_INSTANCES {
        return core::ptr::null_mut();
    }
    unsafe {
        if SPI_INSTANCES[index].is_none() {
            SPI_INSTANCES[index] = Some(Spi::new(instance, pinCs));
        }
        SPI_HANDLES[index] = SPI_HANDLES[index].saturating_add(1);
    }
    pack_handle(instance)
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_delete(handle: *mut ln_spi_c) {
    let index = unpack_handle(handle) as usize;
    if index < SPI_MAX_INSTANCES {
        unsafe {
            let ref_count = SPI_HANDLES[index].saturating_sub(1);
            SPI_HANDLES[index] = ref_count;
            if ref_count == 0 {
                SPI_INSTANCES[index] = None;
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_begin(handle: *mut ln_spi_c, dataSize: u32) {
    with_spi(handle, (), |s| s.begin(dataSize));
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_end(handle: *mut ln_spi_c) {
    with_spi(handle, (), |s| s.end());
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_set_speed(handle: *mut ln_spi_c, speed: u32) {
    with_spi(handle, (), |s| s.set_speed(speed));
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_set_data_mode(handle: *mut ln_spi_c, mode: u32) {
    with_spi(handle, (), |s| s.set_data_mode(SpiMode::from(mode)));
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_set_bit_order(handle: *mut ln_spi_c, order: u32) {
    with_spi(handle, (), |s| s.set_bit_order(BitOrder::from(order)));
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_set_ssel(handle: *mut ln_spi_c, ssel: i32) {
    with_spi(handle, (), |s| s.set_ssel(ssel));
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_set_dma_mode(handle: *mut ln_spi_c, enable: bool) {
    with_spi(handle, (), |s| s.set_dma_mode(enable));
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_set(handle: *mut ln_spi_c, st: *const lnSPISettings) {
    if !st.is_null() {
        let settings = unsafe { &*st };
        with_spi(handle, (), |s| {
            s.set_ssel(settings.pin_cs);
            s.set_bit_order(BitOrder::from(settings.bit_order));
            s.set_data_mode(SpiMode::from(settings.data_mode));
            s.set_speed(settings.speed);
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_write8(handle: *mut ln_spi_c, data: u8) -> bool {
    with_spi(handle, false, |s| s.write8(data))
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_write16(handle: *mut ln_spi_c, data: u16) -> bool {
    with_spi(handle, false, |s| s.write16(data))
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_wait_for_completion(handle: *mut ln_spi_c) -> bool {
    with_spi(handle, false, |s| s.wait_for_completion())
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_block_write8(handle: *mut ln_spi_c, nbByte: u32, data: *const u8) -> bool {
    if data.is_null() {
        return false;
    }
    let slice = unsafe { core::slice::from_raw_parts(data, nbByte as usize) };
    with_spi(handle, false, |s| s.block_write8(slice))
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_block_write16(handle: *mut ln_spi_c, nbWord: u32, data: *const u16) -> bool {
    if data.is_null() {
        return false;
    }
    let slice = unsafe { core::slice::from_raw_parts(data, nbWord as usize) };
    with_spi(handle, false, |s| s.block_write16(slice))
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_block_write8_repeat(handle: *mut ln_spi_c, nbByte: u32, data: u8) -> bool {
    with_spi(handle, false, |s| s.block_write8_repeat(nbByte, data))
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_block_write16_repeat(handle: *mut ln_spi_c, nbWord: u32, data: u16) -> bool {
    with_spi(handle, false, |s| s.block_write16_repeat(nbWord, data))
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_asyncWrite8(
    handle: *mut ln_spi_c,
    nbBytes: u32,
    data: *const u8,
    cb: lnSpiCallback,
    cookie: *mut c_void,
    repeat: bool,
) -> bool {
    if data.is_null() {
        return false;
    }
    let slice = unsafe { core::slice::from_raw_parts(data, nbBytes as usize) };
    with_spi(handle, false, |s| s.async_write8(slice, cb, cookie, repeat))
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_async_write8(
    handle: *mut ln_spi_c,
    nbBytes: u32,
    data: *const u8,
    cb: lnSpiCallback,
    cookie: *mut c_void,
    repeat: bool,
) -> bool {
    lnspi_asyncWrite8(handle, nbBytes, data, cb, cookie, repeat)
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_nextWrite8(
    handle: *mut ln_spi_c,
    nbBytes: u32,
    data: *const u8,
    cb: lnSpiCallback,
    cookie: *mut c_void,
    repeat: bool,
) -> bool {
    lnspi_asyncWrite8(handle, nbBytes, data, cb, cookie, repeat)
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_next_write8(
    handle: *mut ln_spi_c,
    nbBytes: u32,
    data: *const u8,
    cb: lnSpiCallback,
    cookie: *mut c_void,
    repeat: bool,
) -> bool {
    lnspi_asyncWrite8(handle, nbBytes, data, cb, cookie, repeat)
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_asyncWrite16(
    handle: *mut ln_spi_c,
    nbWords: u32,
    data: *const u16,
    cb: lnSpiCallback,
    cookie: *mut c_void,
    repeat: bool,
) -> bool {
    if data.is_null() {
        return false;
    }
    let slice = unsafe { core::slice::from_raw_parts(data, nbWords as usize) };
    with_spi(handle, false, |s| s.async_write16(slice, cb, cookie, repeat))
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_async_write16(
    handle: *mut ln_spi_c,
    nbWords: u32,
    data: *const u16,
    cb: lnSpiCallback,
    cookie: *mut c_void,
    repeat: bool,
) -> bool {
    lnspi_asyncWrite16(handle, nbWords, data, cb, cookie, repeat)
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_nextWrite16(
    handle: *mut ln_spi_c,
    nbWords: u32,
    data: *const u16,
    cb: lnSpiCallback,
    cookie: *mut c_void,
    repeat: bool,
) -> bool {
    lnspi_asyncWrite16(handle, nbWords, data, cb, cookie, repeat)
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_next_write16(
    handle: *mut ln_spi_c,
    nbWords: u32,
    data: *const u16,
    cb: lnSpiCallback,
    cookie: *mut c_void,
    repeat: bool,
) -> bool {
    lnspi_asyncWrite16(handle, nbWords, data, cb, cookie, repeat)
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_finish_async_dma(handle: *mut ln_spi_c) -> bool {
    with_spi(handle, false, |s| s.finish_async_dma())
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_finishAsyncDma(handle: *mut ln_spi_c) -> bool {
    lnspi_finish_async_dma(handle)
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_wait_for_async(handle: *mut ln_spi_c) -> bool {
    with_spi(handle, false, |s| s.wait_for_async())
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_waitForAsync(handle: *mut ln_spi_c) -> bool {
    lnspi_wait_for_async(handle)
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_transfer8(
    handle: *mut ln_spi_c,
    nbBytes: u32,
    dataOut: *const u8,
    dataIn: *mut u8,
) -> bool {
    if dataOut.is_null() || dataIn.is_null() {
        return false;
    }
    let tx = unsafe { core::slice::from_raw_parts(dataOut, nbBytes as usize) };
    let rx = unsafe { core::slice::from_raw_parts_mut(dataIn, nbBytes as usize) };
    with_spi(handle, false, |s| s.transfer8(tx, rx))
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_transfer16(
    handle: *mut ln_spi_c,
    nbWords: u32,
    dataOut: *const u16,
    dataIn: *mut u16,
) -> bool {
    if dataOut.is_null() || dataIn.is_null() {
        return false;
    }
    let tx = unsafe { core::slice::from_raw_parts(dataOut, nbWords as usize) };
    let rx = unsafe { core::slice::from_raw_parts_mut(dataIn, nbWords as usize) };
    with_spi(handle, false, |s| s.dma_transfer16(tx, rx))
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_read8(handle: *mut ln_spi_c, nbBytes: u32, dataIn: *mut u8) -> bool {
    if dataIn.is_null() {
        return false;
    }
    let rx = unsafe { core::slice::from_raw_parts_mut(dataIn, nbBytes as usize) };
    with_spi(handle, false, |s| s.read8(rx))
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_read16(handle: *mut ln_spi_c, nbWords: u32, dataIn: *mut u16) -> bool {
    if dataIn.is_null() {
        return false;
    }
    let rx = unsafe { core::slice::from_raw_parts_mut(dataIn, nbWords as usize) };
    with_spi(handle, false, |s| s.dma_read16(rx))
}
