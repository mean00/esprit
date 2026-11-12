//! Static cells shared between a driver and the rest of the firmware.
//!
//! [`SyncCell`] is a `static` of a `Sync` type around one value: no `static mut`,
//! no `unsafe impl Sync` at every call site, and each access states the invariant
//! it relies on (rule §6). `with()` is for state that must not be seen half
//! updated — the critical section is short; never drive a bus inside it.
//!
//! This is the **task-side** cell: it masks interrupts. The cell an interrupt
//! itself touches is `rs_mcu_traits::handler::HandlerCell`, which never takes a
//! lock (an ISR arms it before enabling its own source).
//!
//! The `critical-section` implementation is installed by `rust_esprit`, which is
//! what a firmware links against; a crate built on its own only needs it to
//! *link*, not to check.

use core::cell::UnsafeCell;

/// A `static`-able cell holding one value.
pub struct SyncCell<T> {
    inner: UnsafeCell<T>,
}

// SAFETY: every access goes through `&self` methods that either take a short
// critical section (`with`, `get`, `set`) or are explicitly `unsafe` and make the
// caller state why no other reference is live (`as_ref`, `as_mut`).
unsafe impl<T> Sync for SyncCell<T> {}

impl<T> SyncCell<T> {
    pub const fn new(value: T) -> Self {
        Self {
            inner: UnsafeCell::new(value),
        }
    }

    /// Run `f` with the only live `&mut` to the value, interrupts masked.
    ///
    /// Keep the closure short: it runs with interrupts disabled, so no transfer,
    /// delay or lock may happen inside it.
    #[inline]
    pub fn with<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        critical_section::with(|_| {
            // SAFETY: the critical section is the only way in, so no other
            // reference to the value can be live.
            unsafe { f(&mut *self.inner.get()) }
        })
    }

    /// The value, when it is `Copy` (atomically as far as the caller can tell:
    /// the copy happens in a critical section).
    #[inline]
    pub fn get(&self) -> T
    where
        T: Copy,
    {
        self.with(|value| *value)
    }

    /// Overwrite the value (in a critical section).
    #[inline]
    pub fn set(&self, value: T)
    where
        T: Copy,
    {
        self.with(|slot| *slot = value)
    }

    /// Shared reference to the value.
    ///
    /// # Safety
    ///
    /// No `&mut` to the value may be live: either nobody else is touching this
    /// cell, or every writer goes through [`SyncCell::with`].
    #[inline]
    pub unsafe fn as_ref(&self) -> &T {
        // SAFETY: the caller guarantees exclusive-enough access.
        unsafe { &*self.inner.get() }
    }

    /// Mutable reference to the value.
    ///
    /// # Safety
    ///
    /// The caller must be the only one touching the cell (the usual case for the
    /// one-task C ABI tables), or must hold it across a `with()` closure.
    #[inline]
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn as_mut(&self) -> &mut T {
        // SAFETY: the caller guarantees exclusive access.
        unsafe { &mut *self.inner.get() }
    }
}
