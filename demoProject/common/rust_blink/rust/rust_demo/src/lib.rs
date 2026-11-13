#![no_std]
use rust_esprit::delay_ms;
use rust_esprit::{Gpio, Pin, PinMode};
use rust_esprit::{logger, logger_init};

logger_init!();

#[cfg(feature = "rp2040")]
const PIN: Pin = Pin::GPIO10;

#[cfg(not(feature = "rp2040"))]
const PIN: Pin = Pin::PB6;

/**
 * The pad is the pin type itself and its behaviour is the `rs_mcu_traits::gpio::Gpio`
 * contract, re-exported as `rust_esprit::Gpio`. The calls are written
 * `Gpio::method(...)` because the pin type also has the family's own inherent
 * methods, with a different arity.
 */
#[unsafe(no_mangle)]
extern "C" fn user_init() {
    logger!("Hello there !\n");

    let mut led = PIN;
    let _ = Gpio::set_mode(&mut led, PinMode::Output);

    let mut on: bool = false;
    for _i in 0..5 {
        Gpio::write(&mut led, on);
        on = !on;
        delay_ms(1000);
    }

    logger!("--end--\n");
}

// EOF

