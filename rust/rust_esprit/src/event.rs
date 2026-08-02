#![allow(dead_code)]

//! FreeRTOS-backed event group with shared ownership.
//!
//! [`EventGroup`] is now `Clone` — cloning creates a new reference to the
//! same underlying FreeRTOS event group. The kernel object is freed when
//! the last clone is dropped.
//!
//! All methods take `&self` (not `&mut self`), making it safe to use from
//! a `static` or behind an `Arc`.

use crate::prelude::*;
use crate::rn_fast_event_c;
use crate::sync::Arc;

// ── Internal ref-counted handle ──────────────────────────────────────

struct EventGroupInner {
    raw: *mut crate::raw::lnfast_event_group_c,
}

impl EventGroupInner {
    fn new() -> Self {
        let raw = unsafe { rn_fast_event_c::lnfast_event_group_create() };
        assert!(!raw.is_null(), "lnfast_event_group_create returned NULL");
        Self { raw }
    }

    fn raw(&self) -> *mut crate::raw::lnfast_event_group_c {
        self.raw
    }
}

impl Drop for EventGroupInner {
    fn drop(&mut self) {
        unsafe {
            rn_fast_event_c::lnfast_event_group_delete(self.raw);
        }
    }
}

// SAFETY: FreeRTOS event groups are usable from any task.
unsafe impl Send for EventGroupInner {}
unsafe impl Sync for EventGroupInner {}

// ── Public type ──────────────────────────────────────────────────────

/// A lightweight FreeRTOS‑compatible event group backed by a fast‑path
/// implementation (no dynamic alloc inside the FreeRTOS scheduler).
///
/// `EventGroup` is `Clone` — all clones share the same underlying kernel
/// object. The object is freed when the last clone is dropped.
///
/// # Example
///
/// ```ignore
/// use rust_esprit::EventGroup;
///
/// let eg = EventGroup::new();
/// let eg2 = eg.clone(); // same kernel object
/// eg.set_events(1);
/// assert!(eg2.wait_events(1, 0) != 0);
/// ```
pub struct EventGroup {
    inner: Arc<EventGroupInner>,
}

impl EventGroup {
    /// Create a new event group.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(EventGroupInner::new()),
        }
    }

    /// Returns a pointer to the raw C object.  Advanced use only.
    #[inline]
    /// Return the raw C handle.  Advanced use only.
    pub fn raw(&self) -> *mut crate::raw::lnfast_event_group_c {
        self.inner.raw()
    }

    /// Take ownership of the underlying C object
    /// It means the calling thread will be the one receiving the events
    pub fn take_ownership(&mut self) {
        unsafe {
            rn_fast_event_c::lnfast_event_group_takeOwnership(self.inner.raw());
        }
    }

    /// Set (signal) one or more event bits.
    ///
    /// Safe to call from ISR context (uses `xEventGroupSetBitsFromISR`
    /// under the hood).
    pub fn set_events(&self, bits: u32) {
        unsafe {
            rn_fast_event_c::lnfast_event_group_set_events(self.inner.raw(), bits);
        }
    }

    /// Wait for the specified `bits` to be set.
    ///
    /// Returns the actual event bits observed.
    ///
    /// - `timeout_ms < 0`: infinite wait.
    /// - `timeout_ms = 0`: non-blocking poll.
    /// - `timeout_ms > 0`: timeout in milliseconds.
    pub fn wait_events(&self, bits: u32, timeout_ms: i32) -> u32 {
        unsafe {
            rn_fast_event_c::lnfast_event_group_wait_events(self.inner.raw(), bits, timeout_ms)
        }
    }

    /// Read the current event bits matching `mask` without waiting
    /// (non‑blocking, non‑mutating).
    pub fn read_events(&self, mask: u32) -> u32 {
        unsafe { rn_fast_event_c::lnfast_event_group_read_events(self.inner.raw(), mask) }
    }
}

impl Default for EventGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for EventGroup {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

// SAFETY: FreeRTOS event groups are usable from any task.
unsafe impl Send for EventGroup {}
unsafe impl Sync for EventGroup {}

