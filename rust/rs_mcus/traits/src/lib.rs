//! MCU-agnostic peripheral contracts for the esprit Rust stack.
//!
//! **One super trait per file**, no chip knowledge, no dependencies. A family
//! crate (`rs_mcus/bluepill/*` for GD32F3 / CH32V3, `rs_mcus/rp2xxx/*` for
//! RP2040 / RP2350, `wrapped/*` for the C-ABI targets) implements these traits
//! for its own drivers; `rust_esprit` is the crate application code (swindle)
//! talks to and re-exports them under the names swindle uses.
//!
//! The rules a trait in this crate obeys (design doc:
//! `supertraits.md` at the repository root):
//!
//! * **No MCU in the contract.** Per-family identity is an associated type
//!   ([`rcu::Rcu::Peripheral`], [`dma::Dma`]'s channel) or a plain number
//!   ([`gpio::Gpio::pin_number`], [`exti::Exti`]'s pin). No pin
//!   variants, no register bases, no `stm32|gd32|ch32|rp2` anywhere.
//! * **`Result`, not C.** Failures are [`Error`]; no `bool`/`i32` status codes
//!   and no out-parameters.
//! * **Typed handlers.** `Option<&'static dyn XHandler>`, one per peripheral
//!   *instance* (see [`handler::HandlerTable`]), never a raw cookie.
//! * **No `unsafe` in a signature.** The family crate owns the `unsafe`.
//!
//! Adding a peripheral: one file, one trait (`uart.rs` → `trait Uart`), its
//! config types and its handler trait. Adding a family: one `impl` block per
//! trait in that family's crates, plus one `HandlerTable` per instance count.

#![no_std]

pub mod error;
pub mod handler;
pub mod mmio;

pub mod adc;
pub mod dma;
pub mod exti;
pub mod gpio;
pub mod i2c;
pub mod pwm;
pub mod rcu;
pub mod spi;
pub mod timer;
pub mod uart;

pub use error::{Error, Result};
pub use handler::SyncCell;
