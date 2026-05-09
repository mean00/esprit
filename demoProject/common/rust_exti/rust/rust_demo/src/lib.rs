#![no_std]

use rust_esprit::delay_ms;
use rust_esprit::{GpioMode::lnINPUT_PULLUP, digital_write, pin, pin_mode};
use rust_esprit::{logger, logger_init};

logger_init!();

#[cfg(any(feature = "rp2040"))]
const PIN: pin = pin::GPIO10;
#[cfg(not(any(feature = "rp2040")))]
const PIN: pin = pin::PA4;

struct MyStruct {
    context: bool,
}

impl rust_esprit::PinCallback for MyStruct {
    fn on_interrupt(&mut self, pin: rust_esprit::pin) {
        panic!("on_interrupt\n");
    }
}

#[unsafe(no_mangle)]
extern "C" fn user_init() {
    logger!("Hello there !\n");

    pin_mode(PIN, lnINPUT_PULLUP);
    let s: MyStruct = MyStruct { context: true };
    rust_esprit::exti_attach_interrupt_typed(PIN, rust_esprit::Edge::Both, &s);
    let mut on: bool = false;
    for _i in 0..5 {
        digital_write(PIN, on);
        on = !on;
        delay_ms(1000);
    }

    logger!("--end--\n");
}
