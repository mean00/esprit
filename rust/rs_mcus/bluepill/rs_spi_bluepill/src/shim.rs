//! C ABI of the SPI driver, i.e. the `lnspi_*` functions declared in
//! `lnSPI_c.h`.  They replace `lnSPI_c.cpp` in the Rust builds.
//!
//! Handles are instance numbers (`ln_spi_c *` is never dereferenced), and the
//! `Spi` objects live in [`SPI_SLOTS`] together with their handle refcount, which
//! keeps the CS pin, the settings and the claimed DMA channel alive between calls
//! — the previous version rebuilt a `Spi` per call, which silently dropped all of
//! that.

use crate::*;
use core::ffi::c_void;
use rs_rtos::sync_cell::SyncCell;

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

/// One instance: the `Spi` object and the number of live handles on it, kept in
/// the same slot so the two can never disagree.
struct SpiSlot {
    spi: Option<Spi>,
    /// `rcSPI` in `lnSPI_c.cpp`: the object is only destroyed when the last handle
    /// is deleted, so two `Spi` objects on one instance do not free the bus from
    /// under each other.
    handles: u32,
}

/// The `Spi` objects created through the C ABI, one slot per instance.
///
/// A `SyncCell` rather than a `static mut` (rule §6): `with_spi` takes `&mut` for
/// exactly the duration of one call, and every access states the invariant — the
/// C ABI is driven from one task per instance, which is also what
/// `lnSPI_c.cpp`'s plain `static lnSPI *` array assumed.
static SPI_SLOTS: SyncCell<[SpiSlot; SPI_MAX_INSTANCES]> = SyncCell::new(
    [const {
        SpiSlot {
            spi: None,
            handles: 0,
        }
    }; SPI_MAX_INSTANCES],
);

/// Run `f` on the `Spi` behind `handle`, or return `default` when there is none.
///
/// Deliberately not a critical section: `f` may run a whole DMA transfer, so the
/// cell is entered through the documented one-owner-per-instance contract instead.
#[inline(always)]
fn with_spi<R>(handle: *mut ln_spi_c, default: R, f: impl FnOnce(&mut Spi) -> R) -> R {
    let index = unpack_handle(handle) as usize;
    if index >= SPI_MAX_INSTANCES {
        return default;
    }
    // SAFETY: one task drives the C ABI for a given instance, and `f` is the only
    // user of the object while it runs.
    unsafe {
        match SPI_SLOTS.as_mut()[index].spi.as_mut() {
            Some(spi) => f(spi),
            None => default,
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_create(instance: u32, pinCs: i32) -> *mut ln_spi_c {
    let index = instance as usize;
    if index >= SPI_MAX_INSTANCES {
        return core::ptr::null_mut();
    }
    // SAFETY: see `with_spi`; creating twice returns the same object and only
    // bumps the reference count, like `lnSPI_c.cpp`.
    unsafe {
        let slot = &mut SPI_SLOTS.as_mut()[index];
        if slot.spi.is_none() {
            slot.spi = Some(Spi::new(instance, pinCs));
        }
        slot.handles = slot.handles.saturating_add(1);
    }
    pack_handle(instance)
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_delete(handle: *mut ln_spi_c) {
    let index = unpack_handle(handle) as usize;
    if index >= SPI_MAX_INSTANCES {
        return;
    }
    // SAFETY: see `with_spi`. Dropping the last handle releases the CS pin and the
    // DMA channel (`Drop for Spi`, `DmaChannel::drop`).
    unsafe {
        let slot = &mut SPI_SLOTS.as_mut()[index];
        // A NULL handle unpacks to instance 0 with no handle having been taken;
        // the refcount is what tells the two apart, so nothing is torn down.
        if slot.handles == 0 {
            return;
        }
        slot.handles -= 1;
        if slot.handles == 0 {
            if let Some(spi) = slot.spi.as_mut() {
                spi.end();
            }
            slot.spi = None;
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
