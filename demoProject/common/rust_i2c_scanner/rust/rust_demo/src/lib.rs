#![no_std]
//use rnarduino::rn_os_helper;
use rust_esprit::delay_ms;
use rust_esprit::i2c;
//use rust_esprit::{GpioMode::lnOUTPUT, digital_write, pin_mode};
use rust_esprit::{lnLogger, lnLogger_init};

lnLogger_init!();
////#[cfg(feature = "rp2040")]
//const PIN: rust_esprit::pin = rust_esprit::pin::GPIO10;
//
//#[cfg(not(feature = "rp2040"))]
//const PIN: rust_esprit::pin = rust_esprit::pin::PB6;

/**
 *
 *
 */
#[unsafe(no_mangle)]
extern "C" fn user_init() {
    lnLogger!("I2C Scanner !\n");
    let mut i2c = i2c::new(0, 200000);
    loop {
        for i in 0..127u8 {
            let dex = i as u8;
            i2c.set_address(dex);
            if !i2c.write(&[]) {
                delay_ms(100);
            } else {
                lnLogger!("Found a i2c device at address {} 0x{:x}\n", dex, dex);
            }
        }
    }
}

// EOF
