//! Safe Rust wrapper around the C++ `lnTimer` class.
//!
//! Provides a `Timer` struct that can be created from a pin number
//! and used to generate single‑shot pulses via `single_shot()`.

use crate::c_api::rn_timer_c::*;
use crate::gpio::lnPin;

/// A hardware timer channel, wrapping the C++ `lnTimer` class.
///
/// # Example
///
/// ```ignore
/// use esprit::timer::Timer;
///
/// let mut t = Timer::from_pin(pin_number);
/// t.single_shot(50, false); // 50 ms pulse, active high
/// // Timer is automatically freed when `t` goes out of scope
/// ```
pub struct Timer {
    inner: *mut ln_timer_c,
}

impl Timer {
    /// Create a new timer from a pin.
    ///
    /// The pin mapping is looked up in the hardware pin‑mapping table
    /// to determine which timer and channel to use.
    pub fn from_pin(pin: lnPin) -> Self {
        let inner = unsafe { ln_timer_create_from_pin(pin) };
        Timer { inner }
    }

    /// Create a new timer from a timer index and channel.
    pub fn new(timer: u32, channel: u32) -> Self {
        let inner = unsafe { ln_timer_create(timer, channel) };
        Timer { inner }
    }

    /// Generate a single‑shot pulse.
    ///
    /// * `duration_ms` – pulse duration in milliseconds (max 100 ms).
    /// * `down` – if `true`, the output goes low for the pulse duration;
    ///            if `false`, it goes high.
    pub fn single_shot(&mut self, duration_ms: u32, up: bool) {
        unsafe { ln_timer_single_shot(self.inner, duration_ms, up) }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        unsafe { ln_timer_delete(self.inner) }
    }
}
