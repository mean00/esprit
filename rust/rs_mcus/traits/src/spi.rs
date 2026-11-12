//! SPI: a master port, blocking and interrupt-driven transfers.
//!
//! Chip-select and DMA are driver knobs, not contract: a family exposes them on
//! its own type. The trait keeps what every SPI master must do.

use crate::error::Result;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpiMode {
    Mode0,
    Mode1,
    Mode2,
    Mode3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BitOrder {
    MsbFirst,
    LsbFirst,
}

/// Rate, clock polarity/phase, bit order and word width of a port.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpiConfig {
    pub speed_hz: u32,
    pub mode: SpiMode,
    pub bit_order: BitOrder,
    pub data_size: u8,
}

impl Default for SpiConfig {
    fn default() -> Self {
        Self {
            speed_hz: 1_000_000,
            mode: SpiMode::Mode0,
            bit_order: BitOrder::MsbFirst,
            data_size: 8,
        }
    }
}

/// What an SPI interrupt calls.
pub trait SpiHandler: Sync {
    /// A received byte (slave mode, or an asynchronous transfer).
    fn on_rx(&self, _byte: u8) {}
    /// A received block (DMA path).
    fn on_rx_buffer(&self, _buffer: &[u8]) {}
    /// The last bit left the shift register.
    fn on_tx_complete(&self) {}
    /// Overrun/underrun/mode fault.
    fn on_error(&self, _error: crate::error::Error) {}
}

/// An SPI master port.
///
/// Three families of operation, all on the same trait:
///
/// * **single byte** — [`transfer`](Self::transfer);
/// * **blocking blocks** — [`write_slice`](Self::write_slice),
///   [`write_slice16`](Self::write_slice16), [`read_slice`](Self::read_slice),
///   [`fill`](Self::fill), [`fill16`](Self::fill16),
///   [`transfer_in_place`](Self::transfer_in_place),
///   [`transfer_exact`](Self::transfer_exact). The defaults clock the bytes one at
///   a time; a family with a DMA/polled block path overrides them, which is the
///   whole point of the contract being on the trait;
/// * **asynchronous** — [`start_write`](Self::start_write) /
///   [`start_write16`](Self::start_write16) return as soon as the transfer is
///   armed, and the handler installed with [`set_handler`](Self::set_handler) is
///   called **from the DMA/SPI interrupt** when the last bit left the shifter
///   ([`SpiHandler::on_tx_complete`]); [`is_busy`](Self::is_busy) polls the same
///   state and [`wait_idle`](Self::wait_idle) blocks on it.
///
/// The asynchronous buffer belongs to the DMA until the completion runs: it must
/// not be reused before `on_tx_complete` or a successful `wait_idle`.
pub trait Spi {
    /// Apply the configuration; the port is left enabled.
    fn configure(&mut self, config: &SpiConfig) -> Result<()>;

    /// Change the clock only.
    fn set_speed(&mut self, speed_hz: u32) -> Result<()>;

    /// Exchange one byte.
    fn transfer(&mut self, byte: u8) -> u8;

    // ---- blocking block operations (overridden by the families) -------------

    /// Clock `data` out, ignoring what comes back.
    fn write_slice(&mut self, data: &[u8]) -> Result<()> {
        for byte in data {
            self.transfer(*byte);
        }
        Ok(())
    }

    /// Clock `data` out as 16-bit frames.
    fn write_slice16(&mut self, data: &[u16]) -> Result<()> {
        for word in data {
            self.transfer((*word >> 8) as u8);
            self.transfer(*word as u8);
        }
        Ok(())
    }

    /// Clock `buffer.len()` bytes in (writing zeros out).
    fn read_slice(&mut self, buffer: &mut [u8]) -> Result<()> {
        for slot in buffer.iter_mut() {
            *slot = self.transfer(0);
        }
        Ok(())
    }

    /// Clock `count` copies of `byte` out.
    fn fill(&mut self, count: u32, byte: u8) -> Result<()> {
        for _ in 0..count {
            self.transfer(byte);
        }
        Ok(())
    }

    /// Clock `count` copies of `word` out as 16-bit frames.
    fn fill16(&mut self, count: u32, word: u16) -> Result<()> {
        for _ in 0..count {
            self.transfer((word >> 8) as u8);
            self.transfer(word as u8);
        }
        Ok(())
    }

    /// Clock `tx` out and `rx` in; the two slices have the same length.
    fn transfer_in_place(&mut self, tx: &[u8], rx: &mut [u8]) -> Result<()> {
        if tx.len() != rx.len() {
            return Err(crate::error::Error::InvalidConfig);
        }
        for (out, in_) in tx.iter().zip(rx.iter_mut()) {
            *in_ = self.transfer(*out);
        }
        Ok(())
    }

    /// [`transfer_in_place`](Self::transfer_in_place) under its historical name.
    fn transfer_exact(&mut self, tx: &[u8], rx: &mut [u8]) -> Result<()> {
        self.transfer_in_place(tx, rx)
    }

    // ---- asynchronous ------------------------------------------------------

    /// Start a DMA write and return; [`SpiHandler::on_tx_complete`] runs in the
    /// interrupt when the last bit is out.
    ///
    /// `Err(Unsupported)` when the family has no asynchronous path.
    fn start_write(&mut self, _data: &[u8]) -> Result<()> {
        Err(crate::error::Error::Unsupported)
    }

    /// [`start_write`](Self::start_write) for 16-bit frames.
    fn start_write16(&mut self, _data: &[u16]) -> Result<()> {
        Err(crate::error::Error::Unsupported)
    }

    /// `true` while an asynchronous transfer is in flight.
    fn is_busy(&self) -> bool {
        false
    }

    /// Block until the asynchronous transfer armed with
    /// [`start_write`](Self::start_write) completes.
    fn wait_idle(&mut self) -> Result<()> {
        Ok(())
    }

    /// Install the handler of **this** instance: it receives `on_tx_complete`,
    /// `on_rx`/`on_rx_buffer` and `on_error` from the SPI/DMA interrupt.
    fn set_handler(&mut self, handler: Option<&'static dyn SpiHandler>);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The byte-at-a-time defaults are what a driver without a block path gets:
    /// `transfer_in_place` must clock exactly the same bytes as `transfer`.
    #[test]
    fn transfer_in_place_checks_lengths() {
        struct Loopback;
        impl Spi for Loopback {
            fn configure(&mut self, _config: &SpiConfig) -> Result<()> {
                Ok(())
            }
            fn set_speed(&mut self, _speed_hz: u32) -> Result<()> {
                Ok(())
            }
            fn transfer(&mut self, byte: u8) -> u8 {
                byte ^ 0xff
            }
            fn set_handler(&mut self, _handler: Option<&'static dyn SpiHandler>) {}
        }

        let mut port = Loopback;
        let mut rx = [0u8; 3];
        assert!(port.transfer_in_place(&[1, 2, 3], &mut rx).is_ok());
        assert_eq!(rx, [0xfe, 0xfd, 0xfc]);

        let mut short = [0u8; 2];
        assert_eq!(
            port.transfer_in_place(&[1, 2, 3], &mut short),
            Err(crate::error::Error::InvalidConfig)
        );
    }
}
