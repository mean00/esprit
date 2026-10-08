#![no_std]
#![allow(dead_code)]

pub mod ffi;
pub use ffi::*;

use rs_esprit::adc::{SimpleAdc as SimpleAdcTrait, TimingAdc as TimingAdcTrait, AdcHandler};

pub struct SimpleAdcWrapped {
    raw: *mut cty::c_void,
}

impl SimpleAdcWrapped {
    pub fn new(instance: u32, pin: u32) -> Self {
        let raw = unsafe { ln_simple_adc_create(instance, pin) };
        assert!(!raw.is_null(), "ln_simple_adc_create returned NULL");
        Self { raw }
    }

    pub fn set_smpt(&mut self, smpt: u32) {
        unsafe {
            ln_simple_adc_set_smpt(self.raw, smpt);
        }
    }
}

impl Drop for SimpleAdcWrapped {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe {
                ln_simple_adc_destroy(self.raw);
            }
        }
    }
}

impl SimpleAdcTrait for SimpleAdcWrapped {
    fn read(&self) -> i32 {
        unsafe {
            ln_simple_adc_read(self.raw)
        }
    }

    fn set_handler(&mut self, _handler: Option<&'static dyn AdcHandler>) {}
}

pub struct TimingAdcWrapped {
    raw: *mut ln_timing_adc_c,
}

impl TimingAdcWrapped {
    pub fn new(instance: i32) -> Self {
        let raw = unsafe { ln_timing_adc_create(instance as cty::c_int) };
        assert!(!raw.is_null(), "ln_timing_adc_create returned NULL");
        Self { raw }
    }

    pub fn raw(&self) -> *mut ln_timing_adc_c {
        self.raw
    }
}

impl Drop for TimingAdcWrapped {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe {
                ln_timing_adc_delete(self.raw);
            }
        }
    }
}

impl TimingAdcTrait for TimingAdcWrapped {
    fn set_source(&mut self, timer: u32, channel: u32, fq: u32, pins: &[u32]) -> bool {
        let pin_ints: [lnPin; 16] = [0; 16];
        let mut pins_buf = pin_ints;
        let count = pins.len().min(16);
        for i in 0..count {
            pins_buf[i] = pins[i] as lnPin;
        }
        unsafe {
            ln_timing_adc_set_source(
                self.raw,
                timer as cty::c_uint,
                channel as cty::c_uint,
                fq as cty::c_uint,
                count as cty::c_uint,
                pins_buf.as_ptr(),
            )
        }
    }

    fn multi_read(&mut self, nb_sample_per_channel: u32, output: &mut [u16]) -> bool {
        unsafe {
            ln_timing_adc_multi_read(
                self.raw,
                nb_sample_per_channel as cty::c_uint,
                output.as_mut_ptr(),
            )
        }
    }

    fn set_handler(&mut self, _handler: Option<&'static dyn AdcHandler>) {}
}
