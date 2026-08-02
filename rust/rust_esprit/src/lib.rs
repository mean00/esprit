//! # rust_esprit
//!
//! Rust bindings for the **Esprit** HAL — the bare‑metal framework for
//! GD32, RP2040/RP2350 and ESP32 microcontrollers (FreeRTOS‑based).
//!
//! ## Public API
//!
//! The entire public API is re‑exported **flat at the crate root**.  The
//! implementation lives in private modules, so there are only three
//! namespaces to know about:
//!
//! | Namespace              | Contents                                     |
//! |------------------------|----------------------------------------------|
//! | `rust_esprit`          | safe, idiomatic API (everything below)       |
//! | `rust_esprit::raw`     | low‑level C FFI (`#[doc(hidden)]`, advanced) |
//! | `rust_esprit::std`     | `fake_std` shim (only with the `fake_std` feature) |
//!
//! ## GPIO
//!
//! * [`Pin`] — canonical pin identifier for the current target
//!   (`Pin::PA5`, `Pin::PB6`, `Pin::GPIO10`, …).
//! * [`GpioPin`] — owned pin value with a rich method set
//!   (`set_mode`, `set_high`, `write`, `toggle`, `is_high`, …).
//! * [`GpioMode`] — pin mode configuration (`GpioMode::Output`,
//!   `GpioMode::InputPullUp`, …).
//!
//! ## Time, tasks and delay
//!
//! [`delay_ms`], [`delay_us`], [`sleep`], [`sleep_ms`], [`time_ms`],
//! [`time_us`], [`time_us64`], [`tick_count`], [`spawn`], [`spawn_raw`],
//! [`current`], [`yield_now`], [`TaskHandle`], [`TaskEntry`], [`Instant`],
//! [`Duration`].
//!
//! ## Synchronisation
//!
//! [`Mutex`], [`MutexGuard`], [`RecursiveMutex`], [`RwLock`], [`OnceLock`],
//! [`LazyLock`], [`Arc`], [`BinarySemaphore`], [`CountingSemaphore`],
//! [`Queue`], [`EventGroup`].
//!
//! ## Buses and peripherals
//!
//! [`I2c`], [`Spi`], [`Serial`], [`SerialTx`], [`Timer`], [`MultiPulse`],
//! [`AdcTiming`], [`AdcBuffer`], `Usb`, `Cdc`, plus the event enums
//! (`CdcEvent`, `UsbEvent`, [`SerialEvent`]) and their handler traits.
//!
//! (`Usb`/`Cdc` and their events are only available with the `cdc` feature.)
//!
//! ## External interrupts
//!
//! [`attach_interrupt`], [`detach_interrupt`], [`enable_interrupt`],
//! [`disable_interrupt`], [`Edge`], [`PinCallback`].
//!
//! ## Logging
//!
//! [`LoggerWriter`] plus the [`logger!`] and [`logger_init!`] macros.
//!
//! ## Feature flags
//!
//! * `rp2040` — RP2040 / RP2350 pin bindings (`Pin::GPIO10`, …).
//! * `esp32` — ESP32 pin bindings.
//! * `cdc` — USB and CDC‑ACM bindings.
//! * `embedded-hal` — `embedded-hal` v1.0 trait implementations (`Delay`).
//! * `fake_std` — FreeRTOS‑backed `std`‑compatible namespace
//!   (`rust_esprit::std`).
//! * `external_std` — run inside an existing `std` framework.
//!
//! [logger!]: macro@logger
//! [logger_init!]: macro@logger_init
//!
//! # Runtime modes
//!
//! * **bare‑metal** (default): `no_std`, FreeRTOS‑backed.
//! * **`fake_std`**: the crate itself is still `no_std`, but a `std`
//!   namespace is provided for user code (`use rust_esprit::std::sync::Mutex`).
//! * **`external_std`**: the crate is used inside an existing `std`‑enabled
//!   framework that provides the real `std`.
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

// `size_t` moved into `raw` (see the `raw` module below).

