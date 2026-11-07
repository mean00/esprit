//! C ABI of the SPI driver, i.e. the `lnspi_*` functions declared in
//! `lnSPI_c.h`.  They replace `lnSPI_c.cpp` in the Rust builds.
//!
//! Handles are instance numbers (`ln_spi_c *` is never dereferenced), and the
//! `Spi` objects live in [`SPI_INSTANCES`], which keeps the CS pin, the
//! settings and the claimed DMA channel alive between calls — the previous
//! version rebuilt a `Spi` per call, which silently dropped all of that.

use crate::*;
use core::ffi::c_void;

/// Opaque handle, same shape as the C++ `ln_spi_c`.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ln_spi_c {
    pub dummy: *mut c_void,
}

/// `lnSPISettings` from `lnSPI_defines.h` — **field order matters**.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct lnSPISettings {
    pub pin_cs: i32,
    pub speed: u32,
    pub bit_order: u32,
    pub data_mode: u32,
}

/// `typedef void lnSpiCallback(void *cookie);`
pub type lnSpiCallback = ::core::option::Option<unsafe extern "C" fn(cookie: *mut c_void)>;

/// The `Spi` objects created through the C ABI, one per instance.
pub(crate) static mut SPI_INSTANCES: [Option<Spi>; SPI_MAX_INSTANCES] = [None, None, None];

/// Number of live handles per instance (`rcSPI` in `lnSPI_c.cpp`): the object is
/// only destroyed when the last handle is deleted, so two `Spi` objects on the
/// same instance do not free the bus from under each other.
static mut SPI_HANDLES: [u32; SPI_MAX_INSTANCES] = [0; SPI_MAX_INSTANCES];

/// Handles are not pointers: they only have to be unique and non-null, and the
/// instance number is all the state needed (`ln_spi_c` is opaque and never
/// dereferenced).  The offset keeps instance 0 away from `NULL`, which the C
/// callers (`rust_esprit::spi`) read as "creation failed".
const HANDLE_BIAS: usize = 1;

#[inline(always)]
pub fn pack_handle(instance: u32) -> *mut ln_spi_c {
    (instance as usize + HANDLE_BIAS) as *mut ln_spi_c
}

#[inline(always)]
pub fn unpack_handle(handle: *mut ln_spi_c) -> u32 {
    // `saturating_sub` maps the `NULL` handle to a number that no instance has.
    (handle as usize).saturating_sub(HANDLE_BIAS) as u32
}

