#![no_std]
#![allow(dead_code)]

pub mod ffi;
pub use ffi::*;

use rs_mcu_traits::gpio::{self, PinMode};
use rs_mcu_traits::Result;

pub struct GpioPinWrapped {
    pin: i32,
}

impl GpioPinWrapped {
    pub fn new(pin: i32) -> Self {
        Self { pin }
    }

    pub fn pin(&self) -> i32 {
        self.pin
    }

    /// [`PinMode`] → the C `lnGpioMode` of the wrapped target.
    fn raw_mode(mode: PinMode) -> lnGpioMode {
        match mode {
            PinMode::Input => lnGpioMode::lnINPUT_FLOATING,
            PinMode::InputPullUp => lnGpioMode::lnINPUT_PULLUP,
            PinMode::InputPullDown => lnGpioMode::lnINPUT_PULLDOWN,
            PinMode::Output => lnGpioMode::lnOUTPUT,
            PinMode::OutputOpenDrain => lnGpioMode::lnOUTPUT_OPEN_DRAIN,
            PinMode::AlternatePushPull => lnGpioMode::lnALTERNATE_PP,
            PinMode::AlternateOpenDrain => lnGpioMode::lnALTERNATE_OD,
            PinMode::Analog => lnGpioMode::lnADC_MODE,
        }
    }
}

impl gpio::Gpio for GpioPinWrapped {
    fn pin_number(&self) -> u32 {
        self.pin as u32
    }

    fn write(&mut self, high: bool) {
        unsafe {
            lnDigitalWrite(self.pin as lnPin, high);
        }
    }

    fn read(&self) -> bool {
        unsafe { lnDigitalRead(self.pin as lnPin) }
    }

    fn toggle(&mut self) {
        unsafe {
            lnDigitalToggle(self.pin as lnPin);
        }
    }

    fn set_mode(&mut self, mode: PinMode) -> Result<()> {
        unsafe {
            lnPinMode(self.pin as lnPin, Self::raw_mode(mode), 0);
        }
        Ok(())
    }

    fn set_mode_speed(&mut self, mode: PinMode, speed_mhz: u32) -> Result<()> {
        unsafe {
            lnPinMode(self.pin as lnPin, Self::raw_mode(mode), speed_mhz as cty::c_uint);
        }
        Ok(())
    }

    fn open_drain(&mut self, drive_low: bool) -> Result<()> {
        unsafe {
            lnOpenDrainClose(self.pin as lnPin, drive_low);
        }
        Ok(())
    }
}
