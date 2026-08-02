// When neither fake_std nor external_std is active → no_std bare-metal mode.
// fake_std:  esprit provides its own std-compatible shim (FreeRTOS-backed).
//            The crate itself is still no_std (runs on embedded target),
//            but it exposes a `std` namespace for user code.
// external_std: esprit is used inside an existing std-enabled framework.
#![cfg_attr(not(feature = "external_std"), no_std)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clashing_extern_declarations)]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(unused_imports)]

// In no_std mode we use the `alloc` crate.
// In fake_std mode we're also no_std (embedded target), so we need alloc too.
// In external_std mode, `std` is available.
#[cfg(not(feature = "external_std"))]
extern crate alloc;

use cfg_if::cfg_if;

// Bring prelude items into scope for the rest of the crate.
use crate::prelude::*;

// ---------------------------------------------------------------------------
//  Re-export the right prelude items depending on mode
// ---------------------------------------------------------------------------
// NOTE: c_void is intentionally NOT in the prelude because
// core::ffi::c_void and std::ffi::c_void, while the same type,
// can cause trait resolution issues with Box::from_raw casts.
// Import it explicitly as `use core::ffi::c_void;` where needed.

mod prelude {
    // Items available in every mode (they are the same types whether they
    // come from `core`, `alloc` or `std`).
    pub use core::cell::UnsafeCell;
    pub use core::convert::Infallible;
    pub use core::marker::PhantomData;
    pub use core::mem;
    pub use core::ops::{Deref, DerefMut};
    pub use core::ptr;
    pub use core::slice;
    pub use core::str;
    pub use core::time::Duration;

    // bare-metal + fake_std run on an embedded target: core + alloc.
    #[cfg(not(feature = "external_std"))]
    pub use core::alloc::{GlobalAlloc, Layout};
    #[cfg(not(feature = "external_std"))]
    pub use core::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering};
    #[cfg(not(feature = "external_std"))]
    pub use alloc::boxed::Box;
    #[cfg(not(feature = "external_std"))]
    pub use alloc::string::String;
    #[cfg(not(feature = "external_std"))]
    pub use alloc::vec::Vec;

    // external_std: a real `std` is provided by the embedding framework.
    #[cfg(feature = "external_std")]
    pub use std::alloc::{GlobalAlloc, Layout};
    #[cfg(feature = "external_std")]
    pub use std::boxed::Box;
    #[cfg(feature = "external_std")]
    pub use std::string::String;
    #[cfg(feature = "external_std")]
    pub use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering};
    #[cfg(feature = "external_std")]
    pub use std::vec::Vec;
}

pub type size_t = cty::c_uint;

// ---------------------------------------------------------------------------
//  C‑bindgen modules (raw FFI) – kept for backward compatibility
// ---------------------------------------------------------------------------
/// Canonical `lnPin` type shared across all FFI binding modules.
/// Re‑exports the platform‑specific `lnPin` from the GPIO bindings.
pub mod pin_types;

/// Raw C bindings – moved to a subfolder for clarity.
/// Re-exported below for backward compatibility.
/// This module is **internal** – not part of the public API.
pub(crate) mod c_api;

// Re-export the C‑FFI modules internally so that existing code (e.g.
// `crate::rn_freertos_c::xQueueCreateMutex(...)`) continues to work.
// These are **internal** bindings, not part of the public API.
#[cfg(feature = "cdc")]
pub(crate) use c_api::rn_cdc_c;
pub(crate) use c_api::rn_debug_c;
pub(crate) use c_api::rn_exti_c;
pub(crate) use c_api::rn_fast_event_c;
pub(crate) use c_api::rn_freertos_c;
#[cfg(not(any(feature = "rp2040", feature = "esp32")))]
pub(crate) use c_api::rn_gpio_bp_c;
#[cfg(feature = "esp32")]
pub(crate) use c_api::rn_gpio_esp32_c;
#[cfg(feature = "rp2040")]
pub(crate) use c_api::rn_gpio_rp2040_c;
pub(crate) use c_api::rn_i2c_c;
pub(crate) use c_api::rn_spi_c;
pub(crate) use c_api::rn_timer_c;
pub(crate) use c_api::rn_serial_c;
pub(crate) use c_api::rn_timing_adc_c;
pub(crate) use c_api::rn_multi_pulse_c;
#[cfg(feature = "cdc")]
pub(crate) use c_api::rn_usb_c;

