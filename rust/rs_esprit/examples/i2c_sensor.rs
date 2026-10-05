#![no_std]
#![no_main]

use core::panic::PanicInfo;
use rs_esprit::i2c::{I2c, I2cConfig, I2cHandler};

/// Application handler responding to I2C transactions.
struct AppI2cHandler;

impl I2cHandler for AppI2cHandler {
    fn on_rx_byte(&self, byte: u8) {
        let _ = byte;
    }

    fn on_rx_buffer(&self, buffer: &[u8]) {
        let _ = buffer.len();
    }

    fn on_tx_complete(&self) {
        // Transmission finished
    }

    fn on_error(&self) {
        // Bus error / NACK
    }
}

static I2C_HANDLER: AppI2cHandler = AppI2cHandler;

/// Demonstrates configuring and using generic I2C peripheral drivers via `rs_esprit::I2c`.
pub fn run_i2c_demo<I: I2c>(mut i2c: I) {
    let config = I2cConfig {
        speed_hz: 400_000,
        own_address: 0x00,
    };

    i2c.configure(&config);

    // Completely safe registration - zero unsafe!
    i2c.set_handler(Some(&I2C_HANDLER));

    // Write command to sensor at address 0x68
    let cmd = [0x6B, 0x00];
    i2c.write_to(0x68, &cmd);

    // Read 6 bytes of data from sensor
    let mut data = [0u8; 6];
    i2c.read_from(0x68, &mut data);
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
