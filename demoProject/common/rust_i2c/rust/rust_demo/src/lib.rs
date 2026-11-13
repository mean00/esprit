//! Common Rust I2C demo using the MCU-agnostic I²C super trait.
//!
//! The public `rust_esprit` API no longer has a flat `I2c` wrapper: an I²C port is
//! the platform driver (`rust_esprit::bluepill::i2c::I2c`, i.e. `rs_i2c_bluepill`)
//! driven through `rs_mcu_traits::i2c::I2c`, re-exported as `rust_esprit::HalI2c`.
//! A transaction is described as a list of `Operation`s, and the convenience
//! methods (`write_to`, `write_read`, `write_chunks`) are defaults of that trait.
//!
//! Follows Esprit Rust Rule 1 (No Magic Numbers), Rule 2 (Two-Tier Architecture),
//! Rule 4 (Mandatory Examples), and Rule 6 (Idiomatic Rust).

#![no_std]

use rust_esprit::{HalI2c, I2cConfig, I2cDriver, logger, logger_init};

logger_init!();

#[cfg(feature = "rp2040")]
compile_error!(
    "this demo drives the GD32/CH32 I2C backend (bluepill48); the RP2xxx I2C driver has no \
     constructor in esprit yet"
);

const I2C_INSTANCE: u32 = 0;
const I2C_SPEED_HZ: u32 = 100_000;
const TARGET_ADDR: u8 = 0x3C; // Standard OLED / EEPROM demo address

#[unsafe(no_mangle)]
extern "C" fn user_init() {
    logger!("Starting I2C test...\n");

    let mut i2c = I2cDriver::new(I2C_INSTANCE, I2C_SPEED_HZ);
    let config = I2cConfig {
        speed_hz: I2C_SPEED_HZ,
        own_address: 0,
    };
    logger!(
        "configure: {}\n",
        if i2c.configure(&config).is_ok() { 1 } else { 0 }
    );

    // 1. Transaction write
    let cmd = [0x00u8, 0xAF]; // Example Display ON command
    logger!("Writing command bytes to addr 0x3C...\n");
    let write_ok = HalI2c::write_to(&mut i2c, TARGET_ADDR, &cmd).is_ok();
    logger!("write_to ok: {}\n", if write_ok { 1 } else { 0 });

    // 2. Multi-buffer gather write: the `write_chunks` default turns the chunks
    //    into one transaction, i.e. a single start/stop around all the payloads.
    let header = [0x00u8];
    let payload = [0x01u8, 0x02, 0x03];
    let chunks: [&[u8]; 2] = [&header, &payload];
    logger!("Multi-writing gather buffers...\n");
    let multi_ok = HalI2c::write_chunks(&mut i2c, TARGET_ADDR, &chunks).is_ok();
    logger!("write_chunks ok: {}\n", if multi_ok { 1 } else { 0 });

    // 3. Read transaction
    let mut rx_buf = [0u8; 2];
    logger!("Reading 2 bytes from addr 0x3C...\n");
    let read_ok = HalI2c::read_from(&mut i2c, TARGET_ADDR, &mut rx_buf).is_ok();
    logger!("read_from ok: {}\n", if read_ok { 1 } else { 0 });

    logger!("--end--\n");
}

