#![no_std]
#![no_main]

use core::panic::PanicInfo;
use rs_esprit::spi::{Spi, SpiBitOrder, SpiConfig, SpiMode, SpiRxHandler, SpiTxHandler};

/// Application handler for SPI receive events.
struct AppSpiRxHandler;

impl SpiRxHandler for AppSpiRxHandler {
    fn on_rx_byte(&self, byte: u8) {
        let _ = byte;
    }

    fn on_rx_buffer(&self, buffer: &[u8]) {
        let _ = buffer.len();
    }
}

/// Application handler for SPI transmit completion.
struct AppSpiTxHandler;

impl SpiTxHandler for AppSpiTxHandler {
    fn on_tx_complete(&self) {
        // Transmission finished
    }
}

static RX_HANDLER: AppSpiRxHandler = AppSpiRxHandler;
static TX_HANDLER: AppSpiTxHandler = AppSpiTxHandler;

/// Demonstrates configuring and using generic SPI drivers via `rs_esprit` traits.
pub fn run_spi_demo<S: Spi>(mut spi: S) {
    let config = SpiConfig {
        speed_hz: 8_000_000,
        mode: SpiMode::Mode0,
        bit_order: SpiBitOrder::MsbFirst,
        data_size: 8,
    };

    spi.configure(&config);

    // Completely safe registration - no unsafe or raw pointers needed!
    spi.set_rx_handler(Some(&RX_HANDLER));
    spi.set_tx_handler(Some(&TX_HANDLER));

    // Single-byte transfer
    let rx_val = spi.transfer8(0x42);
    let _ = rx_val;

    // Asynchronous block transfer
    let tx_data = [0x01, 0x02, 0x03, 0x04];
    spi.write_async(&tx_data);
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
