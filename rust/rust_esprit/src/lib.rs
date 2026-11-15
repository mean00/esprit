#![allow(unused_unsafe)]
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
//! * [`Gpio`] — the MCU-agnostic pin contract, implemented by [`Pin`] itself
//!   (`set_mode`, `write`, `read`, `toggle`, `open_drain`, …).
//! * [`PinMode`] — pin mode configuration (`PinMode::Output`,
//!   `PinMode::InputPullUp`, …).
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
//! [`Uart`], [`UartDriver`], [`AdcDriver`], [`ScanAdc`], `Usb`, `Cdc`, plus the
//! event enums (`CdcEvent`, `UsbEvent`) and their handler traits.
//!
//! The peripheral *contracts* (SPI, I²C, ADC, timers, DMA, PWM, RCU, GPIO) are
//! re-exported from `rs_mcu_traits` under `hal` and as the `Hal*` names; the
//! family driver that implements one is reachable as [`SpiDriver`],
//! [`gpio_rp2xxx`], [`uart_rp2xxx`] or `rust_esprit::{bluepill, rp2xxx}`.
//!
//! (`Usb`/`Cdc` and their events are only available with the `cdc` feature.)
//!
//! ## External interrupts
//!
//! The [`Exti`] contract ([`Edge`], [`ExtiHandler`]) with the family driver as
//! [`ExtiDriver`]: `attach(pin, edge, handler)`, `enable`, `disable`, `detach`.
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
use critical_section::{Impl, RawRestoreState, set_impl};
#[cfg(all(any(feature = "bluepill", feature = "rp2xxx"), not(feature = "external_std")))]
mod libc_stub;
#[cfg(all(any(feature = "bluepill", feature = "rp2xxx"), not(feature = "external_std")))]
mod runtime;
#[cfg(all(any(feature = "bluepill", feature = "rp2xxx"), not(feature = "external_std")))]
mod printf;
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

    cfg_if::cfg_if! {
        if #[cfg(feature = "external_std")] {
            pub use std::alloc::{GlobalAlloc, Layout};
            pub use std::boxed::Box;
            pub use std::string::String;
            pub use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering};
            pub use std::vec::Vec;
        } else {
            pub use alloc::boxed::Box;
            pub use alloc::string::String;
            pub use alloc::vec::Vec;
            pub use core::alloc::{GlobalAlloc, Layout};
            pub use core::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering};
        }
    }
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
pub(crate) use c_api::rn_debug_c;
pub(crate) use c_api::rn_freertos_c;
// `rn_timer_c` stays on the C API for every target: `task.rs` uses the real C
// helpers (`lnGetUs`, `lnDelay_C`, …), which esprit provides everywhere.
pub(crate) use c_api::rn_timer_c;

cfg_if::cfg_if! {
    if #[cfg(feature = "rp2xxx")] {
        pub(crate) use c_api::rn_gpio_rp2040_c;
    } else if #[cfg(any(feature = "esp32", feature = "wrapped"))] {
        #[cfg(feature = "esp32")]
        pub(crate) use c_api::rn_gpio_esp32_c;
    } else {
        pub(crate) use c_api::rn_gpio_bp_c;
    }
}


// ---------------------------------------------------------------------------
//  Private implementation modules
//
//  The public API is re-exported flat at the crate root; these modules are
//  implementation details and must not be used directly.
// ---------------------------------------------------------------------------

mod gpio;
pub use rs_rtos::{event, queue, sync, task};
mod logger;
/// The Esprit logger's C ABI (`lnDebug.cpp`) in Rust: CMake `USE_RUST_LOGGER`.
#[cfg(feature = "rust_logger")]
mod logger_c;
#[cfg(all(
    feature = "rust_logger",
    not(any(feature = "bluepill", feature = "rp2xxx"))
))]
compile_error!(
    "the `rust_logger` feature needs either the `bluepill` or `rp2040`/`rp2350` platform backend"
);

