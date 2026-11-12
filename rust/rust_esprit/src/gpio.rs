#![allow(dead_code)]

//! GPIO: the pin identifier of the current target, and the MCU-agnostic
//! [`Gpio`] contract it implements.
//!
//! The contract itself lives in `rs_mcu_traits::gpio` (one file per trait, see
//! `supertraits.md`): [`Gpio`] and [`PinMode`]. This module only picks the pin
//! **type** of the target — GD32 `Pin::PA0`…`PG15`, RP2xxx/ESP32 `Pin::GPIO0`… —
//! so application code says `pin.set_mode(PinMode::Output)` on whatever chip it is
//! compiled for.
//!
//! Pin **interrupts** are the external-interrupt controller's:
//! `rust_esprit::{Exti, ExtiDriver, ExtiHandler, Edge}` (the contract is
//! `rs_mcu_traits::exti`), not a method of the pad.
//!
//! The behaviour of a pad (mode, write, read, toggle, open drain) is the
//! [`Gpio`] contract, implemented by the family driver crate; this module only
//! names the target's pin type and holds the bus-routing glue.

// ---------------------------------------------------------------------------
//  Pin identifiers
// ---------------------------------------------------------------------------

/// Canonical pin identifier for the current target.
///
/// This is `pin_types::lnPin`, the one pin type of the whole stack: GD32 uses
/// `Pin::PA0` … `Pin::PG15`, RP2040/RP2350 and ESP32 use `Pin::GPIO0` … . Its
/// behaviour (`set_mode`, `write`, `read`, …) is the [`Gpio`] contract.
///
/// # Example
/// ```ignore
/// let led = Pin::PB6;
/// ```
#[doc(alias = "lnPin")]
pub use crate::pin_types::lnPin as Pin;

/// Build a [`Pin`] from a raw numeric value (advanced).
///
/// ESP32 only: on RP2040/RP2350 [`Pin`] is `rs_gpio_rp2xxx::Pin`, a foreign type
/// that cannot gain an `impl` here — and deliberately has no infallible
/// conversion either, because the raw value may name no pad. Use its checked
/// `Pin::from_number` there instead.
#[cfg(feature = "esp32")]
impl From<u32> for Pin {
    fn from(val: u32) -> Self {
        // SAFETY: `lnPin` is a C enum; this mirrors the cast the C-facing pin
        // tables use.
        unsafe { core::mem::transmute(val) }
    }
}

// ---------------------------------------------------------------------------
//  The contract (`rs_mcu_traits::gpio`)
// ---------------------------------------------------------------------------

/// The MCU-agnostic GPIO trait, implemented by [`Pin`] for the current target.
pub use rs_mcu_traits::gpio::Gpio;

/// Electrical/functional mode of a pad (`Input`, `Output`, `Analog`, …).
pub use rs_mcu_traits::gpio::PinMode;

// Pin *interrupts* are not part of this module: they are the external-interrupt
// controller's (`rs_mcu_traits::exti`, re-exported as `rust_esprit::PinEdge` /
// `rust_esprit::Exti` / `rust_esprit::ExtiHandler`).

/// Route `pins` to the SPI peripheral of the current target.
///
/// The pin contract has no "SPI mode" on purpose (which *instance* a pad reaches
/// is a family property, §R2 of `supertraits.md`), so a driver that owns a bus
/// asks for the routing here:
///
/// * RP2040 / RP2350 — IO_BANK0 `FUNCSEL = 1` (SPI) on each pad;
/// * GD32F3 / CH32V3 — the SPI alternate function on each pad;
/// * ESP32 / wrapped — `PinMode::AlternatePushPull`.
///
/// The pads then reach the instance `rs_spi_*::Spi::new(instance, …)` talks to
/// (`GPIO0`…`GPIO3` are SPI0 on an RP2xxx, `GPIO8`…`GPIO11` SPI1, and so on).
pub fn set_spi_pins(pins: &[Pin]) {
    #[cfg(feature = "rp2040")]
    {
        for pin in pins {
            let mut pad = *pin;
            // FUNCSEL 1 is SPI on every RP2xxx pad (datasheet Table 279).
            pad.set_function(rs_gpio_rp2xxx::registers::FUNCSEL_SPI0);
        }
    }

    #[cfg(all(not(feature = "rp2040"), not(any(feature = "esp32", feature = "wrapped"))))]
    {
        for pin in pins {
            // The GD32 alternate function, with the same 50 MHz drive the C++ uses.
            rs_gpio_bluepill::set_mode(*pin, rs_gpio_bluepill::Mode::SpiMode, 50);
        }
    }

    #[cfg(any(feature = "esp32", feature = "wrapped"))]
    {
        for pin in pins {
            let mut pad = *pin;
            let _ = Gpio::set_mode(&mut pad, PinMode::AlternatePushPull);
        }
    }
}

