use crate::c_api::rn_simple_adc_c;
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
