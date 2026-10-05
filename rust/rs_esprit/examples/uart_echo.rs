#![no_std]
#![no_main]

use core::panic::PanicInfo;
use rs_esprit::uart::{Parity, StopBits, UartConfig, UartRx, UartRxHandler, UartTx, UartTxHandler};

/// Custom application handler implementing UART reception callbacks.
struct EchoRxHandler;

impl UartRxHandler for EchoRxHandler {
    fn on_rx_byte(&self, byte: u8) {
        let _ = byte;
    }

    fn on_rx_buffer(&self, buffer: &[u8]) {
        let _ = buffer.len();
    }

    fn on_error(&self) {
        // Handle framing / parity / overrun errors
    }
}

/// Custom application handler implementing UART transmission callbacks.
struct EchoTxHandler;

impl UartTxHandler for EchoTxHandler {
    fn on_tx_complete(&self) {
        // Notify transmission completed
    }
}

static RX_HANDLER: EchoRxHandler = EchoRxHandler;
static TX_HANDLER: EchoTxHandler = EchoTxHandler;

/// Demonstrates configuring and using generic UART drivers via `rs_esprit` traits.
pub fn run_uart_demo<TX: UartTx, RX: UartRx>(mut tx: TX, mut rx: RX) {
    let config = UartConfig {
        baudrate: 115200,
        data_bits: 8,
        stop_bits: StopBits::One,
        parity: Parity::None,
    };

    tx.configure(&config);
    rx.configure(&config);

    // Completely safe registration - no unsafe or raw pointers needed!
    tx.set_tx_handler(Some(&TX_HANDLER));
    rx.set_rx_handler(Some(&RX_HANDLER));

    rx.enable_rx(true);

    let hello = b"Hello from rs_esprit UartTx!\r\n";
    tx.transmit(hello);
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
