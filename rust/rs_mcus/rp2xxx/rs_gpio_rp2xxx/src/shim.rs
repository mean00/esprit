//! C++ ABI shims for RP2xxx GPIO (`lnPinMode`, `lnDigitalWrite`, ...).

use crate::registers::*;
use crate::{
    PAD_OUTPUT, Pin, apply_mode, drive, flip, open_drain_close, sense, write_ctrl, write_pad,
};
use rs_mcu_traits::gpio::PinMode;

/// C++ `lnPinModePIO(const lnPin pin, const int instance, const bool pullup)`
#[unsafe(export_name = "_Z12lnPinModePIO5lnPinib")]
pub extern "C" fn ln_pin_mode_pio(pin: i32, instance: i32, pullup: bool) {
    let Some(p) = Pin::from_number(pin as u32) else {
        return;
    };
    let funcsel = if instance == 0 {
        FUNCSEL_PIO0
    } else {
        FUNCSEL_PIO1
    };
    let mut pad = PAD_OUTPUT;
    if pullup {
        pad |= PAD_PUE;
    }
    write_ctrl(p.number(), funcsel, GPIO_CTRL_OE_NORMAL);
    write_pad(p.number(), pad);
}

/// C++ `lnPinMode(const lnPin pin, const lnGpioMode mode, const uint32_t speedInMhz)`
#[unsafe(export_name = "_Z9lnPinMode5lnPin10lnGpioModej")]
pub extern "C" fn ln_pin_mode(pin: i32, mode: u32, _speed_mhz: u32) {
    let Some(p) = Pin::from_number(pin as u32) else {
        return;
    };
    let n = p.number();
    match mode {
        0 | 1 => apply_mode(n, PinMode::Input),          // lnFLOATING / lnINPUT_FLOATING
        2 => apply_mode(n, PinMode::InputPullUp),        // lnINPUT_PULLUP
        3 => apply_mode(n, PinMode::InputPullDown),      // lnINPUT_PULLDOWN
        4 => apply_mode(n, PinMode::Output),             // lnOUTPUT
        5 => apply_mode(n, PinMode::OutputOpenDrain),    // lnOUTPUT_OPEN_DRAIN
        6 => apply_mode(n, PinMode::AlternatePushPull),  // lnALTERNATE_PP
        7 => apply_mode(n, PinMode::AlternateOpenDrain), // lnALTERNATE_OD
        8 => {
            write_pad(n, PAD_OUTPUT);
            write_ctrl(n, FUNCSEL_PWM0, GPIO_CTRL_OE_NORMAL);
        } // lnPWM
        9 | 10 => apply_mode(n, PinMode::Analog),        // lnADC_MODE / lnDAC_MODE
        11 => {
            write_pad(n, PAD_OUTPUT);
            write_ctrl(n, FUNCSEL_UART0, GPIO_CTRL_OE_NORMAL);
        } // lnUART
        12 => {
            write_pad(n, PAD_OUTPUT);
            write_ctrl(n, FUNCSEL_SPI0, GPIO_CTRL_OE_NORMAL);
        } // lnSPI_MODE
        _ => {}
    }
}

/// C++ `lnDigitalWrite(const lnPin pin, bool value)`
#[unsafe(export_name = "_Z14lnDigitalWrite5lnPinb")]
pub extern "C" fn ln_digital_write(pin: i32, value: bool) {
    if let Some(p) = Pin::from_number(pin as u32) {
        drive(p.number(), value);
    }
}

/// C++ `lnDigitalRead(const lnPin pin)`
#[unsafe(export_name = "_Z13lnDigitalRead5lnPin")]
pub extern "C" fn ln_digital_read(pin: i32) -> bool {
    Pin::from_number(pin as u32).is_some_and(|p| sense(p.number()))
}

/// C++ `lnDigitalToggle(const lnPin pin)`
#[unsafe(export_name = "_Z15lnDigitalToggle5lnPin")]
pub extern "C" fn ln_digital_toggle(pin: i32) {
    if let Some(p) = Pin::from_number(pin as u32) {
        flip(p.number());
    }
}

/// C++ `lnOpenDrainClose(const lnPin pin, const bool close)`
#[unsafe(export_name = "_Z16lnOpenDrainClose5lnPinb")]
pub extern "C" fn ln_open_drain_close(pin: i32, close: bool) {
    if let Some(p) = Pin::from_number(pin as u32) {
        open_drain_close(p.number(), close);
    }
}
