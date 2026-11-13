//! Common Rust Serial demo using the MCU-agnostic UART super trait.
//!
//! The port is the family driver (`rust_esprit::bluepill::uart::Uart`, i.e.
//! `rs_uart_bluepill`), configured in one call, and the traffic goes through
//! `rs_mcu_traits::uart::Uart`, re-exported as `rust_esprit::HalUart`.
//!
//! Follows Esprit Rust Rule 1 (No Magic Numbers), Rule 2 (Two-Tier Architecture),
//! Rule 4 (Mandatory Examples), and Rule 6 (Idiomatic Rust).

#![no_std]

use rust_esprit::{HalUart, UartConfig, UartDriver, logger, logger_init};

logger_init!();

const UART_INSTANCE: u32 = 0;
const UART_BAUDRATE: u32 = 115_200;
const RX_BUFFER_SIZE: usize = 64;

#[unsafe(no_mangle)]
extern "C" fn user_init() {
    logger!("Starting Serial test...\n");

    // 8N1 at the requested rate: the frame format and the rate are one call.
    let config = UartConfig {
        baudrate: UART_BAUDRATE,
        ..UartConfig::default()
    };
    let mut serial = match UartDriver::new(UART_INSTANCE, &config) {
        Ok(serial) => serial,
        Err(_) => {
            logger!("uart init failed\n");
            return;
        }
    };

    let msg = b"Hello from Rust Serial!\r\n";
    logger!("Transmitting greeting...\n");
    let ok = HalUart::write(&mut serial, msg).is_ok();
    logger!("transmit ok: {}\n", if ok { 1 } else { 0 });

    // Non-blocking drain of whatever the receiver ring already holds.
    let mut buffer = [0u8; RX_BUFFER_SIZE];
    let n = HalUart::read(&mut serial, &mut buffer).unwrap_or(0);
    logger!("received {} bytes\n", n);

    logger!("--end--\n");
}

