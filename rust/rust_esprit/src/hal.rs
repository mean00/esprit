#![allow(dead_code)]

//! Optional `embedded-hal` trait implementations.
//!
//! Enable with `features = ["embedded-hal"]` when you need to use
//! drivers from the `embedded-hal` ecosystem.

use crate::gpio::{lnPin, Pin};
use crate::task;

// ---- GPIO pins ----

impl embedded_hal::digital::ErrorType for Pin {
    type Error = core::convert::Infallible;
}

impl embedded_hal::digital::InputPin for Pin {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(self.is_high())
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(self.is_low())
    }
}

impl embedded_hal::digital::OutputPin for Pin {
    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.set_high();
        Ok(())
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.set_low();
        Ok(())
    }
}

impl embedded_hal::digital::StatefulOutputPin for Pin {
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok(self.is_high())
    }

    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok(self.is_low())
    }
}

impl embedded_hal::digital::ToggleableOutputPin for Pin {
    fn toggle(&mut self) -> Result<(), Self::Error> {
        self.toggle();
        Ok(())
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