// ---------------------------------------------------------------------------
//  Public API — flat, idiomatic re-exports at the crate root
// ---------------------------------------------------------------------------
pub use event::EventGroup;
pub use gpio::{Gpio, Pin, PinMode};
/// The ADC contracts (`Adc` single shot, `ScanAdc` scan/DMA) and their config,
/// handler and buffer types — all of them from `rs_mcu_traits::adc`.
pub use rs_mcu_traits::adc::{Adc, AdcBuffer, AdcConfig, AdcHandler, ScanAdc, ScanConfig};
/// Route pads to the SPI peripheral of the current target (the family decides
/// which instance a pad reaches, so the pin contract has no "SPI mode").
pub use gpio::set_spi_pins;
/// Pin **interrupt** contract: the external-interrupt controller arms an edge on a
/// pin number and owns the handler (`rs_mcu_traits::exti`). `Exti` is the trait;
/// the family type that implements it is [`ExtiDriver`].
pub use rs_mcu_traits::exti::{Edge, Exti, ExtiHandler};
// Platform-specific peripheral driver selections, low-level crate re-exports,
// and startup / link anchors (`platform/{rp2xxx,bluepill,wrapped}.rs`).
cfg_if::cfg_if! {
    if #[cfg(feature = "rp2xxx")] {
        #[path = "platform/rp2xxx.rs"]
        mod platform;
    } else if #[cfg(any(feature = "esp32", feature = "wrapped"))] {
        #[path = "platform/wrapped.rs"]
        mod platform;
    } else {
        #[path = "platform/bluepill.rs"]
        mod platform;
    }
}
pub use platform::*;

pub use logger::LoggerWriter;
pub use queue::Queue;
#[cfg(feature = "cdc")]
#[allow(deprecated)]
pub use rs_tinyusb::{Cdc, CdcAcm, CdcEvent, CdcEventHandler, CdcPort, LineState, UsbError};
/// The retired wrapper over the C++ `lnUsbStack` - only for the transition ABI.
#[cfg(all(feature = "cdc", feature = "c_abi"))]
pub use rs_tinyusb::{Usb, UsbBus, UsbEvent, UsbEventHandler};
pub use sync::{
    Arc, BinarySemaphore, CountingSemaphore, LazyLock, Mutex, MutexGuard, OnceLock, RecursiveMutex,
    RecursiveMutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard, SemaphoreGuard,
};
pub use task::{
    Duration, Instant, TaskEntry, TaskHandle, current, delay_ms, delay_us, sleep, sleep_ms, spawn,
    spawn_raw, tick_count, time_ms, time_us, time_us64, yield_now,
};

/// The USB device framework above tinyUSB (`usb.md` P4), re-exported so the board does
/// not have to depend on it directly.
#[cfg(feature = "usb_stack")]
pub use rs_usb;

// ---------------------------------------------------------------------------
//  Deprecated legacy aliases
//
//  Kept so existing projects keep compiling.  They are thin wrappers / type
//  aliases for the idiomatic names above and are marked `#[deprecated]`; new
//  code should use the idiomatic names.
// ---------------------------------------------------------------------------

// Legacy time functions.
#[allow(deprecated)]
pub use task::{get_time_ms, get_time_us};

// ---------------------------------------------------------------------------
//  Interrupt control
// ---------------------------------------------------------------------------
unsafe extern "C" {
    pub(crate) fn deadEnd(code: cty::c_int);
    #[cfg(feature = "external_std")]
    fn lnInterrupts();
    #[cfg(feature = "external_std")]
    fn lnNoInterrupt();
}

pub fn disable_interrupts() {
    #[cfg(not(feature = "external_std"))]
    unsafe {
        rs_rtos::c_freertos::vPortEnterCritical()
    }
    #[cfg(feature = "external_std")]
    unsafe {
        lnNoInterrupt()
    }
}
pub fn enable_interrupts() {
    #[cfg(not(feature = "external_std"))]
    unsafe {
        rs_rtos::c_freertos::vPortExitCritical()
    }
    #[cfg(feature = "external_std")]
    unsafe {
        lnInterrupts()
    }
}

