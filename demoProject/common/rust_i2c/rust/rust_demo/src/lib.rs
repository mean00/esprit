//! Common Rust I2C demo using high-level `rust_esprit` API.
//!
//! Follows Esprit Rust Rule 1 (No Magic Numbers), Rule 2 (Two-Tier Architecture),
//! Rule 4 (Mandatory Examples), and Rule 6 (Idiomatic Rust).

#![no_std]

use rust_esprit::{logger, logger_init};
use rust_esprit::I2c;

logger_init!();

const I2C_INSTANCE: u32 = 0;
const I2C_SPEED_HZ: u32 = 100_000;
const TARGET_ADDR: u8 = 0x3C; // Standard OLED / EEPROM demo address

#[unsafe(no_mangle)]
extern "C" fn user_init() {
    logger!("Starting I2C test...\n");

    let mut i2c = I2c::new(I2C_INSTANCE, I2C_SPEED_HZ);
    i2c.set_speed(I2C_SPEED_HZ);
    i2c.set_address(TARGET_ADDR);

    // 1. Transaction write
    let cmd = [0x00u8, 0xAF]; // Example Display ON command
    logger!("Writing command bytes to addr 0x3C...\n");
    let write_ok = i2c.write_to(TARGET_ADDR, &cmd);
    logger!("write_to ok: {}\n", if write_ok { 1 } else { 0 });

    // 2. Multi-buffer gather write
    let header = [0x00u8];
    let payload = [0x01u8, 0x02, 0x03];
    let chunks: [&[u8]; 2] = [&header, &payload];
    logger!("Multi-writing gather buffers...\n");
    let multi_ok = i2c.multi_write_to(TARGET_ADDR, &[1, 3], &chunks);
    logger!("multi_write_to ok: {}\n", if multi_ok { 1 } else { 0 });

    // 3. Read transaction
    let mut rx_buf = [0u8; 2];
    logger!("Reading 2 bytes from addr 0x3C...\n");
    let read_ok = i2c.read_from(TARGET_ADDR, &mut rx_buf);
    logger!("read_from ok: {}\n", if read_ok { 1 } else { 0 });

    logger!("--end--\n");
}
