#![no_std]
#![allow(dead_code)]

pub mod ffi;
pub use ffi::*;

use rs_esprit::timer::{Timer as TimerTrait, TimerHandler, DelayTimer as DelayTimerTrait};

pub struct TimerWrapped {
    raw: *mut ln_timer_c,
}

impl TimerWrapped {
    pub fn new(timer: u32, channel: u32) -> Self {
        let raw = unsafe { ln_timer_create(timer as cty::c_uint, channel as cty::c_uint) };
        assert!(!raw.is_null(), "ln_timer_create returned NULL");
        Self { raw }
    }

    pub fn from_pin(pin: i32) -> Self {
        let raw = unsafe { ln_timer_create_from_pin(pin as lnPin) };
        assert!(!raw.is_null(), "ln_timer_create_from_pin returned NULL");
        Self { raw }
    }

    pub fn raw(&self) -> *mut ln_timer_c {
        self.raw
    }
}

impl Drop for TimerWrapped {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe {
                ln_timer_delete(self.raw);
            }
        }
    }
}

impl TimerTrait for TimerWrapped {
    fn set_frequency(&mut self, _frequency_hz: u32) -> Result<u32, &'static str> {
        Ok(0)
    }

    fn enable(&mut self) {}

    fn disable(&mut self) {}

    fn set_handler(&mut self, _handler: Option<&'static dyn TimerHandler>) {}

    fn single_shot(&mut self, duration_ms: u32, up: bool) {
        unsafe {
            ln_timer_single_shot(self.raw, duration_ms as cty::c_uint, up);
        }
    }
}

pub struct DelayTimerWrapped {
    raw: *mut ln_delay_timer_c,
}

impl DelayTimerWrapped {
    pub fn new(timer: u32) -> Result<Self, &'static str> {
        let raw = unsafe { ln_delay_timer_create(timer as cty::c_int, 0) };
        if raw.is_null() {
            Err("Failed to create delay timer: instance unavailable")
        } else {
            Ok(Self { raw })
        }
    }

    pub fn new_with_channel(timer: u32, channel: u32) -> Result<Self, &'static str> {
        let raw = unsafe { ln_delay_timer_create(timer as cty::c_int, channel as cty::c_int) };
        if raw.is_null() {
            Err("Failed to create delay timer: instance unavailable")
        } else {
            Ok(Self { raw })
        }
    }

    pub fn raw(&self) -> *mut ln_delay_timer_c {
        self.raw
    }
}

impl Drop for DelayTimerWrapped {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe {
                ln_delay_timer_delete(self.raw);
            }
        }
    }
}

impl DelayTimerTrait for DelayTimerWrapped {
    fn arm(&mut self, duration_us: u32) {
        unsafe {
            ln_delay_timer_arm(self.raw, duration_us as cty::c_int);
        }
    }

    fn set_interrupt(
        &mut self,
        handler: Option<unsafe extern "C" fn(cookie: *mut core::ffi::c_void)>,
        cookie: *mut core::ffi::c_void,
    ) {
        unsafe {
            ln_delay_timer_set_interrupt(self.raw, handler, cookie);
        }
    }

    fn enable_interrupt(&mut self) {
        unsafe {
            ln_delay_timer_enable_interrupt(self.raw);
        }
    }

    fn disable_interrupt(&mut self) {
        unsafe {
            ln_delay_timer_disable_interrupt(self.raw);
        }
    }
}
