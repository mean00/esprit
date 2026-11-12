#![no_std]
#![allow(dead_code)]

pub mod ffi;
pub use ffi::*;

use rs_mcu_traits::handler::HandlerTable;
use rs_mcu_traits::timer;
use rs_mcu_traits::Result as HalResult;

/// One handler slot: the wrapped C ABI exposes one timer channel per handle.
static HANDLERS: HandlerTable<dyn timer::TimerHandler, 1> = HandlerTable::new();
static HANDLERS_DELAY: HandlerTable<dyn timer::DelayHandler, 1> = HandlerTable::new();

/// Adapter from the C-style delay callback to the typed handler of the trait.
unsafe extern "C" fn delay_trampoline(_cookie: *mut core::ffi::c_void) {
    // SAFETY: the handler was installed before the interrupt was enabled.
    if let Some(handler) = unsafe { HANDLERS_DELAY.isr_get(0) } {
        handler.on_elapsed();
    }
}

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

impl timer::Timer for TimerWrapped {
    /// The wrapped C timer has no frequency query: it accepts the request.
    fn start_periodic(&mut self, hz: u32) -> HalResult<u32> {
        let _ = hz;
        Ok(hz)
    }

    /// The C ABI takes milliseconds; the trait takes microseconds.
    fn start_single_shot(&mut self, duration_us: u32, active_high: bool) -> HalResult<()> {
        let ms = duration_us.div_ceil(1_000).max(1);
        unsafe {
            ln_timer_single_shot(self.raw, ms as cty::c_uint, active_high);
        }
        Ok(())
    }

    fn stop(&mut self) {}

    fn set_handler(&mut self, handler: Option<&'static dyn timer::TimerHandler>) {
        HANDLERS.set(0, handler);
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

impl timer::DelayTimer for DelayTimerWrapped {
    fn arm_us(&mut self, duration_us: u32) -> HalResult<()> {
        unsafe {
            ln_delay_timer_arm(self.raw, duration_us as cty::c_int);
        }
        Ok(())
    }

    /// The wrapped C ABI takes its own callback; the trampoline below looks the
    /// typed handler up in the table.
    fn set_handler(&mut self, handler: Option<&'static dyn timer::DelayHandler>) {
        HANDLERS_DELAY.set(0, handler);
        match handler {
            Some(_) => unsafe {
                ln_delay_timer_set_interrupt(self.raw, Some(delay_trampoline), core::ptr::null_mut());
            },
            None => unsafe {
                ln_delay_timer_set_interrupt(self.raw, None, core::ptr::null_mut());
            },
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
