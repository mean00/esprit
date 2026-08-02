#![no_std]
use rust_esprit::delay_ms;
use rust_esprit::{GpioMode, GpioPin, Pin};
use rust_esprit::{logger, logger_init};

logger_init!();

#[cfg(feature = "rp2040")]
const PIN: Pin = Pin::GPIO10;

#[cfg(not(feature = "rp2040"))]
const PIN: Pin = Pin::PB6;

/**
 *
 *
 */
#[unsafe(no_mangle)]
extern "C" fn user_init() {
    logger!("Hello there !\n");

    let mut led = GpioPin::new(PIN);
    led.set_mode(GpioMode::Output);

    let mut on: bool = false;
    for _i in 0..5 {
        led.write(on);
        on = !on;
        delay_ms(1000);
    }

    logger!("--end--\n");
}

// EOF
