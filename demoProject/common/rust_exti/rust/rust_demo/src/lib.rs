#![no_std]

use rust_esprit::delay_ms;
use rust_esprit::{Edge, Exti, ExtiDriver, ExtiHandler, Gpio, Pin, PinMode, logger, logger_init};

logger_init!();

#[cfg(any(feature = "rp2040"))]
const PIN: Pin = Pin::GPIO10;
#[cfg(not(any(feature = "rp2040")))]
const PIN: Pin = Pin::PA4;

/// Runs in the EXTI interrupt, so it only touches atomics or the logger.
struct MyStruct {
    context: bool,
}

impl ExtiHandler for MyStruct {
    fn on_exti(&self, _pin: u32) {
        logger!("on_interrupt (context={})\n", if self.context { 1 } else { 0 });
    }
}

static HANDLER: MyStruct = MyStruct { context: true };

#[unsafe(no_mangle)]
extern "C" fn user_init() {
    logger!("Hello there !\n");

    // A pin interrupt is two contracts: the pad (`Gpio`) and the interrupt
    // controller (`Exti`), which is addressed by pin number and takes a typed
    // handler.
    let mut pin = PIN;
    let _ = Gpio::set_mode(&mut pin, PinMode::InputPullUp);
    let mut exti = ExtiDriver;
    let _ = Exti::attach(&mut exti, PIN as u32, Edge::Both, Some(&HANDLER));

    let mut on: bool = false;
    for _i in 0..5 {
        Gpio::write(&mut pin, on);
        on = !on;
        delay_ms(1000);
    }

    logger!("--end--\n");
}

