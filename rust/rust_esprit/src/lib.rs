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
    if #[cfg(feature = "rp2040")] {
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
#[cfg(all(feature = "rust_logger", not(feature = "bluepill")))]
compile_error!(
    "the `rust_logger` feature drives the debug UART through rs_uart_bluepill: it needs the `bluepill` platform backend"
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
/// The SPI driver of the current target — the family crate that implements
/// `rs_mcu_traits::spi::Spi` (`rs_spi_rp2xxx`, `rs_spi_bluepill`, …).
///
/// A driver that owns a bus takes this type, muxes its pads with
/// [`gpio::set_spi_pins`] and programs the port through the trait, so the same
/// code runs on every family.
#[cfg(feature = "rp2040")]
pub use rs_spi_rp2xxx::Spi as SpiDriver;
#[cfg(all(
    not(feature = "rp2040"),
    not(any(feature = "esp32", feature = "wrapped"))
))]
pub use rs_spi_bluepill::Spi as SpiDriver;
#[cfg(any(feature = "esp32", feature = "wrapped"))]
pub use rs_spi_wrapped::SpiWrapped as SpiDriver;
/// The single-shot ADC driver of the current target — the family type that
/// implements [`Adc`] (`rs_adc_rp2xxx::SimpleAdc`, `rs_adc_bluepill::SimpleAdc`,
/// `rs_adc_wrapped::SimpleAdcWrapped`).
pub use adc_driver::AdcDriver;
/// The scan (DMA / multi-sample) ADC driver of the current target — the family
/// type that implements [`ScanAdc`].
pub use adc_driver::ScanAdcDriver;
/// The external-interrupt driver of the current target — the family type that
/// implements [`Exti`] (`rs_exti_rp2xxx::Exti`, `rs_exti_bluepill::Exti`,
/// `rs_exti_wrapped::ExtiWrapped`).
pub use exti_driver::ExtiDriver;
/// Platform selection of the EXTI driver: the only place a family is named.
mod exti_driver {
    cfg_if::cfg_if! {
        if #[cfg(feature = "rp2040")] {
            /// IO_BANK0: one nibble per pad, one shared `IO_IRQ_BANK0` line.
            pub use rs_exti_rp2xxx::Exti as ExtiDriver;
        } else if #[cfg(any(feature = "esp32", feature = "wrapped"))] {
            /// The C-ABI backend.
            pub use rs_exti_wrapped::ExtiWrapped as ExtiDriver;
        } else {
            /// The GD32 / CH32 EXTI block.
            pub use rs_exti_bluepill::Exti as ExtiDriver;
        }
    }
}
/// The UART driver of the current target — the family type that implements
/// [`Uart`] (`rs_uart_rp2xxx::Uart`, `rs_uart_bluepill::Uart`,
/// `rs_uart_wrapped::Uart`).
///
/// `UartDriver::with_dma(instance, &config, rx_buffer_size)` opens the port the
/// way a pass-through bridge needs it: an interrupt-driven receive ring that
/// reports [`rs_mcu_traits::uart::UartHandler::on_rx_available`], and a
/// transmitter that [`Uart::try_write`] can fill partially.
pub use uart_driver::UartDriver;
/// Platform selection of the UART driver: the only place a family is named.
mod uart_driver {
    cfg_if::cfg_if! {
        if #[cfg(feature = "rp2040")] {
            /// PL011, `UART0`/`UART1`, receive ring + interrupts.
            pub use rs_uart_rp2xxx::Uart as UartDriver;
        } else if #[cfg(any(feature = "esp32", feature = "wrapped"))] {
            /// The C-ABI backend.
            pub use rs_uart_wrapped::Uart as UartDriver;
        } else {
            /// GD32 / CH32 USART, DMA receive ring + buffered DMA transmitter.
            pub use rs_uart_bluepill::Uart as UartDriver;
        }
    }
}
/// The I²C driver of the current target — the family type that implements
/// [`HalI2c`] (`rs_i2c_rp2xxx::I2c`, `rs_i2c_bluepill::I2c`,
/// `rs_i2c_wrapped::I2cWrapped`).
pub use i2c_driver::I2cDriver;
/// Platform selection of the I²C driver: the only place a family is named.
mod i2c_driver {
    cfg_if::cfg_if! {
        if #[cfg(feature = "rp2040")] {
            /// The RP2xxx I²C block.
            pub use rs_i2c_rp2xxx::I2c as I2cDriver;
        } else if #[cfg(any(feature = "esp32", feature = "wrapped"))] {
            /// The C-ABI backend.
            pub use rs_i2c_wrapped::I2cWrapped as I2cDriver;
        } else {
            /// The GD32 / CH32 I²C block.
            pub use rs_i2c_bluepill::I2c as I2cDriver;
        }
    }
}
/// The timer channel driver of the current target — the family type that
/// implements [`HalTimer`] (`rs_timer_rp2xxx::Timer`, `rs_timer_bluepill::Timer`,
/// `rs_timer_wrapped::TimerWrapped`).
pub use timer_driver::TimerDriver;
/// The one-shot delay driver of the current target — the family type that
/// implements [`HalDelayTimer`].
pub use timer_driver::DelayTimerDriver;
/// Platform selection of the timer drivers: the only place a family is named.
mod timer_driver {
    cfg_if::cfg_if! {
        if #[cfg(feature = "rp2040")] {
            /// The RP2xxx timer channels.
            pub use rs_timer_rp2xxx::{DelayTimer as DelayTimerDriver, Timer as TimerDriver};
        } else if #[cfg(any(feature = "esp32", feature = "wrapped"))] {
            /// The C-ABI backend.
            pub use rs_timer_wrapped::{
                DelayTimerWrapped as DelayTimerDriver, TimerWrapped as TimerDriver,
            };
        } else {
            /// The GD32 / CH32 timer channels.
            pub use rs_timer_bluepill::{DelayTimer as DelayTimerDriver, Timer as TimerDriver};
        }
    }
}
/// Platform selection of the two ADC drivers: the only place a family is named.
mod adc_driver {
    cfg_if::cfg_if! {
        if #[cfg(feature = "rp2040")] {
            /// `rs_adc_rp2xxx` (pure Rust, polled SAR).
            pub use rs_adc_rp2xxx::SimpleAdc as AdcDriver;
            /// `rs_adc_rp2xxx` (DMA + DREQ, `ScanAdc::start_async`).
            pub use rs_adc_rp2xxx::TimingAdc as ScanAdcDriver;
        } else if #[cfg(any(feature = "esp32", feature = "wrapped"))] {
            /// The C-ABI backend (no Rust driver for that family).
            pub use rs_adc_wrapped::{SimpleAdcWrapped as AdcDriver, TimingAdcWrapped as ScanAdcDriver};
        } else {
            /// `rs_adc_bluepill` (pure Rust, polled SAR).
            pub use rs_adc_bluepill::SimpleAdc as AdcDriver;
            /// `rs_adc_bluepill` (timer-triggered scan).
            pub use rs_adc_bluepill::TimingAdc as ScanAdcDriver;
        }
    }
}
pub use logger::LoggerWriter;
pub use queue::Queue;
/// The RP2040 / RP2350 pins (`rs_gpio_rp2xxx`): [`Pin`] above, on that platform,
/// *is* that crate's `Pin`, and this module is where the pin type, its mode
/// mapping and the `Gpio` implementation all live.
#[cfg(feature = "rp2040")]
pub use rs_gpio_rp2xxx as gpio_rp2xxx;
/// The RP2040 / RP2350 UART driver (`rs_uart_rp2xxx`), the `Uart` implementation
/// of that platform.
#[cfg(feature = "rp2040")]
pub use rs_uart_rp2xxx as uart_rp2xxx;
#[cfg(feature = "cdc")]
#[allow(deprecated)]
pub use rs_tinyusb::{
    Cdc, CdcAcm, CdcEvent, CdcEventHandler, CdcPort, LineState, Usb, UsbBus, UsbError, UsbEvent,
    UsbEventHandler,
};
pub use sync::{
    Arc, BinarySemaphore, CountingSemaphore, LazyLock, Mutex, MutexGuard, OnceLock, RecursiveMutex,
    RecursiveMutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard, SemaphoreGuard,
};
pub use task::{
    Duration, Instant, TaskEntry, TaskHandle, current, delay_ms, delay_us, sleep, sleep_ms, spawn,
    spawn_raw, tick_count, time_ms, time_us, time_us64, yield_now,
};

