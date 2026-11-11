//! DMA: move bytes between memory and a peripheral without the CPU.
//!
//! The two families differ a lot underneath (GD32: per-channel `CCR`/config
//! registers; RP2xxx: one `CTRL_TRIG` word plus DREQ numbers), so the contract is
//! the *shape* of a transfer — direction, width, count, how it completes — and the
//! channel handle is an associated type owned by the family's controller.
//!
//! # Lifetime rule
//!
//! [`Dma::start_write`] and [`Dma::start_read`] program *and* start the transfer;
//! the buffer belongs to the DMA until the transfer completes, i.e. until
//! [`Dma::wait`] returns or the handler installed with [`Dma::set_handler`] runs.
//! The borrow checker cannot enforce that across an asynchronous transfer, so it
//! is part of the contract: a caller that reuses the buffer before the completion
//! has stepped outside the safe API.

use crate::error::Result;

/// Width of one bus element.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransferWidth {
    Byte,
    HalfWord,
    Word,
}

impl TransferWidth {
    /// Size of one element, in bytes.
    pub const fn byte_size(self) -> usize {
        match self {
            Self::Byte => 1,
            Self::HalfWord => 2,
            Self::Word => 4,
        }
    }
}

/// Which way the data moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    MemoryToPeripheral,
    PeripheralToMemory,
    MemoryToMemory,
}

/// What a DMA interrupt calls.
pub trait DmaHandler: Sync {
    /// The transfer armed on this channel completed.
    fn on_complete(&self);

    /// The transfer ended with a bus/FIFO error.
    fn on_error(&self, _error: crate::error::Error) {}
}

/// The DMA controller of a family.
///
/// The controller *owns* the channels it hands out, so a handle stays valid until
/// [`Dma::release`] — a driver that keeps its handle for its whole life (the ADC
/// and SPI drivers do) never has to re-claim one.
pub trait Dma {
    /// Handle of one channel of this family.
    type Channel: Copy;

    /// Take a free channel, or `Err(Busy)` when none is left.
    fn claim(&mut self) -> Result<Self::Channel>;

    /// Release a channel claimed with [`Dma::claim`]; an in-flight transfer is
    /// aborted.
    fn release(&mut self, channel: Self::Channel);

    /// Program and start a memory → peripheral transfer of `buffer`.
    ///
    /// `peripheral_address` is the peripheral's data register (`DR`, `SSPDR`, the
    /// ADC FIFO…), `request` the family's DREQ number that paces it (`None` for an
    /// unpaced transfer).
    fn start_write(
        &mut self,
        channel: Self::Channel,
        buffer: &[u8],
        peripheral_address: u32,
        width: TransferWidth,
        request: Option<u32>,
    ) -> Result<()>;

    /// Program and start a peripheral → memory transfer into `buffer`.
    fn start_read(
        &mut self,
        channel: Self::Channel,
        buffer: &mut [u8],
        peripheral_address: u32,
        width: TransferWidth,
        request: Option<u32>,
    ) -> Result<()>;

    /// Block until the transfer on `channel` completes.
    fn wait(&mut self, channel: Self::Channel, timeout_ms: u32) -> Result<()>;

    /// Install the completion handler of **this** channel.
    fn set_handler(&mut self, channel: Self::Channel, handler: Option<&'static dyn DmaHandler>);
}

/// Elements in a `buffer` of `width`-sized elements — what a transfer counter
/// holds.
pub fn elements(buffer: &[u8], width: TransferWidth) -> usize {
    buffer.len() / width.byte_size()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A transfer counts elements, not bytes: 64 bytes are 64 byte-transfers, 32
    /// half-word ones and 16 word ones.
    #[test]
    fn element_count_follows_the_width() {
        let buffer = [0u8; 64];
        assert_eq!(elements(&buffer, TransferWidth::Byte), 64);
        assert_eq!(elements(&buffer, TransferWidth::HalfWord), 32);
        assert_eq!(elements(&buffer, TransferWidth::Word), 16);
    }

    /// A tail that does not fill one element is not transferred (the counter is
    /// an integer number of elements).
    #[test]
    fn a_partial_element_is_dropped() {
        assert_eq!(elements(&[0u8; 3], TransferWidth::HalfWord), 1);
        assert_eq!(elements(&[0u8; 3], TransferWidth::Word), 0);
    }
}
