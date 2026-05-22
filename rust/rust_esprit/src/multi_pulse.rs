//! Safe Rust wrapper for the DMA-based multi-pulse generator.
//!
//! Provides a safe, RAII-style interface to `lnMultiPulse`.
//!
//! # Example
//!
//! ```ignore
//! use esprit::multi_pulse::MultiPulse;
//!
//! let mut mp = MultiPulse::new(lnPin::PB1, 100_000).unwrap();
//! mp.fire(5, 2, 5); // 5ms pulse, 2ms gap, 5ms pulse
//! mp.wait_done();
//! // mp is dropped → automatic cleanup
//! ```

use crate::pin_types::lnPin;
use crate::c_api::rn_multi_pulse_c::{self, ln_multi_pulse_c, ln_multi_pulse_create, ln_multi_pulse_delete, ln_multi_pulse_fire, ln_multi_pulse_wait_done, ln_multi_pulse_stop};

/// Safe wrapper around a multi-pulse generator.
///
/// Manages the lifetime of the underlying C++ `lnMultiPulse` object.
/// When this struct is dropped, the generator is stopped and freed.
pub struct MultiPulse {
    inner: *mut ln_multi_pulse_c,
}

// The pointer is safe to send between threads because the underlying
// hardware is accessed through a single owner.
unsafe impl Send for MultiPulse {}

impl MultiPulse {
    /// Create a new multi-pulse generator on the given pin.
    ///
    /// `tick_fq_hz` is the timer tick frequency in Hz (e.g. 100000 = 10 µs ticks).
    /// Higher values give better timing resolution but limit the maximum total
    /// pulse duration (max ~65536 ticks).
    ///
    /// Returns `None` if allocation fails.
    pub fn new(pin: lnPin, tick_fq_hz: i32) -> Option<Self> {
        let inner = unsafe { ln_multi_pulse_create(pin, tick_fq_hz) };
        if inner.is_null() {
            None
        } else {
            Some(MultiPulse { inner })
        }
    }

    /// Fire a double pulse.
    ///
    /// * `pulse1_ms` — Duration of the first pulse in ms.
    /// * `gap_ms`    — Gap between pulses in ms.
    /// * `pulse2_ms` — Duration of the second pulse in ms.
    ///
    /// Returns immediately. The hardware runs the sequence autonomously.
    /// Call `wait_done()` to block until completion.
    pub fn fire(&self, pulse1_ms: i32, gap_ms: i32, pulse2_ms: i32) {
        unsafe {
            ln_multi_pulse_fire(self.inner, pulse1_ms, gap_ms, pulse2_ms);
        }
    }

    /// Block until the pulse sequence completes.
    pub fn wait_done(&self) {
        unsafe {
            ln_multi_pulse_wait_done(self.inner);
        }
    }

    /// Abort a running pulse sequence.
    pub fn stop(&self) {
        unsafe {
            ln_multi_pulse_stop(self.inner);
        }
    }
}

impl Drop for MultiPulse {
    fn drop(&mut self) {
        unsafe {
            ln_multi_pulse_delete(self.inner);
        }
    }
}