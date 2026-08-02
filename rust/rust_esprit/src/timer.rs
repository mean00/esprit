//! Safe Rust wrapper around the C++ `lnTimer` class.
//!
//! Provides a [`Timer`] struct that can be created from a pin and used to
//! generate single‑shot pulses via [`Timer::single_shot`].

use crate::c_api::rn_timer_c as rt;
use crate::gpio::Pin;

/// A hardware timer channel, wrapping the C++ `lnTimer` class.
///
/// # Example
///
/// ```ignore
/// use rust_esprit::Timer;
///
/// let mut t = Timer::from_pin(Pin::PB6);
/// t.single_shot(50, false); // 50 ms pulse, active high
/// // Timer is automatically freed when `t` goes out of scope
/// ```
pub struct Timer {
    inner: *mut crate::raw::ln_timer_c,
}

impl Timer {
    /// Create a new timer from a pin.
    ///
    /// The pin mapping is looked up in the hardware pin‑mapping table
    /// to determine which timer and channel to use.
    pub fn from_pin(pin: Pin) -> Self {
        let inner = unsafe { rt::ln_timer_create_from_pin(pin) };
        Timer { inner }
    }

    /// Create a new timer from a timer index and channel.
    pub fn new(timer: u32, channel: u32) -> Self {
        let inner = unsafe { rt::ln_timer_create(timer, channel) };
        Timer { inner }
    }

    /// Generate a single‑shot pulse.
    ///
    /// * `duration_ms` – pulse duration in milliseconds (max 100 ms).
    /// * `up` – pulse polarity: `true` = output goes high for the pulse
    ///   duration, `false` = it goes low.
    pub fn single_shot(&mut self, duration_ms: u32, up: bool) {
        unsafe { rt::ln_timer_single_shot(self.inner, duration_ms, up) }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        unsafe { rt::ln_timer_delete(self.inner) }
    }
}
