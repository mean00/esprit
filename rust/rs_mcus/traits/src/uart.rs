//! UART: one trait for the whole port, one handler per instance.
//!
//! `UartTx` and `UartRx` were two traits on a single port; this is one, because
//! a port is configured once and an interrupt dispatches both directions. Two
//! UARTs later (bridge on UART0, logger on UART1) are two instances with two
//! handlers — see [`crate::handler::HandlerTable`].

use crate::error::{Error, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Parity {
    None,
    Even,
    Odd,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StopBits {
    One,
    Two,
}

/// Frame format and rate of a port.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UartConfig {
    pub baudrate: u32,
    pub data_bits: u8,
    pub stop_bits: StopBits,
    pub parity: Parity,
}

impl Default for UartConfig {
    fn default() -> Self {
        Self {
            baudrate: 115_200,
            data_bits: 8,
            stop_bits: StopBits::One,
            parity: Parity::None,
        }
    }
}

/// What a UART interrupt calls. Every method has a default: a caller that only
/// wants blocking I/O installs no handler at all.
pub trait UartHandler: Sync {
    /// One received byte (called from the RX interrupt).
    fn on_rx(&self, _byte: u8) {}
    /// A received block, when the driver has one (DMA/RX-timeout paths).
    fn on_rx_buffer(&self, _buffer: &[u8]) {}
    /// New bytes are waiting in the driver's buffer: the next
    /// [`Uart::read`] returns them.
    ///
    /// This is the cheapest notification a driver with a receive ring can give
    /// from its interrupt — it means "wake the consumer", not "here is the
    /// data" (that is [`on_rx_buffer`](Self::on_rx_buffer), which needs the
    /// driver to copy out of the ring). A pass-through bridge only needs this
    /// one; a handler that wants the bytes inline uses `on_rx`/`on_rx_buffer`.
    fn on_rx_available(&self) {}
    /// The last byte left the shift register.
    fn on_tx_complete(&self) {}
    /// Framing/overrun/parity error.
    fn on_error(&self, _error: Error) {}
}

/// A UART port.
pub trait Uart {
    /// Apply a frame format and rate; the port is left enabled.
    fn configure(&mut self, config: &UartConfig) -> Result<()>;

    /// Change the rate only (the frame format is kept).
    fn set_baudrate(&mut self, baudrate: u32) -> Result<()>;

    /// Send every byte; returns once the last one left the shift register.
    fn write(&mut self, data: &[u8]) -> Result<()>;

    /// Queue as much of `data` as fits without blocking; `Ok(n)` is the number of
    /// bytes accepted, which may be less than `data.len()`.
    fn try_write(&mut self, data: &[u8]) -> Result<usize>;

    /// Block until everything queued has been transmitted.
    fn flush(&mut self) -> Result<()>;

    /// Copy the bytes that are already in the receive path; `Ok(0)` means none.
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize>;

    /// Block until `buffer` is full or the port went idle: `Ok(n)` is how many
    /// bytes arrived.
    fn read_until_idle(&mut self, buffer: &mut [u8]) -> Result<usize>;

    /// Enable or disable the receiver.
    fn enable_rx(&mut self, enabled: bool) -> Result<()>;

    /// Install the handler of **this** instance.
    fn set_handler(&mut self, handler: Option<&'static dyn UartHandler>);
}