// ---------------------------------------------------------------------------
//  Global allocator (FreeRTOS)
// ---------------------------------------------------------------------------
cfg_if! {
    if #[cfg(all(not(target_os = "espidf"), not(feature = "external_std")))] {
        pub struct FreeRtosAllocator;
        struct FreeRtosCriticalSection;
        critical_section::set_impl!(FreeRtosCriticalSection);

    unsafe impl critical_section::Impl for FreeRtosCriticalSection {
        unsafe fn acquire() -> critical_section::RawRestoreState {
            // Calls FreeRTOS portENTER_CRITICAL()
             rn_freertos_c::vPortEnterCritical()
        }

        unsafe fn release(_token: critical_section::RawRestoreState) {
            // Calls FreeRTOS portEXIT_CRITICAL()
             rn_freertos_c::vPortExitCritical()
        }
     }

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
            pub(crate) fn pvPortMalloc(xSize: cty::c_uint) -> *mut cty::c_void;
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

    // --- ZERO-COST CORE RE-EXPORTS ---
    pub use core::any;
    pub use core::cell;
    pub use core::clone;
    pub use core::cmp;
    pub use core::convert;
    pub use core::default;
    pub use core::fmt;
    pub use core::hash;
    pub use core::iter;
    pub use core::marker;
    pub use core::mem;
    pub use core::num;
    pub use core::ops;
    pub use core::option;
    pub use core::pin;
    pub use core::ptr;
    pub use core::result;
    pub use core::slice;
    pub use core::str;
    pub use core::task;

    // --- ZERO-COST ALLOC RE-EXPORTS ---
    pub mod vec {
        pub use alloc::vec::*;
    }
    pub mod string {
        pub use alloc::string::*;
    }
    pub mod boxed {
        pub use alloc::boxed::*;
    }
    pub mod borrow {
        pub use alloc::borrow::*;
    }
    pub mod rc {
        pub use alloc::rc::*;
    }

    pub mod collections {
        //! Heap-allocated collections (backed by FreeRTOS heap).
        pub use alloc::collections::*;
    }

    pub mod error {
        //! Error handling traits.
        pub use core::error::Error;
    }

    pub mod prelude {
        pub mod v1 {
            pub use alloc::borrow::ToOwned;
            pub use alloc::boxed::Box;
            pub use alloc::string::{String, ToString};
            pub use alloc::vec::Vec;
            pub use core::prelude::rust_2021::*;
        }
    }

    pub mod sync {
        //! FreeRTOS-backed synchronisation primitives.
        pub use crate::sync::{
            Arc, BinarySemaphore, CountingSemaphore, LazyLock, Mutex, MutexGuard, OnceLock,
            RecursiveMutex, RecursiveMutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
            SemaphoreGuard,
        };

        pub mod mpsc {
            //! FreeRTOS-backed message passing.
            use crate::queue::Queue;
            use crate::sync::Arc;

            #[derive(PartialEq, Eq, Clone, Copy, Debug)]
            pub struct RecvError;

            #[derive(PartialEq, Eq, Clone, Copy, Debug)]
            pub struct SendError<T>(pub T);

            pub struct SyncSender<T>(Arc<Queue<T>>);
            pub struct Receiver<T>(Arc<Queue<T>>);

            #[inline(always)]
            pub fn sync_channel<T>(bound: usize) -> (SyncSender<T>, Receiver<T>) {
                let q = Arc::new(Queue::new(bound as u32));
                (SyncSender(q.clone()), Receiver(q))
            }

            impl<T> SyncSender<T> {
                #[inline]
                pub fn send(&self, t: T) -> Result<(), SendError<T>> {
                    self.0.send(t).map_err(SendError)
                }
            }

            impl<T> Receiver<T> {
                #[inline]
                pub fn recv(&self) -> Result<T, RecvError> {
                    Ok(self.0.receive())
                }
            }

            impl<T> Clone for SyncSender<T> {
                fn clone(&self) -> Self {
                    SyncSender(self.0.clone())
                }
            }
        }
    }

    pub mod time {
        //! Hardware-timer-backed time types.
        pub use crate::task::{Duration, Instant};
    }

    pub mod thread {
        //! FreeRTOS-backed task spawning.
        pub use crate::task::{current, sleep, sleep_ms, spawn, yield_now};
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

pub use rs_mcu_traits as hal;
pub use rs_mcu_traits::{
    adc::{Adc as HalAdc, AdcBuffer as HalAdcBuffer, ScanAdc as HalScanAdc},
    gpio::{Gpio as HalGpio},
    exti::{Exti as HalExti},
    i2c::{I2c as HalI2c, I2cConfig, I2cHandler, Operation},
    pwm::{Pwm as HalPwm, PwmConfig, PwmHandler},
    dma::{Dma as HalDma, DmaHandler},
    spi::{BitOrder as HalSpiBitOrder, Spi as HalSpi, SpiConfig, SpiHandler, SpiMode as HalSpiMode},
    timer::{DelayHandler, DelayTimer as HalDelayTimer, Timer as HalTimer, TimerHandler},
    uart::{Uart as HalUart, UartConfig, UartHandler},
};
