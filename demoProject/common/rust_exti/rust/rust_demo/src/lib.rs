#![no_std]

use rust_esprit::delay_ms;
use rust_esprit::{Edge, GpioMode, GpioPin, Pin, PinCallback};
use rust_esprit::{logger, logger_init};

logger_init!();

#[cfg(any(feature = "rp2040"))]
const PIN: Pin = Pin::GPIO10;
#[cfg(not(any(feature = "rp2040")))]
const PIN: Pin = Pin::PA4;

struct MyStruct {
    context: bool,
}

impl PinCallback for MyStruct {
    fn on_interrupt(&mut self, pin: Pin) {
        panic!("on_interrupt\n");
    }
}

#[unsafe(no_mangle)]
extern "C" fn user_init() {
    logger!("Hello there !\n");

    let mut pin = GpioPin::new(PIN);
    pin.set_mode(GpioMode::InputPullUp);
    let s: MyStruct = MyStruct { context: true };
    rust_esprit::attach_interrupt(PIN, Edge::Both, &s);
    let mut on: bool = false;
    for _i in 0..5 {
        pin.write(on);
        on = !on;
        delay_ms(1000);
    }

    logger!("--end--\n");
}
