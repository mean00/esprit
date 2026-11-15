#![no_std]

//! RP2040 chip-specific core support (CPUID, IRQ helpers, runtime, reset/deadEnd).

pub mod cpuid;
pub mod irq;
pub mod runtime;
pub mod sys;

pub use runtime::{ln_get_us, ln_get_us64};
pub use sys::*;