// ---------------------------------------------------------------------------
//  Idiomatic Rust wrappers (public API)
// ---------------------------------------------------------------------------

/// Platform‑abstracted GPIO FFI facade + `lnPin` / `GpioMode` / `Pin`.
pub mod gpio;

/// External interrupts with trait‑based callbacks.
pub mod exti;

/// SPI bus wrapper with Drop.
pub mod spi;

/// I²C bus wrapper with Drop.
pub mod i2c;

/// Fast event group (FreeRTOS‑compatible).
pub mod event;

/// Timing‑driven multi‑channel ADC.
pub mod adc;

/// USB peripheral stack wrapper with Drop.
#[cfg(feature = "cdc")]
pub mod usb;

/// CDC‑ACM (virtual COM port) wrapper with Drop.
#[cfg(feature = "cdc")]
pub mod cdc;

/// Time, delay, and task spawning functions.
pub mod task;

/// Synchronisation primitives wrapping FreeRTOS mutexes and semaphores.
pub mod sync;

/// FreeRTOS-backed fixed-capacity message queue.
pub mod queue;

/// Timer wrapper with single‑shot pulse generation.
pub mod timer;

/// DMA-based multi-pulse generator.
pub mod multi_pulse;

/// UART serial communication (Tx‑only and bidirectional).
pub mod serial;

/// `ufmt`‑based logging macros (`logger!`, `logger_init!`).
pub mod logger;


/// Optional `embedded-hal` trait implementations.
#[cfg(feature = "embedded-hal")]
pub mod hal;

// ---------------------------------------------------------------------------
//  Convenience re‑exports — use these so you don't need to remember
//  which module owns which symbol.
// ---------------------------------------------------------------------------
pub use adc::AdcTiming;
#[cfg(feature = "cdc")]
pub use cdc::{CdcAcm, CdcEvent, CdcEventHandler};
pub use event::EventGroup;
pub use exti::{
    Edge, PinCallback, attach_interrupt, detach_interrupt, disable_interrupt, enable_interrupt,
};
pub use gpio::{GpioMode, Pin, lnPin, pin_to_lnpin};
pub use i2c::I2cBus;
pub use spi::SpiBus;

// -- Time and task types --
pub use task::{
    delay_ms, delay_us, spawn, spawn_raw, time_ms, time_us, time_us64, tick_count, current,
    yield_now, TaskHandle, TaskEntry, Instant, Duration,
};

// -- Synchronisation primitives --
pub use sync::{
    Arc,
    Mutex, MutexGuard,
    RecursiveMutex, RecursiveMutexGuard,
    RwLock, RwLockReadGuard, RwLockWriteGuard,
    OnceLock,
    LazyLock,
    BinarySemaphore, CountingSemaphore, SemaphoreGuard,
};

// -- Message queues --
pub use queue::Queue;

#[cfg(feature = "cdc")]
pub use usb::{UsbBus, UsbEvent, UsbEventHandler};


// ---------------------------------------------------------------------------
//  Legacy re‑exports (exist so that existing demo projects compile unchanged)
// ---------------------------------------------------------------------------

// `rust_esprit::pin` is how demo projects refer to the raw pin enum.
pub use gpio::lnPin as pin;

// `rust_esprit::i2c` is the struct demo projects use.
// (Module already exists as `pub mod i2c;`, no need for extra alias)

// `rust_esprit::pin_edge` (rnEdge equivalent).
pub use exti::Edge as pin_edge;

// `rust_esprit::pin_edge::LN_EDGE_BOTH`, etc.
//pub use exti::Edge as rnEdge;

// `rust_esprit::exti_attach_interrupt_typed` is the typed variant from old rn_exti.
pub use exti::attach_interrupt as exti_attach_interrupt;
pub use exti::attach_interrupt as exti_attach_interrupt_typed;
pub use exti::detach_interrupt as exti_detach_interrupt;
pub use exti::enable_interrupt as exti_enable_interrupt;

// Free functions (the demo projects call these directly)
pub use gpio::digital_read;
pub use gpio::digital_toggle;
pub use gpio::digital_write;
pub use gpio::pin_mode;
//

// Re-export the raw C-functions namespace so fn pointers work.
// Demo projects sometimes import `lnLogger` / `lnLogger_init` as macros.
// Those are exported as `#[macro_export]` from logger.rs / rn_logger.rs,
// so they are already in scope globally (no need to re-export).

