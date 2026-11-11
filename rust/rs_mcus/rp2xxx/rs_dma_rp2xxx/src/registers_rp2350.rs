//! DMA register layout specific to the **RP2350**: 16 channels, a global block
//! with four interrupt groups, the shifted `CTRL_TRIG` fields and its own DREQ
//! numbers.
//!
//! Numbers are RP2350 datasheet §12.6 (Table 1148 for `CTRL_TRIG`, Table 1143 for
//! the DREQs). Selected by `registers.rs`; the parts that do not differ are in
//! `registers_common.rs`.

#![allow(dead_code)]

use super::registers::DmaChannelRegisters;

/// The RP2350 raised the DMA channel count from 12 to 16 (§12.6 "Increased the
/// number of DMA channels from 12 to 16").
pub const DMA_MAX_CHANNELS: usize = 16;

// --- CTRL_TRIG bitfields (Table 1148) ------------------------------------------
//
// Same low fields as the RP2040 (EN, HIGH_PRIORITY, DATA_SIZE, INCR_READ/WRITE,
// RING_SEL), but the RP2350 inserts INCR_*_REV at bits 5 and 7 and moves
// CHAIN_TO, RING_SIZE and TREQ_SEL up by two, so the two chips cannot share these
// constants.

pub const DMA_CTRL_EN: u32 = 1 << 0;
pub const DMA_CTRL_HIGH_PRIORITY: u32 = 1 << 1;
pub const DMA_CTRL_DATA_SIZE_8: u32 = 0 << 2;
pub const DMA_CTRL_DATA_SIZE_16: u32 = 1 << 2;
pub const DMA_CTRL_DATA_SIZE_32: u32 = 2 << 2;
pub const DMA_CTRL_DATA_SIZE_MASK: u32 = 3 << 2;
pub const DMA_CTRL_DATA_SIZE_SHIFT: u32 = 2;
pub const DMA_CTRL_INCR_READ: u32 = 1 << 4;
pub const DMA_CTRL_INCR_READ_REV: u32 = 1 << 5;
pub const DMA_CTRL_INCR_WRITE: u32 = 1 << 6;
pub const DMA_CTRL_INCR_WRITE_REV: u32 = 1 << 7;
pub const DMA_CTRL_RING_SIZE_SHIFT: u32 = 8;
pub const DMA_CTRL_RING_SIZE_MASK: u32 = 0xF << 8;
pub const DMA_CTRL_RING_SEL_WRITE: u32 = 1 << 12;
pub const DMA_CTRL_CHAIN_TO_SHIFT: u32 = 13;
pub const DMA_CTRL_CHAIN_TO_MASK: u32 = 0xF << 13;
pub const DMA_CTRL_TREQ_SHIFT: u32 = 17;
pub const DMA_CTRL_TREQ_MASK: u32 = 0x3F << 17;
pub const DMA_CTRL_IRQ_QUIET: u32 = 1 << 23;
pub const DMA_CTRL_BSWAP: u32 = 1 << 24;
pub const DMA_CTRL_SNIFF_EN: u32 = 1 << 25;
pub const DMA_CTRL_BUSY: u32 = 1 << 26;
pub const DMA_CTRL_WRITE_ERROR: u32 = 1 << 29;
pub const DMA_CTRL_READ_ERROR: u32 = 1 << 30;
pub const DMA_CTRL_AHB_ERROR: u32 = 1 << 31;

// --- DREQ numbers (Table 1143) -------------------------------------------------
//
// PIO2 sits between PIO1 and the serial peripherals on this chip, so everything
// from SPI0 onwards is 8 higher than on the RP2040.

pub const DREQ_SPI0_TX: u32 = 24;
pub const DREQ_SPI0_RX: u32 = 25;
pub const DREQ_SPI1_TX: u32 = 26;
pub const DREQ_SPI1_RX: u32 = 27;
/// `TREQ_SEL = 0x3f` is a permanent request on this chip too (§12.6.4).
pub const DREQ_PERMANENT: u32 = 0x3F;

// --- Global register block (after the 16 channels) ----------------------------

#[repr(C)]
pub struct DmaRegisters {
    pub channels: [DmaChannelRegisters; DMA_MAX_CHANNELS], // 0x000 -> 0x400
    pub intr: u32,               // 0x400 Interrupt Status (raw)
    pub inte0: u32,              // 0x404 Interrupt Enables for IRQ 0
    pub intf0: u32,              // 0x408 Force Interrupts for IRQ 0
    pub ints0: u32,              // 0x40C Interrupt Status for IRQ 0
    _reserved_410: u32,          // 0x410
    pub inte1: u32,              // 0x414 Interrupt Enables for IRQ 1
    pub intf1: u32,              // 0x418 Force Interrupts for IRQ 1
    pub ints1: u32,              // 0x41C Interrupt Status for IRQ 1
    _reserved_420: u32,          // 0x420
    pub inte2: u32,              // 0x424 Interrupt Enables for IRQ 2
    pub intf2: u32,              // 0x428 Force Interrupts for IRQ 2
    pub ints2: u32,              // 0x42C Interrupt Status for IRQ 2
    _reserved_430: u32,          // 0x430
    pub inte3: u32,              // 0x434 Interrupt Enables for IRQ 3
    pub intf3: u32,              // 0x438 Force Interrupts for IRQ 3
    pub ints3: u32,              // 0x43C Interrupt Status for IRQ 3
    _reserved_440: u32,          // 0x440
    pub timer0: u32,             // 0x444 Pacing Timer 0
    pub timer1: u32,             // 0x448 Pacing Timer 1
    pub timer2: u32,             // 0x44C Pacing Timer 2
    pub timer3: u32,             // 0x450 Pacing Timer 3
    pub multi_chan_trigger: u32, // 0x454 Multi-channel trigger
    pub sniff_ctrl: u32,         // 0x458 Sniffer Control
    _reserved_45c: u32,          // 0x45C
    pub sniff_data: u32,         // 0x460 Sniffer Data accumulator
    pub fifo_levels: u32,        // 0x464 FIFO Levels
    pub chan_abort: u32,         // 0x468 Channel Abort
}

const _: () = assert!(
    core::mem::offset_of!(DmaRegisters, channels) == 0x000
        && core::mem::size_of::<[DmaChannelRegisters; DMA_MAX_CHANNELS]>() == 0x400,
    "DmaRegisters (RP2350): 16 channel blocks fill 0x000..0x400 with no gap"
);
const _: () = assert!(
    core::mem::offset_of!(DmaRegisters, intr) == 0x400
        && core::mem::offset_of!(DmaRegisters, inte0) == 0x404
        && core::mem::offset_of!(DmaRegisters, ints0) == 0x40c
        && core::mem::offset_of!(DmaRegisters, inte1) == 0x414
        && core::mem::offset_of!(DmaRegisters, ints1) == 0x41c,
    "DmaRegisters (RP2350): INTR at +0x400, the IRQ0/IRQ1 groups at +0x404/+0x414"
);
const _: () = assert!(
    core::mem::offset_of!(DmaRegisters, inte3) == 0x434
        && core::mem::offset_of!(DmaRegisters, timer0) == 0x444
        && core::mem::offset_of!(DmaRegisters, multi_chan_trigger) == 0x454
        && core::mem::offset_of!(DmaRegisters, sniff_ctrl) == 0x458
        && core::mem::offset_of!(DmaRegisters, fifo_levels) == 0x464
        && core::mem::offset_of!(DmaRegisters, chan_abort) == 0x468,
    "DmaRegisters (RP2350): the IRQ3 group at +0x434, TIMER0..CHAN_ABORT from +0x444 to +0x468"
);
