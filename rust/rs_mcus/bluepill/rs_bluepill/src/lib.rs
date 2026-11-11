#![no_std]

//! Hardware memory map of the **bluepill family** (STM32F1 / GD32F30x / CH32V3).
//!
//! Three files, by rule: `peripherals_common.rs` holds every address that is the
//! same on both families, `peripherals_stm32.rs` (ARM: STM32F1, GD32F30x) and
//! `peripherals_ch32v3x.rs` hold the ones that only exist on one side. This file
//! is the **one** place the family is chosen; the drivers never see a `#[cfg]`.
//!
//! `ch32v3x` wins when both features are set: the family feature (`bluepill`) is
//! the outer selector and this one only refines it, exactly like
//! `rp2040`/`rp2350` in `rs_rp2xxx`. With no feature at all the ARM map is used,
//! which is what the family built for before the split.

mod peripherals_common;

#[cfg(feature = "ch32v3x")]
#[path = "peripherals_ch32v3x.rs"]
mod peripherals_chip;

#[cfg(not(feature = "ch32v3x"))]
#[path = "peripherals_stm32.rs"]
mod peripherals_chip;

pub use peripherals_chip::*;
pub use peripherals_common::*;

