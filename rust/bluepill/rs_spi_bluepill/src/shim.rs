use crate::*;
use core::ffi::c_void;

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

#[inline(always)]
pub fn pack_handle(instance: u32) -> *mut ln_spi_c {
    instance as *mut ln_spi_c
}

#[inline(always)]
pub fn unpack_handle(handle: *mut ln_spi_c) -> u32 {
    handle as usize as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_create(instance: u32, pinCs: i32) -> *mut ln_spi_c {
    let _spi = Spi::new(instance, pinCs);
    pack_handle(instance)
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_delete(_handle: *mut ln_spi_c) {}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_begin(handle: *mut ln_spi_c, dataSize: u32) {
    let mut spi = Spi { instance: unpack_handle(handle), pin_cs: -1 };
    spi.begin(dataSize);
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_end(handle: *mut ln_spi_c) {
    let mut spi = Spi { instance: unpack_handle(handle), pin_cs: -1 };
    spi.end();
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_set_speed(handle: *mut ln_spi_c, speed: u32) {
    let mut spi = Spi { instance: unpack_handle(handle), pin_cs: -1 };
    spi.set_speed(speed);
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_set_data_mode(handle: *mut ln_spi_c, mode: u32) {
    let mut spi = Spi { instance: unpack_handle(handle), pin_cs: -1 };
    spi.set_data_mode(mode);
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_set_bit_order(handle: *mut ln_spi_c, order: u32) {
    let mut spi = Spi { instance: unpack_handle(handle), pin_cs: -1 };
    spi.set_bit_order(order);
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_set_ssel(handle: *mut ln_spi_c, ssel: i32) {
    let mut spi = Spi { instance: unpack_handle(handle), pin_cs: -1 };
    spi.set_ssel(ssel);
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_set(_handle: *mut ln_spi_c, _st: *const lnSPISettings) {}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_write8(handle: *mut ln_spi_c, data: u8) -> bool {
    let mut spi = Spi { instance: unpack_handle(handle), pin_cs: -1 };
    spi.write8(data)
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_write16(handle: *mut ln_spi_c, data: u16) -> bool {
    let mut spi = Spi { instance: unpack_handle(handle), pin_cs: -1 };
    spi.write16(data)
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_wait_for_completion(handle: *mut ln_spi_c) -> bool {
    let spi = Spi { instance: unpack_handle(handle), pin_cs: -1 };
    spi.wait_for_completion()
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_block_write8(_handle: *mut ln_spi_c, _n: u32, _data: *const u8) -> bool { true }
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_block_write16(_handle: *mut ln_spi_c, _n: u32, _data: *const u16) -> bool { true }
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_block_write8_repeat(_handle: *mut ln_spi_c, _n: u32, _data: u8) -> bool { true }
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_block_write16_repeat(_handle: *mut ln_spi_c, _n: u32, _data: u16) -> bool { true }

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_asyncWrite8(_handle: *mut ln_spi_c, _n: u32, _data: *const u8, _cb: lnSpiCallback, _ctx: *mut c_void, _repeat: bool) -> bool {
    true
}
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_nextWrite8(_handle: *mut ln_spi_c, _n: u32, _data: *const u8, _cb: lnSpiCallback, _ctx: *mut c_void, _repeat: bool) -> bool { true }
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_asyncWrite16(_handle: *mut ln_spi_c, _n: u32, _data: *const u16, _cb: lnSpiCallback, _ctx: *mut c_void, _repeat: bool) -> bool { true }
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_nextWrite16(_handle: *mut ln_spi_c, _n: u32, _data: *const u16, _cb: lnSpiCallback, _ctx: *mut c_void, _repeat: bool) -> bool { true }
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_finishAsyncDma(_handle: *mut ln_spi_c) -> bool { true }
#[unsafe(no_mangle)]
pub extern "C" fn lnspi_waitForAsync(_handle: *mut ln_spi_c) -> bool { true }

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_transfer8(_handle: *mut ln_spi_c, _count: u32, _tx: *const u8, _rx: *mut u8) -> bool {
    true
}

pub extern "C" fn lnspi_transfer8_old(handle: *mut ln_spi_c, val: u8) -> u8 {
    let mut spi = Spi { instance: unpack_handle(handle), pin_cs: -1 };
    spi.transfer8(val)
}

#[unsafe(no_mangle)]
pub extern "C" fn lnspi_set_dma_mode(handle: *mut ln_spi_c, enable: bool) {
    let mut spi = Spi { instance: unpack_handle(handle), pin_cs: -1 };
    spi.set_dma_mode(enable);
}