// ---------------------------------------------------------------------------
//  Interrupt control
// ---------------------------------------------------------------------------
unsafe extern "C" {
    pub fn deadEnd(code: cty::c_int);
    fn lnInterrupts();
    fn lnNoInterrupt();
}

pub fn disable_interrupts() {
    unsafe { lnNoInterrupt() }
}
pub fn enable_interrupts() {
    unsafe { lnInterrupts() }
}

// ---------------------------------------------------------------------------
//  Global allocator (FreeRTOS)
// ---------------------------------------------------------------------------
cfg_if! {
    if #[cfg(all(not(target_os = "espidf"), not(feature = "external_std")))] {
        pub struct FreeRtosAllocator;

        unsafe impl GlobalAlloc for FreeRtosAllocator {
            unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
                let res = pvPortMalloc(layout.size() as cty::c_uint);
                res as *mut u8
            }
            unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
                vPortFree(ptr as *mut cty::c_void);
            }
        }

        #[global_allocator]
        static GLOBAL: FreeRtosAllocator = FreeRtosAllocator;

        unsafe extern "C" {
            pub fn pvPortMalloc(xSize: size_t) -> *mut cty::c_void;
            pub fn vPortFree(pv: *mut cty::c_void);
        }

        // Critical section helper
        struct InterruptGuard;

        pub fn critical_section<F, R>(f: F) -> R
        where
            F: FnOnce() -> R,
        {
            disable_interrupts();
            let _guard = InterruptGuard;
            f()
        }

        impl Drop for InterruptGuard {
            fn drop(&mut self) {
                enable_interrupts();
            }
        }

        // Panic handler (fake_std is also no_std on embedded target)
        #[cfg(not(feature = "external_std"))]
        #[panic_handler]
        fn panic(_info: &core::panic::PanicInfo) -> ! {
            unsafe { deadEnd(55) }
            loop {}
        }
    }
}

// ---------------------------------------------------------------------------
//  std-compatible namespace (fake_std mode)
// ---------------------------------------------------------------------------
// When `fake_std` is active, we provide a `std` module that re-exports
// esprit's FreeRTOS-backed implementations under the standard paths.
// This allows embedded crates that do `use std::sync::Mutex` to work
// on the embedded target.
//
// NOTE: We use `mod std_shim` internally and re-export as `pub use std_shim as std`
// to avoid a name conflict with the real `extern crate std`.
// ---------------------------------------------------------------------------
#[cfg(feature = "fake_std")]
pub mod std_shim {
    //! FreeRTOS-backed `std`-compatible namespace.
    //!
    //! When the `fake_std` feature is enabled, this module provides
    //! `std::sync::Mutex`, `std::time::Instant`, etc. backed by
    //! FreeRTOS primitives and the hardware timer.
    //!
    //! # Usage
    //! ```ignore
    //! // In your Cargo.toml:
    //! // rust_esprit = { features = ["fake_std"] }
    //!
    //! // In your code:
    //! use rust_esprit::std::sync::Mutex;
    //! use rust_esprit::std::time::Instant;
    //! ```

    pub mod sync {
        //! FreeRTOS-backed synchronisation primitives.
        pub use crate::sync::{
            Arc,
            Mutex, MutexGuard,
            RecursiveMutex, RecursiveMutexGuard,
            RwLock, RwLockReadGuard, RwLockWriteGuard,
            OnceLock,
            LazyLock,
            BinarySemaphore, CountingSemaphore, SemaphoreGuard,
        };
    }

    pub mod time {
        //! Hardware-timer-backed time types.
        pub use crate::task::{Duration, Instant};
    }

    pub mod thread {
        //! FreeRTOS-backed task spawning.
        pub use crate::task::{spawn, yield_now, current, sleep, sleep_ms};
    }

    pub mod collections {
        //! Heap-allocated collections (backed by FreeRTOS heap).
        //! Re-exports from `alloc`.
        pub use alloc::vec::Vec;
        pub use alloc::string::String;
        pub use alloc::boxed::Box;
    }
}

/// Re-export the std-compatible shim as `std` for crate consumers.
///
/// When `fake_std` is enabled, users can write:
/// ```ignore
/// use rust_esprit::std::sync::Mutex;
/// ```
#[cfg(feature = "fake_std")]
pub use std_shim as std;
