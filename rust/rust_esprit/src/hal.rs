#![allow(dead_code)]

//! Optional `embedded-hal` trait implementations.
//!
//! Enable with `features = ["embedded-hal"]` when you need to use
//! drivers from the `embedded-hal` ecosystem.

use crate::gpio::GpioPin;
use crate::task;

// ---- GPIO pins ----

impl embedded_hal::digital::ErrorType for GpioPin {
    type Error = core::convert::Infallible;
}

impl embedded_hal::digital::InputPin for GpioPin {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(GpioPin::is_high(self))
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(GpioPin::is_low(self))
    }
}

impl embedded_hal::digital::OutputPin for GpioPin {
    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.set_high();
        Ok(())
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.set_low();
        Ok(())
    }
}

impl embedded_hal::digital::StatefulOutputPin for GpioPin {
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok(GpioPin::is_high(self))
    }

    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok(GpioPin::is_low(self))
    }
}

// ---- delay ----

/// Minimal delay provider that implements `embedded_hal::delay::DelayNs`.
pub struct Delay;

impl embedded_hal::delay::DelayNs for Delay {
    fn delay_ns(&mut self, ns: u32) {
        // Coarse: round up to nearest microsecond.
        let us = (ns + 999) / 1_000;
        task::delay_us(us);
    }

    fn delay_us(&mut self, us: u32) {
        task::delay_us(us);
    }

    fn delay_ms(&mut self, ms: u32) {
        task::delay_ms(ms);
    }
}