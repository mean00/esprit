#![no_std]
#![allow(dead_code)]

pub mod ffi;
pub use ffi::*;

use rs_esprit::gpio::{GpioPin as GpioPinTrait, GpioInterruptHandler, GpioMode, EdgeTrigger};

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
}

impl GpioPinTrait for GpioPinWrapped {
    fn write(&mut self, value: bool) {
        unsafe {
            lnDigitalWrite(self.pin as lnPin, value);
        }
    }

    fn read(&self) -> bool {
        unsafe {
            lnDigitalRead(self.pin as lnPin)
        }
    }

    fn toggle(&mut self) {
        unsafe {
            lnDigitalToggle(self.pin as lnPin);
        }
    }

    fn set_mode(&mut self, mode: GpioMode) {
        let m = match mode {
            GpioMode::Floating => lnGpioMode::lnFLOATING,
            GpioMode::InputFloating => lnGpioMode::lnINPUT_FLOATING,
            GpioMode::InputPullUp => lnGpioMode::lnINPUT_PULLUP,
            GpioMode::InputPullDown => lnGpioMode::lnINPUT_PULLDOWN,
            GpioMode::Output => lnGpioMode::lnOUTPUT,
            GpioMode::OutputOpenDrain => lnGpioMode::lnOUTPUT_OPEN_DRAIN,
            GpioMode::AlternatePushPull => lnGpioMode::lnALTERNATE_PP,
            GpioMode::AlternateOpenDrain => lnGpioMode::lnALTERNATE_OD,
            GpioMode::Pwm => lnGpioMode::lnPWM,
            GpioMode::Adc => lnGpioMode::lnADC_MODE,
            GpioMode::Dac => lnGpioMode::lnDAC_MODE,
            GpioMode::Uart => lnGpioMode::lnUART,
            GpioMode::Spi => lnGpioMode::lnSPI_MODE,
            GpioMode::UartAlt => lnGpioMode::lnUART_Alt,
        };
        unsafe {
            lnPinMode(self.pin as lnPin, m, 0);
        }
    }

    fn set_mode_speed(&mut self, mode: GpioMode, speed_mhz: u32) {
        let m = match mode {
            GpioMode::Floating => lnGpioMode::lnFLOATING,
            GpioMode::InputFloating => lnGpioMode::lnINPUT_FLOATING,
            GpioMode::InputPullUp => lnGpioMode::lnINPUT_PULLUP,
            GpioMode::InputPullDown => lnGpioMode::lnINPUT_PULLDOWN,
            GpioMode::Output => lnGpioMode::lnOUTPUT,
            GpioMode::OutputOpenDrain => lnGpioMode::lnOUTPUT_OPEN_DRAIN,
            GpioMode::AlternatePushPull => lnGpioMode::lnALTERNATE_PP,
            GpioMode::AlternateOpenDrain => lnGpioMode::lnALTERNATE_OD,
            GpioMode::Pwm => lnGpioMode::lnPWM,
            GpioMode::Adc => lnGpioMode::lnADC_MODE,
            GpioMode::Dac => lnGpioMode::lnDAC_MODE,
            GpioMode::Uart => lnGpioMode::lnUART,
            GpioMode::Spi => lnGpioMode::lnSPI_MODE,
            GpioMode::UartAlt => lnGpioMode::lnUART_Alt,
        };
        unsafe {
            lnPinMode(self.pin as lnPin, m, speed_mhz as cty::c_uint);
        }
    }

    fn open_drain_close(&mut self, close: bool) {
        unsafe {
            lnOpenDrainClose(self.pin as lnPin, close);
        }
    }

    fn enable_interrupt(&mut self, _trigger: EdgeTrigger) -> bool {
        true
    }

    fn set_handler(&mut self, _handler: Option<&'static dyn GpioInterruptHandler>) {}
}