// ---------------------------------------------------------------------------
//  Hardware-specific backend re-exports
// ---------------------------------------------------------------------------
#[cfg(feature = "bluepill")]
pub mod bluepill {
    //! Low-level Bluepill (GD32F3 / CH32V3) peripheral driver crates.
    pub use rs_adc_bluepill as adc;
    pub use rs_afio_bluepill as afio;
    pub use rs_dma_bluepill as dma;
    pub use rs_exti_bluepill as exti;
    pub use rs_gpio_bluepill as gpio;
    pub use rs_i2c_bluepill as i2c;
    pub use rs_multi_pulse_bluepill as multi_pulse;
    #[cfg(feature = "bluepill48")]
    pub use rs_pinout48_bluepill as pinout;
    pub use rs_rcu_bluepill as rcu;
    pub use rs_spi_bluepill as spi;
    pub use rs_timer_bluepill as timer;
    pub use rs_uart_bluepill as uart;
}

#[cfg(feature = "bluepill")]
pub use {
    rs_adc_bluepill, rs_afio_bluepill, rs_dma_bluepill, rs_exti_bluepill, rs_gpio_bluepill,
    rs_i2c_bluepill, rs_multi_pulse_bluepill, rs_rcu_bluepill, rs_spi_bluepill, rs_timer_bluepill,
    rs_uart_bluepill,
};

#[cfg(feature = "rp2040")]
pub mod rp2xxx {
    //! Low-level RP2xxx (RP2040 / RP2350) peripheral driver crates.
    pub use rs_adc_rp2xxx as adc;
    pub use rs_dma_rp2xxx as dma;
    pub use rs_gpio_rp2xxx as gpio;
    pub use rs_i2c_rp2xxx as i2c;
    pub use rs_multi_pulse_rp2xxx as multi_pulse;
    pub use rs_rcu_rp2xxx as rcu;
    pub use rs_spi_rp2xxx as spi;
    pub use rs_timer_rp2xxx as timer;
}

#[cfg(feature = "rp2040")]
pub use {
    rs_adc_rp2xxx, rs_dma_rp2xxx, rs_gpio_rp2xxx, rs_i2c_rp2xxx, rs_multi_pulse_rp2xxx,
    rs_rcu_rp2xxx, rs_spi_rp2xxx, rs_timer_rp2xxx,
};

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
