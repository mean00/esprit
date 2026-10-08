//! Safe Rust wrapper around the C++ `lnTimer` class.
//!
//! Provides a [`Timer`] struct that can be created from a pin and used to
//! generate single‑shot pulses via [`Timer::single_shot`].

#[cfg(all(not(any(feature = "rp2040", feature = "esp32", feature = "wrapped")), feature = "bluepill"))]
use rs_timer_bluepill as rt;
#[cfg(feature = "rp2040")]
use rs_timer_rp2xxx as rt;
#[cfg(any(feature = "esp32", feature = "wrapped"))]
use rs_timer_wrapped as rt;

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
    inner: *mut rt::ln_timer_c,
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

/// A hardware microsecond delay timer with update interrupt callback.
///
/// Wraps the platform `DelayTimer` driver / C++ `lnDelayTimer`.
///
/// # Example
///
/// ```ignore
/// use rust_esprit::DelayTimer;
///
/// let mut dt = DelayTimer::new(1).expect("Timer 1 available");
/// dt.set_interrupt(Some(my_callback), core::ptr::null_mut());
/// dt.arm(500); // 500 µs
/// ```
pub struct DelayTimer {
    inner: *mut rt::ln_delay_timer_c,
}

unsafe impl Send for DelayTimer {}
unsafe impl Sync for DelayTimer {}

impl DelayTimer {
    /// Create a new DelayTimer instance on a hardware timer index (1..=4).
    pub fn new(timer: u32) -> Result<Self, &'static str> {
        let inner = unsafe { rt::ln_delay_timer_create(timer as i32, 0) };
        if inner.is_null() {
            Err("Failed to create delay timer: instance unavailable or invalid index")
        } else {
            Ok(DelayTimer { inner })
        }
    }

    /// Create a new DelayTimer instance on a hardware timer index and channel (for API parity).
    pub fn new_with_channel(timer: u32, channel: u32) -> Result<Self, &'static str> {
        let inner = unsafe { rt::ln_delay_timer_create(timer as i32, channel as i32) };
        if inner.is_null() {
            Err("Failed to create delay timer: instance unavailable or invalid index")
        } else {
            Ok(DelayTimer { inner })
        }
    }

    /// Arm the timer to expire after `duration_us` microseconds and trigger the interrupt.
    pub fn arm(&mut self, duration_us: u32) {
        unsafe {
            rt::ln_delay_timer_arm(self.inner, duration_us as i32);
        }
    }

    /// Set the interrupt callback and cookie pointer.
    pub fn set_interrupt(
        &mut self,
        handler: Option<unsafe extern "C" fn(cookie: *mut core::ffi::c_void)>,
        cookie: *mut core::ffi::c_void,
    ) {
        unsafe {
            rt::ln_delay_timer_set_interrupt(self.inner, handler, cookie);
        }
    }

    /// Enable the hardware timer update interrupt in NVIC/PFIC and DIEN.
    pub fn enable_interrupt(&mut self) {
        unsafe {
            rt::ln_delay_timer_enable_interrupt(self.inner);
        }
    }

    /// Disable the hardware timer update interrupt.
    pub fn disable_interrupt(&mut self) {
        unsafe {
            rt::ln_delay_timer_disable_interrupt(self.inner);
        }
    }
}

impl Drop for DelayTimer {
    fn drop(&mut self) {
        unsafe {
            rt::ln_delay_timer_delete(self.inner);
        }
    }
}
