//! DMA register layout specific to the **RP2040**: 12 channels, the global block
//! behind them, the `CTRL_TRIG` field layout and the DREQ numbers.
//!
//! Numbers are RP2040 datasheet §2.5.7 (Table 124 for `CTRL_TRIG`) and §2.5.3.1
//! Table 119 for the DREQs. Selected by `registers.rs`; the parts that do not
//! differ are in `registers_common.rs`.

#![allow(dead_code)]

use super::registers::DmaChannelRegisters;

/// RP2040 has 12 DMA channels (RP2350 has 16 — see `registers_rp2350.rs`).
pub const DMA_MAX_CHANNELS: usize = 12;

/// Words between the last channel block (12 × 0x40 = 0x300) and the first global
/// register (INTR at 0x400). The RP2350 has no such gap: its 16 channels reach
/// 0x400 exactly.
pub const DMA_GLOBAL_PADDING_WORDS: usize = 64;

// --- CTRL_TRIG bitfields (datasheet Table 124) --------------------------------

pub const DMA_CTRL_EN: u32 = 1 << 0;
pub const DMA_CTRL_HIGH_PRIORITY: u32 = 1 << 1;
pub const DMA_CTRL_DATA_SIZE_8: u32 = 0 << 2;
pub const DMA_CTRL_DATA_SIZE_16: u32 = 1 << 2;
pub const DMA_CTRL_DATA_SIZE_32: u32 = 2 << 2;
pub const DMA_CTRL_DATA_SIZE_MASK: u32 = 3 << 2;
pub const DMA_CTRL_DATA_SIZE_SHIFT: u32 = 2;
pub const DMA_CTRL_INCR_READ: u32 = 1 << 4;
pub const DMA_CTRL_INCR_WRITE: u32 = 1 << 5;
pub const DMA_CTRL_RING_SIZE_SHIFT: u32 = 6;
pub const DMA_CTRL_RING_SIZE_MASK: u32 = 0xF << 6;
pub const DMA_CTRL_RING_SEL_WRITE: u32 = 1 << 10;
pub const DMA_CTRL_CHAIN_TO_SHIFT: u32 = 11;
pub const DMA_CTRL_CHAIN_TO_MASK: u32 = 0xF << 11;
pub const DMA_CTRL_TREQ_SHIFT: u32 = 15;
pub const DMA_CTRL_TREQ_MASK: u32 = 0x3F << 15;
pub const DMA_CTRL_IRQ_QUIET: u32 = 1 << 21;
pub const DMA_CTRL_BSWAP: u32 = 1 << 22;
pub const DMA_CTRL_SNIFF_EN: u32 = 1 << 23;
pub const DMA_CTRL_BUSY: u32 = 1 << 24;
pub const DMA_CTRL_WRITE_ERROR: u32 = 1 << 29;
pub const DMA_CTRL_READ_ERROR: u32 = 1 << 30;
pub const DMA_CTRL_AHB_ERROR: u32 = 1 << 31;

// --- DREQ numbers (datasheet §2.5.3.1 Table 119) -------------------------------
//
// The RP2350 numbers the same peripherals differently (it has PIO2 in between),
// so these live with the chip, not with the driver: SPI0_TX is 16 here and 24
// there.

pub const DREQ_SPI0_TX: u32 = 16;
pub const DREQ_SPI0_RX: u32 = 17;
pub const DREQ_SPI1_TX: u32 = 18;
pub const DREQ_SPI1_RX: u32 = 19;
/// Permanently asserted request: an unpaced transfer (memory-to-memory, or the
/// control channel of a gather/scatter pair).
pub const DREQ_PERMANENT: u32 = 0x3F;

// --- Global register block (after the 12 channels) ----------------------------

#[repr(C)]
pub struct DmaRegisters {
    pub channels: [DmaChannelRegisters; DMA_MAX_CHANNELS],
    _reserved_padding: [u32; DMA_GLOBAL_PADDING_WORDS], // 0x300 -> 0x400
    pub intr: u32,               // 0x400 Interrupt Status (raw)
    pub inte0: u32,              // 0x404 Interrupt Enables for IRQ 0
    pub intf0: u32,              // 0x408 Force Interrupts for IRQ 0
    pub ints0: u32,              // 0x40C Interrupt Status for IRQ 0
    _reserved_irq: u32,          // 0x410
    pub inte1: u32,              // 0x414 Interrupt Enables for IRQ 1
    pub intf1: u32,              // 0x418 Force Interrupts for IRQ 1
    pub ints1: u32,              // 0x41C Interrupt Status for IRQ 1
    pub timer0: u32,             // 0x420 Pacing Timer 0
    pub timer1: u32,             // 0x424 Pacing Timer 1
    pub timer2: u32,             // 0x428 Pacing Timer 2
    pub timer3: u32,             // 0x42C Pacing Timer 3
    pub multi_chan_trigger: u32, // 0x430 Multi-channel trigger
    pub sniff_ctrl: u32,         // 0x434 Sniffer Control
    pub sniff_data: u32,         // 0x438 Sniffer Data accumulator
    _reserved_sniff: u32,        // 0x43C
    pub fifo_levels: u32,        // 0x440 FIFO Levels
    pub chan_abort: u32,         // 0x444 Channel Abort
}

const _: () = assert!(
    core::mem::offset_of!(DmaRegisters, intr) == 0x400
        && core::mem::offset_of!(DmaRegisters, inte0) == 0x404
        && core::mem::offset_of!(DmaRegisters, ints0) == 0x40c
        && core::mem::offset_of!(DmaRegisters, inte1) == 0x414
        && core::mem::offset_of!(DmaRegisters, ints1) == 0x41c,
    "DmaRegisters (RP2040): INTR at +0x400, the two IRQ groups at +0x404/+0x414"
);
const _: () = assert!(
    core::mem::offset_of!(DmaRegisters, timer0) == 0x420
        && core::mem::offset_of!(DmaRegisters, multi_chan_trigger) == 0x430
        && core::mem::offset_of!(DmaRegisters, sniff_ctrl) == 0x434
        && core::mem::offset_of!(DmaRegisters, fifo_levels) == 0x440
        && core::mem::offset_of!(DmaRegisters, chan_abort) == 0x444,
    "DmaRegisters (RP2040): TIMER0..CHAN_ABORT run from +0x420 to +0x444"
);
