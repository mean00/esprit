//! Caller constants and configuration enums for the RP2040 SPI driver.

#![allow(dead_code)]

pub const SPI_INSTANCE_0: u32 = 0;
pub const SPI_INSTANCE_1: u32 = 1;
pub const SPI_MAX_INSTANCES: usize = 2;

/// Default SPI bus clock speed in Hertz (1 MHz).
pub const DEFAULT_SPEED_HZ: u32 = 1_000_000;

/// Default peripheral clock frequency on RP2040 (125 MHz).
pub const DEFAULT_PERI_CLOCK_HZ: u32 = 125_000_000;

/// SPI data frame size in bits.
pub const SPI_DATA_SIZE_8: u32 = 8;
pub const SPI_DATA_SIZE_16: u32 = 16;

/// Spin budget for polling hardware status flags.
pub const POLL_LIMIT: u32 = 1_000_000;

/// Minimum transfer size to use DMA rather than polling.
pub const MIN_DMA_TRANSFER_SIZE: usize = 16;

/// Default timeout in milliseconds for synchronous DMA transfers.
pub const DMA_TIMEOUT_MS: u32 = 2000;

/// Default timeout in milliseconds for asynchronous operations.
pub const ASYNC_TIMEOUT_MS: u32 = 5000;

/// Dummy byte pattern transmitted during receive-only operations.
pub const SPI_DUMMY_BYTE: u8 = 0xFF;
pub const SPI_DUMMY_WORD: u16 = 0xFFFF;

/// SPI clock phase and polarity modes (Motorola format).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum SpiMode {
    /// CPOL = 0, CPHA = 0: Clock idle low, data captured on first clock edge.
    Mode0 = 0,
    /// CPOL = 0, CPHA = 1: Clock idle low, data captured on second clock edge.
    Mode1 = 1,
    /// CPOL = 1, CPHA = 0: Clock idle high, data captured on first clock edge.
    Mode2 = 2,
    /// CPOL = 1, CPHA = 1: Clock idle high, data captured on second clock edge.
    Mode3 = 3,
}

impl From<u32> for SpiMode {
    fn from(val: u32) -> Self {
        match val & 3 {
            0 => Self::Mode0,
            1 => Self::Mode1,
            2 => Self::Mode2,
            _ => Self::Mode3,
        }
    }
}

/// Bit transmission order.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum BitOrder {
    LsbFirst = 0,
    MsbFirst = 1,
}

impl From<u32> for BitOrder {
    fn from(val: u32) -> Self {
        match val {
            0 => Self::LsbFirst,
            _ => Self::MsbFirst,
        }
    }
}
