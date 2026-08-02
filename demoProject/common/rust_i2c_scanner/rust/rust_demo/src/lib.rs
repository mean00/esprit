#![no_std]
use rust_esprit::I2c;
use rust_esprit::delay_ms;
use rust_esprit::{logger, logger_init};

logger_init!();

/**
 *
 *
 */
#[unsafe(no_mangle)]
extern "C" fn user_init() {
    logger!("I2C Scanner !\n");
    let mut i2c = I2c::new(0, 200000);
    loop {
        for i in 0..127u8 {
            let dex = i as u8;
            i2c.set_address(dex);
            if !i2c.write(&[]) {
                delay_ms(100);
            } else {
                logger!("Found a i2c device at address {} 0x{:x}\n", dex, dex);
            }
        }
    }
}

// EOF
