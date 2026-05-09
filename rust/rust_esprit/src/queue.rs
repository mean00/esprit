#![allow(dead_code)]

//! A generic FreeRTOS-backed message queue.
//!
//! # Example
//! ```ignore
//! let q = Queue::<u32, 16>::new();
//! q.send(42).ok();
//! let val = q.receive(); // blocks until an item is available
//! ```

use crate::prelude::*;
use crate::rn_freertos_c;
use core::ffi::c_void;

/// A fixed-capacity queue of items of type `T`.
///
/// The queue is backed by a FreeRTOS `QueueHandle_t`.  Items are copied
/// into the queue by value (the kernel stores a copy).
pub struct Queue<T> {
    handle: rn_freertos_c::QueueHandle_t,
    _phantom: PhantomData<T>,
}

// SAFETY: FreeRTOS queues are safe to transfer between tasks.
unsafe impl<T: Send> Send for Queue<T> {}
unsafe impl<T: Send> Sync for Queue<T> {}

// FreeRTOS queues are fixed-size at creation.  We store the capacity
// so we can report it back.
const _: () = ();

impl<T> Queue<T> {
    /// Create a new queue with the given `capacity` (in items).
    ///
    /// Panics if the underlying FreeRTOS queue cannot be created (out of heap).
    pub fn new(capacity: u32) -> Self {
        let item_size = core::mem::size_of::<T>() as u32;
        let handle = unsafe {
            rn_freertos_c::xQueueGenericCreate(capacity, item_size, 0) // 0 = standard queue
        };
        assert!(!handle.is_null(), "Queue::new: xQueueGenericCreate returned NULL");
        Self {
            handle,
            _phantom: PhantomData,
        }
    }

    // -- Sending -----------------------------------------------------------

    /// Send an item, blocking indefinitely until space is available.
    pub fn send(&self, item: T) -> Result<(), T> {
        let val_ref = &item as *const T as *const c_void;
        let ret =
            unsafe { rn_freertos_c::xQueueGenericSend(self.handle, val_ref, u32::MAX, 0) };
        if ret != 0 {
            // The item was copied into the queue – forget it so we don't drop
            // the original (the kernel now owns a copy).
            core::mem::forget(item);
            Ok(())
        } else {
            Err(item)
        }
    }

    /// Try to send without blocking.
    pub fn try_send(&self, item: T) -> Result<(), T> {
        let val_ref = &item as *const T as *const c_void;
        let ret = unsafe { rn_freertos_c::xQueueGenericSend(self.handle, val_ref, 0, 0) };
        if ret != 0 {
            core::mem::forget(item);
            Ok(())
        } else {
            Err(item)
        }
    }

    /// Send with a timeout in milliseconds.
    pub fn send_timeout(&self, item: T, timeout_ms: u32) -> Result<(), T> {
        let ticks = if timeout_ms == u32::MAX {
            u32::MAX
        } else {
            (timeout_ms as u64 * 1000 / 1000) as u32
        };
        let val_ref = &item as *const T as *const c_void;
        let ret = unsafe { rn_freertos_c::xQueueGenericSend(self.handle, val_ref, ticks, 0) };
        if ret != 0 {
            core::mem::forget(item);
            Ok(())
        } else {
            Err(item)
        }
    }

    /// Send from an ISR context.
    ///
    /// Returns `(Ok(()), true)` if sent and a higher-priority task was woken.
    /// # Safety
    /// Must only be called from an ISR.
    pub unsafe fn send_from_isr(&self, item: T) -> Result<bool, T> {
        let val_ref = &item as *const T as *const c_void;
        let mut higher_woken: rn_freertos_c::BaseType_t = 0;
        let ret = rn_freertos_c::xQueueGenericSendFromISR(
            self.handle,
            val_ref,
            &mut higher_woken,
            0,
        );
        if ret != 0 {
            core::mem::forget(item);
            Ok(higher_woken == 1)
        } else {
            Err(item)
        }
    }

    // -- Receiving ---------------------------------------------------------

    /// Receive an item, blocking until one is available.
    pub fn receive(&self) -> T {
        let mut val: core::mem::MaybeUninit<T> = core::mem::MaybeUninit::uninit();
        let ret = unsafe {
            rn_freertos_c::xQueueReceive(
                self.handle,
                val.as_mut_ptr() as *mut c_void,
                u32::MAX,
            )
        };
        assert!(ret != 0, "Queue::receive failed (timed out on infinite wait)");
        unsafe { val.assume_init() }
    }

    /// Try to receive without blocking.
    pub fn try_receive(&self) -> Option<T> {
        let mut val: core::mem::MaybeUninit<T> = core::mem::MaybeUninit::uninit();
        let ret = unsafe {
            rn_freertos_c::xQueueReceive(self.handle, val.as_mut_ptr() as *mut c_void, 0)
        };
        if ret != 0 {
            Some(unsafe { val.assume_init() })
        } else {
            None
        }
    }

    /// Receive with a timeout in milliseconds.
    pub fn receive_timeout(&self, timeout_ms: u32) -> Option<T> {
        let ticks = if timeout_ms == u32::MAX {
            u32::MAX
        } else {
            (timeout_ms as u64 * 1000 / 1000) as u32
        };
        let mut val: core::mem::MaybeUninit<T> = core::mem::MaybeUninit::uninit();
        let ret = unsafe {
            rn_freertos_c::xQueueReceive(self.handle, val.as_mut_ptr() as *mut c_void, ticks)
        };
        if ret != 0 {
            Some(unsafe { val.assume_init() })
        } else {
            None
        }
    }

    // -- Status queries ----------------------------------------------------

    /// Number of items currently in the queue.
    pub fn len(&self) -> u32 {
        unsafe { rn_freertos_c::uxQueueMessagesWaiting(self.handle) }
    }

    /// Remaining free space.
    pub fn available(&self) -> u32 {
        unsafe { rn_freertos_c::uxQueueSpacesAvailable(self.handle) }
    }

    /// Returns `true` if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Peek at the front item without removing it, blocking until available.
    pub fn peek(&self) -> T {
        let mut val: core::mem::MaybeUninit<T> = core::mem::MaybeUninit::uninit();
        let ret = unsafe {
            rn_freertos_c::xQueuePeek(self.handle, val.as_mut_ptr() as *mut c_void, u32::MAX)
        };
        assert!(ret != 0, "Queue::peek failed");
        unsafe { val.assume_init() }
    }

    /// Peek without blocking.
    pub fn try_peek(&self) -> Option<T> {
        let mut val: core::mem::MaybeUninit<T> = core::mem::MaybeUninit::uninit();
        let ret = unsafe {
            rn_freertos_c::xQueuePeek(self.handle, val.as_mut_ptr() as *mut c_void, 0)
        };
        if ret != 0 {
            Some(unsafe { val.assume_init() })
        } else {
            None
        }
    }

    /// Reset the queue to empty.
    pub fn reset(&self) {
        unsafe { rn_freertos_c::xQueueGenericReset(self.handle, 0) };
    }

    /// Return the raw handle (advanced).
    pub fn handle(&self) -> rn_freertos_c::QueueHandle_t {
        self.handle
    }
}

impl<T> Drop for Queue<T> {
    fn drop(&mut self) {
        unsafe { rn_freertos_c::vQueueDelete(self.handle) }
    }
}
