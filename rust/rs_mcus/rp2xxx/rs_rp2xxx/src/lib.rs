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
//!
//! `rp2350` wins if both are set: `rust_esprit`'s `rp2350` feature implies its
//! `rp2040` one (that is the "RP2xxx platform" selector), so a chip-rp2350 build
//! reaches this crate with both flags. Every consumer of the root re-export (the
//! drivers' `registers.rs`) resolves the pair the same way.

pub mod common;
pub mod rp2040;
pub mod rp2350;

#[cfg(feature = "rp2350")]
pub use rp2350::*;

#[cfg(all(feature = "rp2040", not(feature = "rp2350")))]
pub use rp2040::*;

#[cfg(not(any(feature = "rp2040", feature = "rp2350")))]
pub use common::*;

// One guard for the whole family: every driver reads its bases from this root, so
// a build that forgot to pick a chip fails here, with this message, instead of
// reporting a missing `IO_BANK0_BASE` in each of them.
#[cfg(not(any(feature = "rp2040", feature = "rp2350")))]
compile_error!("rs_rp2xxx: pick the chip layout with the `rp2040` or `rp2350` feature");
