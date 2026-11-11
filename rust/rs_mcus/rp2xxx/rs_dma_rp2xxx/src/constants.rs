//! Constants for the RP2040 DMA driver used by callers and configuration layers.

#![allow(dead_code)]

/// Maximum number of DMA channels on the RP2040 (0..11).
pub const DMA_MAX_CHANNELS: usize = 12;

/// Default timeout in milliseconds for synchronous DMA transfers.
pub const DMA_TIMEOUT_MS: u32 = 2000;

/// Default timeout in milliseconds for asynchronous DMA operations.
pub const ASYNC_TIMEOUT_MS: u32 = 5000;

/// Maximum spin loops for polling operations before giving up.
pub const POLL_LIMIT: u32 = 1_000_000;

// --- DREQ (Data Request) Channel Selectors (Datasheet §2.5.3.1 Table 119) ---
pub const DREQ_PIO0_TX0: u32 = 0;
pub const DREQ_PIO0_TX1: u32 = 1;
pub const DREQ_PIO0_TX2: u32 = 2;
pub const DREQ_PIO0_TX3: u32 = 3;
pub const DREQ_PIO0_RX0: u32 = 4;
pub const DREQ_PIO0_RX1: u32 = 5;
pub const DREQ_PIO0_RX2: u32 = 6;
pub const DREQ_PIO0_RX3: u32 = 7;
pub const DREQ_PIO1_TX0: u32 = 8;
pub const DREQ_PIO1_TX1: u32 = 9;
pub const DREQ_PIO1_TX2: u32 = 10;
pub const DREQ_PIO1_TX3: u32 = 11;
pub const DREQ_PIO1_RX0: u32 = 12;
pub const DREQ_PIO1_RX1: u32 = 13;
pub const DREQ_PIO1_RX2: u32 = 14;
pub const DREQ_PIO1_RX3: u32 = 15;
pub const DREQ_SPI0_TX: u32 = 16;
pub const DREQ_SPI0_RX: u32 = 17;
pub const DREQ_SPI1_TX: u32 = 18;
pub const DREQ_SPI1_RX: u32 = 19;
pub const DREQ_UART0_TX: u32 = 20;
pub const DREQ_UART0_RX: u32 = 21;
pub const DREQ_UART1_TX: u32 = 22;
pub const DREQ_UART1_RX: u32 = 23;
pub const DREQ_PWM_WRAP0: u32 = 24;
pub const DREQ_PWM_WRAP1: u32 = 25;
pub const DREQ_PWM_WRAP2: u32 = 26;
pub const DREQ_PWM_WRAP3: u32 = 27;
pub const DREQ_PWM_WRAP4: u32 = 28;
pub const DREQ_PWM_WRAP5: u32 = 29;
pub const DREQ_PWM_WRAP6: u32 = 30;
pub const DREQ_PWM_WRAP7: u32 = 31;
pub const DREQ_I2C0_TX: u32 = 32;
pub const DREQ_I2C0_RX: u32 = 33;
pub const DREQ_I2C1_TX: u32 = 34;
pub const DREQ_I2C1_RX: u32 = 35;
pub const DREQ_ADC: u32 = 36;
pub const DREQ_XIP_STREAM: u32 = 37;
pub const DREQ_XIP_SSIRX: u32 = 38;
pub const DREQ_XIP_SSITX: u32 = 39;
pub const DREQ_TIMER0: u32 = 0x3B;
pub const DREQ_TIMER1: u32 = 0x3C;
pub const DREQ_TIMER2: u32 = 0x3D;
pub const DREQ_TIMER3: u32 = 0x3E;
/// Permanent unpaced transfer (runs continuously without pacing).
pub const DREQ_PERMANENT: u32 = 0x3F;

/// Transfer priority level for scheduling.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum DmaPriority {
    Low = 0,
    High = 1,
}

/// Width of each bus element transferred by DMA (CTRL.DATA_SIZE).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum DmaTransferWidth {
    Byte = 0,     // 8-bit
    HalfWord = 1, // 16-bit
    Word = 2,     // 32-bit
}

impl DmaTransferWidth {
    #[inline(always)]
    pub const fn byte_size(self) -> usize {
        match self {
            Self::Byte => 1,
            Self::HalfWord => 2,
            Self::Word => 4,
        }
    }
}

/// DMA Transfer Direction.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DmaDirection {
    MemoryToPeripheral,
    PeripheralToMemory,
    MemoryToMemory,
}

/// Control block for DMA gather operations (multi-chunk TX).
///
/// In Alias 3, `trans_count` and `read_addr_trig` are adjacent. Writing
/// this 2-word structure into `al3_trans_count` sets the count and triggers
/// the channel with the new buffer pointer.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct DmaGatherBlock {
    pub trans_count: u32,
    pub read_addr: *const u8,
}

/// Control block for DMA scatter operations (multi-chunk RX).
///
/// In Alias 1, `write_addr` and `trans_count_trig` are adjacent. Writing
/// this 2-word structure into `al1_write_addr` sets the write pointer and triggers
/// the channel with the transfer count.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct DmaScatterBlock {
    pub write_addr: *mut u8,
    pub trans_count: u32,
}
