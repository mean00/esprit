#![allow(dead_code)]
#![allow(unused_imports)]

#[cfg(feature = "rp2040")]
mod import_gpio {
    pub use crate::rn_fast_gpio_rp2040 as rn_fast_gpio;
    pub use crate::rn_gpio_rp2040_c;
    pub use crate::rn_gpio_rp2040_c as gpio;
}
#[cfg(feature = "esp32")]
mod import_gpio {
    pub use crate::rn_fast_gpio_esp32c3 as rn_fast_gpio;
    pub use crate::rn_gpio_esp32_c;
    pub use crate::rn_gpio_esp32_c as gpio;
}

#[cfg(not(any(feature = "rp2040", feature = "esp32")))]
mod import_gpio {
    pub use crate::rn_fast_gpio_bp as rn_fast_gpio;
    pub use crate::rn_gpio_bp_c;
    pub use crate::rn_gpio_bp_c as gpio;
}
//-----------------
//-----------------
pub use crate::rn_exti;

//pub use crate::rn_gpio_bp_c ;
//pub use crate::rn_gpio_bp_c as gpio;

//
pub use import_gpio::gpio::lnPin;
//
use crate::rn_gpio::import_gpio::gpio;
pub use import_gpio::gpio::lnGetGpioToggleRegister;
pub use import_gpio::gpio::lnGpioMode as rnGpioMode;
pub use rn_exti::rnEdge;

// reexport
pub use import_gpio::gpio::lnPin as rnPin;

/*
 */
pub fn rnpin2lnpin(pin: rnPin) -> lnPin {
    let asi32: i32 = pin as i32;
    unsafe {
        let aslnpin: lnPin = core::mem::transmute(asi32);
        aslnpin
    }
}
/*
 */

pub fn digitalWrite(pin: rnPin, value: bool) {
    unsafe {
        gpio::lnDigitalWrite(rnpin2lnpin(pin), value);
    }
}
/*
 */
pub fn digitalToggle(pin: rnPin) {
    unsafe {
        gpio::lnDigitalToggle(rnpin2lnpin(pin));
    }
}
/*
 */
pub fn digitalRead(pin: rnPin) -> bool {
    unsafe { gpio::lnDigitalRead(rnpin2lnpin(pin)) }
}

pub fn pinMode(pin: rnPin, mode: rnGpioMode) {
    unsafe {
        gpio::lnPinMode(rnpin2lnpin(pin), mode, 0);
    }
}
/*
 */

pub fn pinModeSpeed(pin: rnPin, mode: rnGpioMode, speed: i32) {
    unsafe {
        gpio::lnPinMode(rnpin2lnpin(pin), mode, speed);
    }
}
/*
 */

pub fn digital_write(pin: rnPin, value: bool) {
    unsafe {
        gpio::lnDigitalWrite(rnpin2lnpin(pin), value);
    }
}
/*
 */
pub fn digital_toggle(pin: rnPin) {
    unsafe {
        gpio::lnDigitalToggle(rnpin2lnpin(pin));
    }
}
/*
 */
pub fn digital_read(pin: rnPin) -> bool {
    unsafe { gpio::lnDigitalRead(rnpin2lnpin(pin)) }
}

/*
 */
pub fn pin_mode(pin: rnPin, mode: rnGpioMode) {
    unsafe {
        gpio::lnPinMode(rnpin2lnpin(pin), mode, 0);
    }
}

/*
 */
pub fn pin_mode_speed(pin: rnPin, mode: rnGpioMode, speed: i32) {
    unsafe {
        gpio::lnPinMode(rnpin2lnpin(pin), mode, speed);
    }
}

//--
