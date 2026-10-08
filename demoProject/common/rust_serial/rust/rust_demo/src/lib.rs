//! Common Rust Serial demo using high-level `rust_esprit` API.
//!
//! Follows Esprit Rust Rule 1 (No Magic Numbers), Rule 2 (Two-Tier Architecture),
//! Rule 4 (Mandatory Examples), and Rule 6 (Idiomatic Rust).

#![no_std]

use rust_esprit::{logger, logger_init};
use rust_esprit::Serial;

logger_init!();

const UART_INSTANCE: u32 = 0;
const RX_BUFFER_SIZE: u32 = 256;
const UART_SPEED_HZ: u32 = 115200;

#[unsafe(no_mangle)]
extern "C" fn user_init() {
    logger!("Starting Serial test...\n");

    let mut serial = Serial::new(UART_INSTANCE, RX_BUFFER_SIZE, false);
    serial.init();
    serial.set_speed(UART_SPEED_HZ);
    serial.enable_rx(true);

    let msg = b"Hello from Rust Serial!\r\n";
    logger!("Transmitting greeting...\n");
    let ok = serial.transmit(msg);
    logger!("transmit ok: {}\n", if ok { 1 } else { 0 });

    logger!("--end--\n");
}
