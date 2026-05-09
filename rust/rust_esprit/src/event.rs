#![allow(dead_code)]

use crate::rn_fast_event_c;

pub use rn_fast_event_c::lnfast_event_group_c;

/// A lightweight FreeRTOS‑compatible event group backed by a fast‑path
/// implementation (no dynamic allocation inside the FreeRTOS scheduler).
///
/// Created via `EventGroup::new()`.
/// The underlying C object is destroyed when `EventGroup` is dropped.
pub struct EventGroup {
    raw: *mut rn_fast_event_c::lnfast_event_group_c,
}

impl EventGroup {
    /// Create a new event group.
    pub fn new() -> Self {
        let raw = unsafe { rn_fast_event_c::lnfast_event_group_create() };
        assert!(!raw.is_null(), "lnfast_event_group_create returned NULL");
        Self { raw }
    }

    /// Returns a pointer to the raw C object.  Advanced use only.
    #[inline]
    pub fn raw(&self) -> *mut rn_fast_event_c::lnfast_event_group_c {
        self.raw
    }

    /// Take ownership of the underlying C object (prevents auto‑deletion on drop).
    /// Use this when the C side manages the lifetime.
    pub fn take_ownership(&mut self) {
        unsafe { rn_fast_event_c::lnfast_event_group_takeOwnership(self.raw); }
    }

    /// Set (signal) one or more event bits.
    pub fn set_events(&mut self, bits: u32) {
        unsafe { rn_fast_event_c::lnfast_event_group_set_events(self.raw, bits); }
    }

    /// Wait for the specified `bits` to be set.
    /// Returns the actual event bits observed.
    /// `timeout_ms` < 0 means infinite wait.
    /// `timeout_ms` = 0 means poll (don't block).
    /// `timeout_ms` > 0 means timeout in milliseconds.
    pub fn wait_events(&mut self, bits: u32, timeout_ms: i32) -> u32 {
        unsafe { rn_fast_event_c::lnfast_event_group_wait_events(self.raw, bits, timeout_ms) }
    }

    /// Read the current event bits matching `mask` without waiting (non‑blocking, non‑mutating).
    pub fn read_events(&self, mask: u32) -> u32 {
        unsafe { rn_fast_event_c::lnfast_event_group_read_events(self.raw, mask) }
    }
}

impl Default for EventGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for EventGroup {
    fn drop(&mut self) {
        unsafe { rn_fast_event_c::lnfast_event_group_delete(self.raw); }
    }
}