//! GPIO: one pin at a time.
//!
//! A *pin* is the family's own type (GD32 `PA0`…`PG15`, RP2xxx `GPIO0`…`GPIO29`).
//! The contract only needs its number, so a board can talk about a pin as a
//! plain `u32` ([`Gpio::pin_number`]) while the driver keeps its typed handle.
//!
//! Pin **interrupts are not here**: they belong to the external-interrupt
//! controller, [`crate::exti::Exti`], which arms an edge on a pin number and owns
//! the handler. This trait is only the pad: mode, level, direction.

use crate::error::Result;

/// Electrical/functional mode of a pad.
///
/// The families that have no analog pad, or no open-drain, answer
/// [`Error::Unsupported`](crate::Error::Unsupported) from
/// [`Gpio::set_mode`] for the modes they cannot reach.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PinMode {
    /// Digital input, no pull.
    Input,
    /// Digital input with the internal pull-up on.
    InputPullUp,
    /// Digital input with the internal pull-down on.
    InputPullDown,
    /// Push-pull output.
    Output,
    /// Open-drain output (the driver drives low or releases the line).
    OutputOpenDrain,
    /// Peripheral function, push-pull (UART/SPI/PWM…).
    AlternatePushPull,
    /// Peripheral function, open-drain.
    AlternateOpenDrain,
    /// Analog, driven by the ADC/DAC: digital functions off, output disabled.
    Analog,
}

/// A GPIO pad.
///
/// Implemented by each family's pin type, so `rust_esprit::Pin` *is* the trait
/// implementation for the current target.
pub trait Gpio {
    /// The pin's number, the identity used by the family's own extended API
    /// (`Pin::from_number` on RP2xxx, port/bit on GD32) and by
    /// [`crate::exti::Exti`].
    fn pin_number(&self) -> u32;

    /// Set the pad's mode with the family's default drive strength.
    fn set_mode(&mut self, mode: PinMode) -> Result<()>;

    /// Set the mode and the output slew/drive speed (ignored where the pad has none).
    fn set_mode_speed(&mut self, mode: PinMode, speed_mhz: u32) -> Result<()>;

    /// Drive the pad: `true` = high, `false` = low.
    fn write(&mut self, high: bool);

    /// Read the pad level.
    fn read(&self) -> bool;

    /// Invert the output latch.
    fn toggle(&mut self);

    /// Emulated open drain: `true` pulls the line low (driver on), `false`
    /// releases it so an external pull-up wins.
    fn open_drain(&mut self, drive_low: bool) -> Result<()>;

    /// `true` when the pad reads high.
    fn is_high(&self) -> bool {
        self.read()
    }

    /// `true` when the pad reads low.
    fn is_low(&self) -> bool {
        !self.read()
    }
}
