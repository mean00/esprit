//! Common Rust SPI demo using high-level `rust_esprit` API.
//!
//! Follows Esprit Rust Rule 1 (No Magic Numbers), Rule 2 (Two-Tier Architecture),
//! Rule 4 (Mandatory Examples), and Rule 6 (Idiomatic Rust).

#![no_std]

use rust_esprit::{logger, logger_init};
use rust_esprit::{Pin, Spi, SpiMode, BitOrder};

logger_init!();

#[cfg(any(feature = "rp2040"))]
const CS_PIN: Pin = Pin::GPIO5;
#[cfg(not(any(feature = "rp2040")))]
const CS_PIN: Pin = Pin::PA4;

const SPI_INSTANCE: u32 = 0;
const SPI_SPEED_HZ: u32 = 4_000_000;

#[unsafe(no_mangle)]
extern "C" fn user_init() {
    logger!("Starting SPI test...\n");

    let mut spi = Spi::new(SPI_INSTANCE, CS_PIN);
    spi.begin(8);
    spi.set_speed(SPI_SPEED_HZ);
    spi.set_data_mode(SpiMode::Mode0);
    spi.set_bit_order(BitOrder::MsbFirst);

    // 1. Single byte transfer
    logger!("Writing single byte 0xAA...\n");
    let ok = spi.write8(0xAA);
    logger!("Single write ok: {}\n", if ok { 1 } else { 0 });

    // 2. Block write
    let tx_data = [0x01u8, 0x02, 0x03, 0x04];
    logger!("Writing 4-byte slice...\n");
    let slice_ok = spi.write_slice8(&tx_data);
    logger!("Slice write ok: {}\n", if slice_ok { 1 } else { 0 });

    // 3. Bidirectional transfer
    let tx = [0x10u8, 0x20, 0x30, 0x40];
    let mut rx = [0u8; 4];
    logger!("Transferring 4 bytes...\n");
    let xfer_ok = spi.transfer8(&tx, &mut rx);
    logger!("Transfer ok: {}\n", if xfer_ok { 1 } else { 0 });

    logger!("--end--\n");
}
