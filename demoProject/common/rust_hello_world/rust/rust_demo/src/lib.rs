#![no_std]
//use rnarduino::rn_os_helper;
use rust_esprit::{logger, logger_init};

logger_init!();

/**
 *
 *
 */
#[unsafe(no_mangle)]
extern "C" fn user_init() {
    logger!("Hello there !\n");
    let number = 0x1234;
    logger!("in decimal {}\n", number);
    logger!("in hex {:x}\n", number);
    logger!("--end--\n");
}

// EOF
