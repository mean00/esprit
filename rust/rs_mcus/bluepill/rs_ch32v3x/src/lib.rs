#![no_std]
#![allow(non_camel_case_types, non_snake_case)]

//! CH32V3x MCU-specific drivers (`drop_c_part2.md`).

pub mod boot;
pub mod cpuid;
pub mod irq;
pub use irq::deadEnd;
mod registers;
pub mod runtime;
pub mod systick;
pub use systick::ln_get_us64;
#[cfg(feature = "usb_stack")]
pub mod usbd;
pub mod vectors;