// ---------------------------------------------------------------------------
//  C‑bindgen modules (raw FFI) – kept for backward compatibility
// ---------------------------------------------------------------------------
/// Canonical `lnPin` type shared across all FFI binding modules
/// (private — re-exported via `raw` and `gpio::Pin`).
mod pin_types;

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
//  Private implementation modules
//
//  The public API is re-exported flat at the crate root; these modules are
//  implementation details and must not be used directly.
// ---------------------------------------------------------------------------

mod gpio;
mod exti;
mod spi;
mod i2c;
mod event;
mod adc;
#[cfg(feature = "cdc")]
mod usb;
#[cfg(feature = "cdc")]
mod cdc;
mod task;
mod sync;
mod queue;
mod timer;
mod multi_pulse;
mod serial;
mod logger;
#[cfg(feature = "embedded-hal")]
mod hal;

// ---------------------------------------------------------------------------
//  Raw C FFI namespace — advanced, low-level use only
// ---------------------------------------------------------------------------
#[doc(hidden)]
pub mod raw {
    //! Raw C FFI layer (hidden from the main docs).
    //!
    //! The safe API at the crate root wraps these types and functions; most
    //! users never need to touch them.  This module exists for advanced
    //! callers that need to hand raw handles to C code or use the low‑level
    //! GPIO/register functions directly.

    /// `usize`-sized unsigned integer used by the C HAL.
    pub use cty::c_uint as size_t;

    // ---- GPIO ----
    /// Canonical pin enum (the same type as the crate-root `Pin`).
    pub use crate::pin_types::lnPin;
    #[cfg(not(any(feature = "rp2040", feature = "esp32")))]
    pub use crate::rn_gpio_bp_c::{
        lnDigitalRead, lnDigitalToggle, lnDigitalWrite, lnGetGpioDirectionRegister,
        lnGetGpioOffRegister, lnGetGpioOnRegister, lnGetGpioToggleRegister,
        lnGetGpioValueRegister, lnGpioMode, lnOpenDrainClose, lnPinMode_c, lnReadPort,
    };
    #[cfg(feature = "rp2040")]
    pub use crate::rn_gpio_rp2040_c::{
        lnDigitalRead, lnDigitalToggle, lnDigitalWrite, lnGetGpioDirectionRegister,
        lnGetGpioOffRegister, lnGetGpioOnRegister, lnGetGpioToggleRegister,
        lnGetGpioValueRegister, lnGpioMode, lnOpenDrainClose, lnPinMode_c, lnReadPort,
    };
    #[cfg(feature = "esp32")]
    pub use crate::rn_gpio_esp32_c::{
        lnDigitalRead, lnDigitalToggle, lnDigitalWrite, lnGetGpioDirectionRegister,
        lnGetGpioOffRegister, lnGetGpioOnRegister, lnGetGpioToggleRegister,
        lnGetGpioValueRegister, lnGpioMode, lnOpenDrainClose, lnPinMode_c, lnReadPort,
    };

    // ---- FreeRTOS core types ----
    pub use crate::rn_freertos_c::{
        BaseType_t, QueueHandle_t, SemaphoreHandle_t, TaskHandle_t, TickType_t,
        configTICK_RATE_HZ_RUST,
    };

    // ---- Peripherals ----
    pub use crate::rn_i2c_c::ln_i2c_c;
    pub use crate::rn_spi_c::{
        lnSpiCallback, lnSPISettings, ln_spi_c, spiBitOrder, spiBitOrder_SPI_LSBFIRST,
        spiBitOrder_SPI_MSBFIRST, spiDataMode, spiDataMode_SPI_MODE0, spiDataMode_SPI_MODE1,
        spiDataMode_SPI_MODE2, spiDataMode_SPI_MODE3,
    };
    pub use crate::rn_fast_event_c::lnfast_event_group_c;
    pub use crate::rn_timer_c::ln_timer_c;
    pub use crate::rn_multi_pulse_c::ln_multi_pulse_c;
    pub use crate::rn_timing_adc_c::ln_timing_adc_c;
    pub use crate::rn_serial_c::{ln_serial_event_cb, ln_serial_rx_c, ln_serial_tx_c};
    pub use crate::rn_exti_c::lnEdge;
    #[cfg(feature = "cdc")]
    pub use crate::rn_cdc_c::lncdc_c;
    #[cfg(feature = "cdc")]
    pub use crate::rn_usb_c::{lnUsbStackEventHandler, lnusb_c};
}

