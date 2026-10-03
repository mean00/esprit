use core::ffi::c_void;

unsafe extern "C" {
    pub fn ln_hw_stopwatch_create(timer_index: u32) -> *mut c_void;
    pub fn ln_hw_stopwatch_setup(sw: *mut c_void);
    pub fn ln_hw_stopwatch_start(sw: *mut c_void);
    pub fn ln_hw_stopwatch_wait(sw: *mut c_void, ticks: u16);
    pub fn ln_hw_stopwatch_destroy(sw: *mut c_void);
}
