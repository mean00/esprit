#![no_std]
#![allow(dead_code)]

pub mod ffi;
pub use ffi::*;

use rs_esprit::i2c::{I2c as I2cTrait, I2cConfig, I2cHandler};

pub struct I2cWrapped {
    raw: *mut ln_i2c_c,
}

impl I2cWrapped {
    pub fn new(instance: u32, speed_hz: u32) -> Self {
        let raw = unsafe { lni2c_create(instance as cty::c_uint, speed_hz as cty::c_uint) };
        assert!(!raw.is_null(), "lni2c_create returned NULL");
        Self { raw }
    }

    pub fn raw(&self) -> *mut ln_i2c_c {
        self.raw
    }
}

impl Drop for I2cWrapped {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe {
                lni2c_delete(self.raw);
            }
        }
    }
}

impl I2cTrait for I2cWrapped {
    fn configure(&mut self, config: &I2cConfig) -> bool {
        self.set_speed(config.speed_hz);
        self.set_address(config.own_address as u32);
        true
    }

    fn write_to(&mut self, target: u16, data: &[u8]) -> bool {
        unsafe {
            lni2c_write_to(
                self.raw,
                target as cty::c_uint,
                data.len() as cty::c_uint,
                data.as_ptr(),
            )
        }
    }

    fn read_from(&mut self, target: u16, buffer: &mut [u8]) -> bool {
        unsafe {
            lni2c_read_from(
                self.raw,
                target as cty::c_uint,
                buffer.len() as cty::c_uint,
                buffer.as_mut_ptr(),
            )
        }
    }

    fn set_handler(&mut self, _handler: Option<&'static dyn I2cHandler>) {}

    fn set_speed(&mut self, speed_hz: u32) {
        unsafe {
            lni2c_setSpeed(self.raw, speed_hz as cty::c_uint);
        }
    }

    fn set_address(&mut self, address: u32) {
        unsafe {
            lni2c_setAddress(self.raw, address as cty::c_uint);
        }
    }

    fn set_dma_mode(&mut self, enable: bool) {
        unsafe {
            lni2c_set_dma_mode(self.raw, enable);
        }
    }

    fn begin(&mut self, target: u8) -> bool {
        unsafe {
            lni2c_begin(self.raw, target as cty::c_uint)
        }
    }

    fn write(&mut self, data: &[u8]) -> bool {
        unsafe {
            lni2c_write(self.raw, data.len() as cty::c_uint, data.as_ptr())
        }
    }

    fn read(&mut self, buffer: &mut [u8]) -> bool {
        unsafe {
            lni2c_read(self.raw, buffer.len() as cty::c_uint, buffer.as_mut_ptr())
        }
    }

    fn multi_write_to(&mut self, target: u8, chunks: &[&[u8]]) -> bool {
        let nb = chunks.len();
        if nb == 0 || nb > 3 {
            return false;
        }
        let mut lengths: [u32; 3] = [0, 0, 0];
        let mut ptrs: [*const u8; 3] = [core::ptr::null(), core::ptr::null(), core::ptr::null()];
        for i in 0..nb {
            lengths[i] = chunks[i].len() as u32;
            ptrs[i] = chunks[i].as_ptr();
        }
        unsafe {
            lni2c_multi_write_to(
                self.raw,
                target as cty::c_uint,
                nb as cty::c_uint,
                lengths.as_ptr(),
                ptrs.as_mut_ptr(),
            )
        }
    }
}
