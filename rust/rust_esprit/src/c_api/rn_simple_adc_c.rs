use core::ffi::c_void;

unsafe extern "C" {
    pub fn ln_simple_adc_create(instance: u32, pin: u32) -> *mut c_void;
    pub fn ln_simple_adc_set_smpt(adc: *mut c_void, smpt: u32);
    pub fn ln_simple_adc_read(adc: *mut c_void) -> i32;
    pub fn ln_simple_adc_destroy(adc: *mut c_void);
}