/// Run `f` on the `Spi` behind `handle`, or return `default` when there is none.
#[inline(always)]
fn with_spi<R>(handle: *mut ln_spi_c, default: R, f: impl FnOnce(&mut Spi) -> R) -> R {
    let index = unpack_handle(handle) as usize;
    if index < SPI_MAX_INSTANCES {
        // SAFETY: the objects in the table never move, and the C ABI is called
        // from one context at a time per instance.
        if let Some(spi) = unsafe { SPI_INSTANCES[index].as_mut() } {
            return f(spi);
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
    // SAFETY: see `with_spi`; creating twice returns the same object, like the
    // reference counting in `lnSPI_c.cpp`.
    unsafe {
        if SPI_INSTANCES[index].is_none() {
            SPI_INSTANCES[index] = Some(Spi::new(instance, pinCs));
        }
        SPI_HANDLES[index] += 1;
    }
    pack_handle(instance)
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_delete(handle: *mut ln_spi_c) {
    let index = unpack_handle(handle) as usize;
    if index < SPI_MAX_INSTANCES {
        // SAFETY: see `with_spi`.  Dropping the last handle releases the CS pin
        // and the DMA channel (`Drop for Spi`, `DmaChannel::drop`).
        unsafe {
            if SPI_HANDLES[index] > 0 {
                SPI_HANDLES[index] -= 1;
            }
            if SPI_HANDLES[index] == 0 {
                if let Some(spi) = SPI_INSTANCES[index].as_mut() {
                    spi.end();
                }
                SPI_INSTANCES[index] = None;
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_begin(handle: *mut ln_spi_c, dataSize: u32) {
    with_spi(handle, (), |spi| spi.begin(dataSize));
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_end(handle: *mut ln_spi_c) {
    with_spi(handle, (), |spi| spi.end());
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_set_speed(handle: *mut ln_spi_c, speed: u32) {
    with_spi(handle, (), |spi| spi.set_speed(speed));
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_set_data_mode(handle: *mut ln_spi_c, mode: u32) {
    with_spi(handle, (), |spi| spi.set_data_mode(mode));
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_set_bit_order(handle: *mut ln_spi_c, order: u32) {
    with_spi(handle, (), |spi| spi.set_bit_order(order));
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_set_ssel(handle: *mut ln_spi_c, ssel: i32) {
    with_spi(handle, (), |spi| spi.set_ssel(ssel));
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_set_dma_mode(handle: *mut ln_spi_c, enable: bool) {
    with_spi(handle, (), |spi| spi.set_dma_mode(enable));
}

/// Apply a whole `lnSPISettings` block (speed, bit order, data mode, CS pin).
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_set(handle: *mut ln_spi_c, st: *const lnSPISettings) {
    if st.is_null() {
        return;
    }
    // SAFETY: the caller passes a valid `lnSPISettings`.
    let st = unsafe { &*st };
    with_spi(handle, (), |spi| {
        spi.set_ssel(st.pin_cs);
        spi.set_speed(st.speed);
        spi.set_bit_order(st.bit_order);
        spi.set_data_mode(st.data_mode);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_write8(handle: *mut ln_spi_c, data: u8) -> bool {
    with_spi(handle, false, |spi| spi.write8(data))
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_write16(handle: *mut ln_spi_c, data: u16) -> bool {
    with_spi(handle, false, |spi| spi.write16(data))
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_wait_for_completion(handle: *mut ln_spi_c) -> bool {
    with_spi(handle, false, |spi| spi.wait_for_completion())
}

// ---- block (DMA) writes ---------------------------------------------------

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_block_write8(handle: *mut ln_spi_c, n: u32, data: *const u8) -> bool {
    if data.is_null() {
        return false;
    }
    // SAFETY: the caller passes `n` bytes at `data`.
    let data = unsafe { core::slice::from_raw_parts(data, n as usize) };
    with_spi(handle, false, |spi| spi.block_write8(data))
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_block_write16(handle: *mut ln_spi_c, n: u32, data: *const u16) -> bool {
    if data.is_null() {
        return false;
    }
    // SAFETY: the caller passes `n` words at `data`.
    let data = unsafe { core::slice::from_raw_parts(data, n as usize) };
    with_spi(handle, false, |spi| spi.block_write16(data))
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_block_write8_repeat(handle: *mut ln_spi_c, n: u32, data: u8) -> bool {
    with_spi(handle, false, |spi| spi.block_write8_repeat(n, data))
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_block_write16_repeat(handle: *mut ln_spi_c, n: u32, data: u16) -> bool {
    with_spi(handle, false, |spi| spi.block_write16_repeat(n, data))
}

// ---- async (DMA interrupt driven) writes ----------------------------------

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_asyncWrite8(
    handle: *mut ln_spi_c,
    n: u32,
    data: *const u8,
    cb: lnSpiCallback,
    cookie: *mut c_void,
    repeat: bool,
) -> bool {
    if data.is_null() {
        return false;
    }
    // SAFETY: the caller passes `n` bytes at `data`.
    let data = unsafe { core::slice::from_raw_parts(data, n as usize) };
    with_spi(handle, false, |spi| spi.async_write8(data, cb, cookie, repeat))
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_nextWrite8(
    handle: *mut ln_spi_c,
    n: u32,
    data: *const u8,
    cb: lnSpiCallback,
    cookie: *mut c_void,
    repeat: bool,
) -> bool {
    if data.is_null() {
        return false;
    }
    // SAFETY: the caller passes `n` bytes at `data`.
    let data = unsafe { core::slice::from_raw_parts(data, n as usize) };
    with_spi(handle, false, |spi| spi.next_write8(data, cb, cookie, repeat))
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_asyncWrite16(
    handle: *mut ln_spi_c,
    n: u32,
    data: *const u16,
    cb: lnSpiCallback,
    cookie: *mut c_void,
    repeat: bool,
) -> bool {
    if data.is_null() {
        return false;
    }
    // SAFETY: the caller passes `n` words at `data`.
    let data = unsafe { core::slice::from_raw_parts(data, n as usize) };
    with_spi(handle, false, |spi| spi.async_write16(data, cb, cookie, repeat))
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_nextWrite16(
    handle: *mut ln_spi_c,
    n: u32,
    data: *const u16,
    cb: lnSpiCallback,
    cookie: *mut c_void,
    repeat: bool,
) -> bool {
    if data.is_null() {
        return false;
    }
    // SAFETY: the caller passes `n` words at `data`.
    let data = unsafe { core::slice::from_raw_parts(data, n as usize) };
    with_spi(handle, false, |spi| spi.next_write16(data, cb, cookie, repeat))
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_finishAsyncDma(handle: *mut ln_spi_c) -> bool {
    with_spi(handle, false, |spi| spi.finish_async_dma())
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_waitForAsync(handle: *mut ln_spi_c) -> bool {
    with_spi(handle, false, |spi| spi.wait_async())
}

// ---- full duplex ----------------------------------------------------------

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_transfer8(
    handle: *mut ln_spi_c,
    count: u32,
    tx: *const u8,
    rx: *mut u8,
) -> bool {
    if tx.is_null() || rx.is_null() {
        return false;
    }
    // SAFETY: the caller passes two buffers of `count` bytes.
    let (tx, rx) = unsafe {
        (
            core::slice::from_raw_parts(tx, count as usize),
            core::slice::from_raw_parts_mut(rx, count as usize),
        )
    };
    with_spi(handle, false, |spi| spi.transfer(tx, rx))
}
