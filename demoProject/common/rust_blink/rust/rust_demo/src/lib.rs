#![no_std]
//use rnarduino::rn_os_helper;
use rust_esprit::delay_ms;
use rust_esprit::{GpioMode::lnOUTPUT, digital_write, pin_mode};
use rust_esprit::{lnLogger, lnLogger_init};

lnLogger_init!();
#[cfg(feature = "rp2040")]
const PIN: rust_esprit::pin = rust_esprit::pin::GPIO10;

#[cfg(not(feature = "rp2040"))]
const PIN: rust_esprit::pin = rust_esprit::pin::PB6;

/**
 *
 *
 */
#[unsafe(no_mangle)]
extern "C" fn user_init() {
    lnLogger!("Hello there !\n");

    pin_mode(PIN, lnOUTPUT);

    let mut on: bool = false;
    for _i in 0..5 {
        digital_write(PIN, on);
        on = !on;
        delay_ms(1000);
    }

    lnLogger!("--end--\n");
}

// EOF
