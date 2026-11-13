//! Common Rust SPI demo using the MCU-agnostic SPI super trait.
//!
//! The port is the family driver (`rust_esprit::bluepill::spi::Spi`, i.e.
//! `rs_spi_bluepill`), which keeps the chip-select pin and the DMA path as its own
//! knobs, and the transfers go through `rs_mcu_traits::spi::Spi`, re-exported as
//! `rust_esprit::HalSpi`.
//!
//! Follows Esprit Rust Rule 1 (No Magic Numbers), Rule 2 (Two-Tier Architecture),
//! Rule 4 (Mandatory Examples), and Rule 6 (Idiomatic Rust).

#![no_std]

use rust_esprit::{HalSpi, HalSpiBitOrder, HalSpiMode, Pin, SpiConfig, SpiDriver, logger, logger_init};

logger_init!();

#[cfg(any(feature = "rp2040"))]
const CS_PIN: Pin = Pin::GPIO5;
#[cfg(not(any(feature = "rp2040")))]
const CS_PIN: Pin = Pin::PA4;

const SPI_INSTANCE: u32 = 0;
const SPI_SPEED_HZ: u32 = 4_000_000;
const SPI_WORD_BITS: u8 = 8;

#[unsafe(no_mangle)]
extern "C" fn user_init() {
    logger!("Starting SPI test...\n");

    // Rate, clock phase and frame size in one call; the chip select is the
    // driver's own knob and travels with the constructor.
    let mut spi = SpiDriver::new(SPI_INSTANCE, CS_PIN as i32);
    let cfg_ok = HalSpi::configure(
        &mut spi,
        &SpiConfig {
            speed_hz: SPI_SPEED_HZ,
            mode: HalSpiMode::Mode0,
            bit_order: HalSpiBitOrder::MsbFirst,
            data_size: SPI_WORD_BITS,
        },
    )
    .is_ok();
    logger!("configure ok: {}\n", if cfg_ok { 1 } else { 0 });

    // 1. Single byte transfer
    logger!("Writing single byte 0xAA...\n");
    let echo = HalSpi::transfer(&mut spi, 0xAA);
    logger!("Single write read back 0x{:x}\n", echo);

    // 2. Block write
    let tx_data = [0x01u8, 0x02, 0x03, 0x04];
    logger!("Writing 4-byte slice...\n");
    let slice_ok = HalSpi::write_slice(&mut spi, &tx_data).is_ok();
    logger!("Slice write ok: {}\n", if slice_ok { 1 } else { 0 });

    // 3. Bidirectional transfer
    let tx = [0x10u8, 0x20, 0x30, 0x40];
    let mut rx = [0u8; 4];
    logger!("Transferring 4 bytes...\n");
    let xfer_ok = HalSpi::transfer_in_place(&mut spi, &tx, &mut rx).is_ok();
    logger!("Transfer ok: {}\n", if xfer_ok { 1 } else { 0 });

    logger!("--end--\n");
}

