//! DMA register layout that is **the same on the RP2040 and the RP2350**.
//!
//! One channel is a 0x40 byte block made of four alias groups of four words, and
//! the `RESETS` side (offset of the reset register, the DMA reset bit) does not
//! move either. Numbers are RP2040 datasheet §2.5.7 / §2.14.3; the RP2350 keeps
//! this part of the block compatible (§12.6.2).
//!
//! What differs lives in the chip file, **not** in `#[cfg]`s sprinkled over the
//! driver: `registers_rp2040.rs` / `registers_rp2350.rs` (selected once, in
//! `lib.rs`) hold the channel count, the global block behind the channels, the
//! `CTRL_TRIG` field layout — the RP2350 moves `CHAIN_TO`, `RING_SIZE` and
//! `TREQ_SEL` up by two bits — and the DREQ numbers.

#![allow(dead_code)]

// --- Shared addresses ----------------------------------------------------------

/// Base of the DMA block (same on both chips).
pub use rs_rp2xxx::common::DMA_BASE;
/// `RESETS` block, selected per chip by `rs_rp2xxx`'s root re-export.
pub use rs_rp2xxx::RESETS_BASE;

// --- Subsystem reset registers (RP2040 §2.14, RP2350 §7.5) --------------------

pub const RESETS_RESET_OFFSET: u32 = 0x00;
pub const RESETS_RESET_DONE_OFFSET: u32 = 0x08;
pub const RESETS_RESET_ADDR: u32 = RESETS_BASE + RESETS_RESET_OFFSET;
pub const RESETS_RESET_DONE_ADDR: u32 = RESETS_BASE + RESETS_RESET_DONE_OFFSET;

/// `RESETS.RESET.DMA` — bit 2 on both chips (RP2040 Table 202, RP2350 Table 532).
pub const RESET_BIT_DMA: u32 = 1 << 2;

// --- One DMA channel (RP2040 §2.5.7, same layout on the RP2350) ----------------

/// Hardware register block for a single DMA channel (0x40 bytes).
///
/// Four alias groups of four words each; writing the last word of a group is what
/// triggers the channel, which is why the order here *is* the hardware contract.
#[repr(C)]
pub struct DmaChannelRegisters {
    // Alias 0 (standard access, triggers on CTRL_TRIG)
    pub read_addr: u32,
    pub write_addr: u32,
    pub trans_count: u32,
    pub ctrl_trig: u32,

    // Alias 1 (triggers on TRANS_COUNT_TRIG)
    pub al1_ctrl: u32,
    pub al1_read_addr: u32,
    pub al1_write_addr: u32,
    pub al1_trans_count_trig: u32,

    // Alias 2 (triggers on WRITE_ADDR_TRIG)
    pub al2_ctrl: u32,
    pub al2_trans_count: u32,
    pub al2_read_addr: u32,
    pub al2_write_addr_trig: u32,

    // Alias 3 (triggers on READ_ADDR_TRIG)
    pub al3_ctrl: u32,
    pub al3_write_addr: u32,
    pub al3_trans_count: u32,
    pub al3_read_addr_trig: u32,
}

// Layout tripwire per channel: the alias groups must stay on their 0x10 boundaries,
// because a control channel writes into them by offset.
const _: () = assert!(
    core::mem::offset_of!(DmaChannelRegisters, read_addr) == 0x00
        && core::mem::offset_of!(DmaChannelRegisters, ctrl_trig) == 0x0c
        && core::mem::offset_of!(DmaChannelRegisters, al1_ctrl) == 0x10
        && core::mem::offset_of!(DmaChannelRegisters, al2_ctrl) == 0x20
        && core::mem::offset_of!(DmaChannelRegisters, al3_ctrl) == 0x30
        && core::mem::size_of::<DmaChannelRegisters>() == 0x40,
    "DmaChannelRegisters: four alias groups at +0x00/+0x10/+0x20/+0x30, 0x40 bytes per channel (RP2040 §2.5.7)"
);
