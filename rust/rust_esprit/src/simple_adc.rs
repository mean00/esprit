//! Safe Rust wrapper around the simple (single-shot) ADC backend.
//!
//! The backend is selected by the platform features (see
//! `esprit/rust/rp2xxx/README.md`): `rs_adc_bluepill` on GD32/CH32,
//! `rs_adc_rp2xxx` on RP2040/RP2350 (which forwards to the C++ `lnSimpleADC`
//! driver through `c_interface/ln_rp_simple_adc_c.cpp`), the C API elsewhere.

use crate::rn_simple_adc_c;
use core::ffi::c_void;

pub struct SimpleAdc {
    handle: *mut c_void,
}

impl SimpleAdc {
    pub fn new(instance: u32, pin: u32) -> Self {
        let handle = unsafe { rn_simple_adc_c::ln_simple_adc_create(instance, pin) };
        assert!(!handle.is_null());
        Self { handle }
    }
    
    pub fn set_smpt(&self, smpt: u32) {
        unsafe { rn_simple_adc_c::ln_simple_adc_set_smpt(self.handle, smpt) }
    }
    
    pub fn read(&self) -> i32 {
        unsafe { rn_simple_adc_c::ln_simple_adc_read(self.handle) }
    }
}

impl Drop for SimpleAdc {
    fn drop(&mut self) {
        unsafe { rn_simple_adc_c::ln_simple_adc_destroy(self.handle) }
    }
}
