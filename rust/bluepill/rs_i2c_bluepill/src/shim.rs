use crate::*;
use core::ffi::c_void;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ln_i2c_c {
    pub dummy: *mut c_void,
}

pub type lnI2cCallback = ::core::option::Option<unsafe extern "C" fn(arg1: *mut c_void)>;

#[inline(always)]
pub fn pack_handle(instance: u32) -> *mut ln_i2c_c {
    instance as *mut ln_i2c_c
}

#[inline(always)]
pub fn unpack_handle(handle: *mut ln_i2c_c) -> u32 {
    handle as usize as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn lni2c_create(instance: u32, speed: u32) -> *mut ln_i2c_c {
    let _i2c = I2c::new(instance, speed);
    pack_handle(instance)
}

#[unsafe(no_mangle)]
pub extern "C" fn lni2c_delete(_handle: *mut ln_i2c_c) {}

#[unsafe(no_mangle)]
pub extern "C" fn lni2c_setSpeed(handle: *mut ln_i2c_c, speed: u32) {
    let mut i2c = I2c { instance: unpack_handle(handle) };
    i2c.set_speed(speed);
}

#[unsafe(no_mangle)]
pub extern "C" fn lni2c_setAddress(handle: *mut ln_i2c_c, address: u32) {
    let mut i2c = I2c { instance: unpack_handle(handle) };
    i2c.set_address(address);
}

#[unsafe(no_mangle)]
pub extern "C" fn lni2c_write(handle: *mut ln_i2c_c, n: u32, data: *const u8) -> bool {
    let mut i2c = I2c { instance: unpack_handle(handle) };
    i2c.write(n, data)
}

#[unsafe(no_mangle)]
pub extern "C" fn lni2c_read(handle: *mut ln_i2c_c, n: u32, data: *mut u8) -> bool {
    let mut i2c = I2c { instance: unpack_handle(handle) };
    i2c.read(n, data)
}

#[unsafe(no_mangle)]
pub extern "C" fn lni2c_write_to(handle: *mut ln_i2c_c, target: u32, n: u32, data: *const u8) -> bool {
    let mut i2c = I2c { instance: unpack_handle(handle) };
    i2c.write_to(target, n, data)
}

#[unsafe(no_mangle)]
pub extern "C" fn lni2c_multi_write_to(
    handle: *mut ln_i2c_c,
    target: u32,
    nbSeqn: u32,
    seqLength: *const u32,
    data: *mut *const u8,
) -> bool {
    let mut i2c = I2c { instance: unpack_handle(handle) };
    i2c.multi_write_to(target, nbSeqn, seqLength, data)
}

#[unsafe(no_mangle)]
pub extern "C" fn lni2c_read_from(handle: *mut ln_i2c_c, target: u32, n: u32, data: *mut u8) -> bool {
    let mut i2c = I2c { instance: unpack_handle(handle) };
    i2c.read_from(target, n, data)
}

#[unsafe(no_mangle)]
pub extern "C" fn lni2c_begin(handle: *mut ln_i2c_c, target: u32) -> bool {
    let mut i2c = I2c { instance: unpack_handle(handle) };
    i2c.begin(target)
}

#[unsafe(no_mangle)]
pub extern "C" fn lni2c_set_dma_mode(handle: *mut ln_i2c_c, enable: bool) {
    let mut i2c = I2c { instance: unpack_handle(handle) };
    i2c.set_dma_mode(enable);
}
