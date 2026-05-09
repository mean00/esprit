#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clashing_extern_declarations)]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(unused_imports)]

extern crate alloc;

use cfg_if::cfg_if;
use core::alloc::{GlobalAlloc, Layout};

pub type size_t = cty::c_uint;

// ---------------------------------------------------------------------------
//  C‑bindgen modules (raw FFI) – kept for backward compatibility
// ---------------------------------------------------------------------------
#[cfg(feature = "cdc")]
pub mod rn_cdc_c;
mod rn_debug_c;
mod rn_exti_c;
mod rn_fast_event_c;
#[cfg(not(any(feature = "rp2040", feature = "esp32")))]
pub mod rn_fast_gpio_bp;
#[cfg(feature = "esp32")]
pub mod rn_fast_gpio_esp32c3;
#[cfg(feature = "rp2040")]
pub mod rn_fast_gpio_rp2040;
pub mod rn_freertos_c;
#[cfg(not(any(feature = "rp2040", feature = "esp32")))]
pub mod rn_gpio_bp_c;
#[cfg(feature = "esp32")]
pub mod rn_gpio_esp32_c;
#[cfg(feature = "rp2040")]
pub mod rn_gpio_rp2040_c;
mod rn_i2c_c;
mod rn_spi_c;
mod rn_timer_c;
mod rn_timing_adc_c;
#[cfg(feature = "cdc")]
pub mod rn_usb_c;

// ---------------------------------------------------------------------------
//  Legacy wrapper modules (kept for backward compat)
// ---------------------------------------------------------------------------
//OBSOLETE pub mod rn_exti;
//OBSOLETE pub mod rn_fast_event_group;
//OBSOLETE pub mod rn_i2c;
pub mod rn_gpio;
// OBSOLETE pub mod rn_logger;
pub mod rn_os_helper;
//pub mod rn_spi;
//pub mod rn_timing_adc;
#[cfg(feature = "cdc")]
pub mod rn_usb;
#[cfg(feature = "cdc")]
pub mod rn_usb_cdc;

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
pub use task::{delay_ms, delay_us, spawn, time_ms, time_us, time_us64};
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
    if #[cfg(all(not(target_os = "espidf"), not(feature = "use_std")))] {
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

        // Panic handler
        #[cfg(not(feature = "use_std"))]
        #[panic_handler]
        fn panic(_info: &core::panic::PanicInfo) -> ! {
            unsafe { deadEnd(55) }
            loop {}
        }
    }
}
