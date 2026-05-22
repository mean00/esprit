//! High-level Rust wrapper around the C++ lnTimingAdc.
//!
//! Provides [`AdcTiming`] which wraps the opaque C++ handle and exposes
//! both synchronous multi-read and asynchronous (callback-based) read.
//!
//! The async callback uses a single-entry global slot (only one DMA
//! transfer can be in-flight at a time). The context is passed as a
//! `&mut T` parameter to [`async_read`](AdcTiming::async_read), and
//! the buffer is obtained via the [`AdcBuffer`] trait.

use crate::lnPin;
use crate::rn_timing_adc_c;
use core::cell::UnsafeCell;
use core::ffi::c_void;
use core::marker::PhantomData;

/// Trait for types that can provide an ADC sample buffer.
///
/// Implemented by the callback context type so that [`async_read`](AdcTiming::async_read)
/// can obtain the buffer pointer without the caller needing to pass it
/// separately — this avoids double-mutable-borrow issues.
pub trait AdcBuffer {
    /// Return a mutable slice of at least `nb` u16 values for DMA to write into.
    fn adc_buffer(&mut self, nb: usize) -> &mut [u16];
}

/// Wraps a C++ `lnTimingAdc` instance.
///
/// `P` is a pin type marker (e.g. `PA1`) used for type-safe construction.
pub struct AdcTiming<P> {
    handle: UnsafeCell<*mut rn_timing_adc_c::ln_timing_adc_c>,
    _pin: PhantomData<P>,
}

// Safety: the C++ object is accessed from one thread at a time.
unsafe impl<P> Send for AdcTiming<P> {}
unsafe impl<P> Sync for AdcTiming<P> {}

// ---------------------------------------------------------------------------
//  Single-entry async callback slot
//
//  Only one DMA transfer can be in-flight at a time, so a single global
//  slot is sufficient. The slot stores:
//    - trampoline: the per-type trampoline (extern "C" fn that casts void* -> T)
//    - user_cb:    the user's typed callback fn(&mut T) (type-erased)
// ---------------------------------------------------------------------------

struct AdcCallbackSlot {
    trampoline: Option<unsafe extern "C" fn(*mut c_void)>,
    user_cb: Option<unsafe extern "C" fn(*mut c_void)>,
}

impl AdcCallbackSlot {
    const fn empty() -> Self {
        AdcCallbackSlot {
            trampoline: None,
            user_cb: None,
        }
    }
}

static mut ADC_CALLBACK_SLOT: AdcCallbackSlot = AdcCallbackSlot::empty();

/// Non-generic trampoline called by the C DMA ISR.
///
/// Dispatches to the per-type trampoline stored in the slot.
unsafe extern "C" fn dma_isr_entry(ctx: *mut c_void) {
    let slot = &raw mut ADC_CALLBACK_SLOT;
    if let Some(tramp) = (*slot).trampoline.take() {
        tramp(ctx);
    }
}

impl<P> AdcTiming<P> {
    /// Create a new ADC timing instance for the given ADC peripheral instance.
    pub fn new(instance: i32) -> Self {
        let handle = unsafe { rn_timing_adc_c::ln_timing_adc_create(instance) };
        assert!(!handle.is_null(), "ln_timing_adc_create failed");
        Self {
            handle: UnsafeCell::new(handle),
            _pin: PhantomData,
        }
    }

    /// Configure the ADC source: timer, channel, frequency, and pins.
    pub fn set_source(&mut self, timer: u32, channel: u32, fq: u32, pins: &[lnPin]) -> bool {
        unsafe {
            rn_timing_adc_c::ln_timing_adc_set_source(
                *self.handle.get(),
                timer,
                channel,
                fq,
                pins.len() as u32,
                pins.as_ptr(),
            )
        }
    }

    /// Synchronous multi-sample read.
    pub fn multi_read(&mut self, nb_sample_per_channel: u32, output: &mut [u16]) -> bool {
        unsafe {
            rn_timing_adc_c::ln_timing_adc_multi_read(
                *self.handle.get(),
                nb_sample_per_channel,
                output.as_mut_ptr(),
            )
        }
    }

    /// Asynchronous multi-sample read.
    ///
    /// Starts a DMA transfer. When complete, `callback` is invoked from the
    /// DMA ISR with `&mut context`.
    ///
    /// The buffer is obtained from `context` via the [`AdcBuffer`] trait,
    /// so the caller only needs to pass `&mut context` once — no separate
    /// buffer argument. This enables clean field-level borrow splitting:
    /// `self.adc.async_read(nb, &mut self.ctx, callback)` works because
    /// `&self.adc` and `&mut self.ctx` are disjoint fields.
    ///
    /// Only one DMA transfer can be in-flight at a time.
    pub fn async_read<T: AdcBuffer>(
        &self,
        nb_sample_per_channel: u32,
        context: &mut T,
        callback: fn(&mut T),
    ) -> bool {
        let buf = context.adc_buffer(nb_sample_per_channel as usize);
        let buf_ptr = buf.as_mut_ptr();
        let _buf_len = buf.len() as u32;

        // Per-type trampoline: casts ctx back to &mut T, reads the user
        // callback from the slot, and calls it.
        unsafe extern "C" fn typed_trampoline<T>(ctx: *mut c_void) {
            let this = unsafe { &mut *(ctx as *mut T) };
            let slot = &raw mut ADC_CALLBACK_SLOT;
            if let Some(cb) = (*slot).user_cb {
                let typed: fn(&mut T) = core::mem::transmute(cb);
                typed(this);
            }
        }

        unsafe {
            ADC_CALLBACK_SLOT = AdcCallbackSlot {
                trampoline: Some(typed_trampoline::<T> as unsafe extern "C" fn(*mut c_void)),
                user_cb: Some(core::mem::transmute::<
                    fn(&mut T),
                    unsafe extern "C" fn(*mut c_void),
                >(callback)),
            };
        }

        unsafe {
            rn_timing_adc_c::ln_timing_adc_async_read(
                *self.handle.get(),
                nb_sample_per_channel,
                buf_ptr,
                Some(dma_isr_entry),
                context as *mut T as *mut c_void,
            )
        }
    }
}

impl<P> Drop for AdcTiming<P> {
    fn drop(&mut self) {
        unsafe {
            rn_timing_adc_c::ln_timing_adc_delete(*self.handle.get());
        }
    }
}

