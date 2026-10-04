//! Safe Rust wrapper around the hardware stopwatch backend.
//!
//! The backend is selected by the platform features (see
//! `esprit/rust/rp2xxx/README.md`): `rs_timer_bluepill` on GD32/CH32,
//! `rs_timer_rp2xxx` (panicking stub) on RP2040/RP2350, the C API elsewhere.

use crate::rn_hw_stopwatch_c;
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
