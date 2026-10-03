use crate::c_api::rn_hw_stopwatch_c;
use core::ffi::c_void;

pub struct HardwareStopwatch {
    handle: *mut c_void,
}

impl HardwareStopwatch {
    pub fn new(timer_index: u32) -> Self {
        let handle = unsafe { rn_hw_stopwatch_c::ln_hw_stopwatch_create(timer_index) };
        assert!(!handle.is_null());
        Self { handle }
    }
    
    pub fn setup(&self) {
        unsafe { rn_hw_stopwatch_c::ln_hw_stopwatch_setup(self.handle) }
    }
    
    pub fn start(&self) {
        unsafe { rn_hw_stopwatch_c::ln_hw_stopwatch_start(self.handle) }
    }
    
    pub fn wait(&self, ticks: u16) {
        unsafe { rn_hw_stopwatch_c::ln_hw_stopwatch_wait(self.handle, ticks) }
    }
}

impl Drop for HardwareStopwatch {
    fn drop(&mut self) {
        unsafe { rn_hw_stopwatch_c::ln_hw_stopwatch_destroy(self.handle) }
    }
}
