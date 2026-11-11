#![no_std]

//! RP2xxx peripheral base addresses.
//!
//! Provides memory-mapped peripheral base addresses for the Raspberry Pi RP2xxx
//! microcontroller family with 3 flavors:
//! - `common`: addresses shared identically between RP2040 and RP2350
//! - `rp2040`: full address map for RP2040
//! - `rp2350`: full address map for RP2350
//!
//! When feature `rp2040` or `rp2350` is active, their definitions are also re-exported
//! at the root level. When neither is active, `common` is re-exported at the root.

pub mod common;
pub mod rp2040;
pub mod rp2350;

#[cfg(feature = "rp2040")]
pub use rp2040::*;

#[cfg(feature = "rp2350")]
pub use rp2350::*;

#[cfg(not(any(feature = "rp2040", feature = "rp2350")))]
pub use common::*;