// ---------------------------------------------------------------------------
//  Public API — flat, idiomatic re-exports at the crate root
// ---------------------------------------------------------------------------
pub use adc::{AdcBuffer, AdcTiming};
pub use event::EventGroup;
pub use exti::{
    Edge, PinCallback, attach_interrupt, detach_interrupt, disable_interrupt, enable_interrupt,
};
pub use gpio::{GpioMode, GpioPin, Pin};
pub use i2c::I2c;
pub use logger::LoggerWriter;
pub use multi_pulse::MultiPulse;
pub use queue::Queue;
pub use serial::{Serial, SerialEvent, SerialEventHandler, SerialTx};
pub use spi::{BitOrder, Spi, SpiMode};
pub use sync::{
    Arc, BinarySemaphore, CountingSemaphore, LazyLock, Mutex, MutexGuard, OnceLock,
    RecursiveMutex, RecursiveMutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
    SemaphoreGuard,
};
pub use task::{
    current, delay_ms, delay_us, sleep, sleep_ms, spawn, spawn_raw, tick_count, time_ms, time_us,
    time_us64, yield_now, Duration, Instant, TaskEntry, TaskHandle,
};
pub use timer::Timer;
#[cfg(feature = "cdc")]
pub use cdc::{Cdc, CdcEvent, CdcEventHandler};
#[cfg(feature = "cdc")]
pub use usb::{Usb, UsbEvent, UsbEventHandler};
#[cfg(feature = "embedded-hal")]
pub use hal::Delay;

// ---------------------------------------------------------------------------
//  Deprecated legacy aliases
//
//  Kept so existing projects keep compiling.  They are thin wrappers / type
//  aliases for the idiomatic names above and are marked `#[deprecated]`; new
//  code should use the idiomatic names.
// ---------------------------------------------------------------------------

// Legacy pin-enum name (`rust_esprit::pin::GPIO10`, `const P: pin = pin::PB6`).
#[allow(deprecated)]
pub use gpio::pin;
// Legacy GPIO free functions.
#[allow(deprecated)]
pub use gpio::{digital_read, digital_toggle, digital_write, pin_mode, pin_mode_speed};
// Legacy pin-conversion helper.
#[allow(deprecated)]
pub use gpio::pin_to_lnpin;
// Legacy external-interrupt function names.
#[allow(deprecated)]
pub use exti::{
    exti_attach_interrupt, exti_attach_interrupt_typed, exti_detach_interrupt,
    exti_enable_interrupt,
};
// Legacy edge name.
#[allow(deprecated)]
pub use exti::pin_edge;
// Legacy time functions.
#[allow(deprecated)]
pub use task::{get_time_ms, get_time_us};
// Legacy bus names.
#[allow(deprecated)]
pub use i2c::I2cBus;
#[allow(deprecated)]
pub use spi::SpiBus;
#[allow(deprecated)]
pub use serial::{SerialRxTx, SerialTxOnly};
#[cfg(feature = "cdc")]
#[allow(deprecated)]
pub use cdc::CdcAcm;
#[cfg(feature = "cdc")]
#[allow(deprecated)]
pub use usb::UsbBus;

// ---------------------------------------------------------------------------
//  Interrupt control
// ---------------------------------------------------------------------------
unsafe extern "C" {
    pub(crate) fn deadEnd(code: cty::c_int);
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
            pub(crate) fn pvPortMalloc(xSize: crate::raw::size_t) -> *mut cty::c_void;
            pub(crate) fn vPortFree(pv: *mut cty::c_void);
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
