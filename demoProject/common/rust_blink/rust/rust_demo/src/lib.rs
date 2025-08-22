#![no_std]
//use rnarduino::rn_os_helper;
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

    rust_esprit::pin_mode(PIN, rust_esprit::GpioMode::lnOUTPUT);

    let mut on: bool = false;
    for _i in 0..5 {
        rust_esprit::digital_write(PIN, on);
        on = !on;
        rust_esprit::delay_ms(1000);
    }

    lnLogger!("--end--\n");
}

// EOF
