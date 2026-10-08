#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unsafe_op_in_unsafe_fn)]

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ln_i2c_c {
    pub dummy: *mut cty::c_void,
}

unsafe extern "C" {
    pub fn lni2c_create(instance: cty::c_uint, speed: cty::c_uint) -> *mut ln_i2c_c;
    pub fn lni2c_delete(i2c: *mut ln_i2c_c);
    pub fn lni2c_setSpeed(ptr: *mut ln_i2c_c, speed: cty::c_uint);
    pub fn lni2c_setAddress(ptr: *mut ln_i2c_c, address: cty::c_uint);
    pub fn lni2c_write(ptr: *mut ln_i2c_c, n: cty::c_uint, data: *const u8) -> bool;
    pub fn lni2c_read(ptr: *mut ln_i2c_c, n: cty::c_uint, data: *mut u8) -> bool;
    pub fn lni2c_write_to(
        ptr: *mut ln_i2c_c,
        target: cty::c_uint,
        n: cty::c_uint,
        data: *const u8,
    ) -> bool;
    pub fn lni2c_multi_write_to(
        ptr: *mut ln_i2c_c,
        target: cty::c_uint,
        nbSeqn: cty::c_uint,
        seqLength: *const cty::c_uint,
        data: *mut *const u8,
    ) -> bool;
    pub fn lni2c_read_from(
        ptr: *mut ln_i2c_c,
        target: cty::c_uint,
        n: cty::c_uint,
        data: *mut u8,
    ) -> bool;
    pub fn lni2c_begin(ptr: *mut ln_i2c_c, target: cty::c_uint) -> bool;
    pub fn lni2c_set_dma_mode(ptr: *mut ln_i2c_c, enable: bool);
}
