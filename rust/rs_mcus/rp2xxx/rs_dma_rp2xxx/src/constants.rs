//! Constants for the RP2xxx DMA driver that do **not** depend on the chip.
//!
//! The chip-specific numbers (channel count, DREQ numbers, the `CTRL_TRIG` field
//! layout) live in `registers_rp2040.rs` / `registers_rp2350.rs`, selected by
//! `registers.rs`.

#![allow(dead_code)]

/// Default timeout in milliseconds for synchronous DMA transfers.
pub const DMA_TIMEOUT_MS: u32 = 2000;

/// Default timeout in milliseconds for asynchronous DMA operations.
pub const ASYNC_TIMEOUT_MS: u32 = 5000;

/// Spin loops allowed for one polling wait (see `wait_until`).
pub const POLL_LIMIT: u32 = 1_000_000;

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

/// How a DMA channel moves data: the element width, the DREQ that paces it, and
/// the two flags `CTRL` carries.
///
/// Bundled because the four travel together and two of them are `bool`s: as loose
/// arguments, `configure_tx(…, width, dreq, true, true)` said nothing about which
/// `true` was "increment" and which was "start now".
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DmaTransferConfig {
    /// Width of one bus element (`CTRL.DATA_SIZE`).
    pub width: DmaTransferWidth,
    /// `CTRL.TREQ`: the DREQ that paces the transfer.
    pub treq: u32,
    /// Increment the moving address (`CTRL.INCR_READ` / `CTRL.INCR_WRITE`).
    /// `false` keeps one address, which is how a FIFO port is fed.
    pub increment: bool,
    /// Write `CTRL_TRIG` (arm *and* start) instead of `AL1_CTRL` (arm only).
    pub trigger: bool,
}

impl DmaTransferConfig {
    /// A paced transfer that walks its moving address and starts on write — what
    /// a buffer transfer wants.
    pub const fn new(width: DmaTransferWidth, treq: u32) -> Self {
        Self {
            width,
            treq,
            increment: true,
            trigger: true,
        }
    }

    /// Feed one address repeatedly (`increment(false)`) or walk a buffer.
    pub const fn increment(self, increment: bool) -> Self {
        Self { increment, ..self }
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

// Layout tripwires (rule §2: anything shared with the hardware carries a
// compile-time guard). Both blocks are handed to a DMA control channel as two
// adjacent 32-bit words — `(TRANS_COUNT, READ_ADDR)` for alias 3 and
// `(WRITE_ADDR, TRANS_COUNT_TRIG)` for alias 1 — so the order and the 8-byte size
// are part of the hardware contract, not an implementation detail.
//
// 32-bit targets only: the layout is what the RP2040/RP2350 DMA sees, and it is
// the only width this crate is built for. A hosted `cargo test` (64-bit pointers)
// pads the blocks to 16 bytes, which says nothing about the target.
#[cfg(target_pointer_width = "32")]
const _: () = assert!(
    core::mem::size_of::<DmaGatherBlock>() == 8
        && core::mem::offset_of!(DmaGatherBlock, trans_count) == 0
        && core::mem::offset_of!(DmaGatherBlock, read_addr) == 4,
    "DmaGatherBlock is (TRANS_COUNT, READ_ADDR) — it is written into DMA alias 3, so the order and size are fixed"
);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(
    core::mem::size_of::<DmaScatterBlock>() == 8
        && core::mem::offset_of!(DmaScatterBlock, write_addr) == 0
        && core::mem::offset_of!(DmaScatterBlock, trans_count) == 4,
    "DmaScatterBlock is (WRITE_ADDR, TRANS_COUNT) — it is written into DMA alias 1, so the order and size are fixed"
);
