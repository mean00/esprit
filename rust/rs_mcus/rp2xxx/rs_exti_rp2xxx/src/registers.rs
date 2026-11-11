//! Register map of the RP2xxx GPIO interrupts, in IO_BANK0.
//!
//! Numbers are the RP2040 datasheet §2.19.6 (Table 300) and the RP2350 datasheet
//! §9.11.2 (Tables 690-697). The block is addressed differently on the two chips
//! — `INTR0` sits at `IO_BANK0 + 0x0f0` with 4 words on the RP2040 and at
//! `IO_BANK0 + 0x230` with 6 words on the RP2350 — so everything that differs
//! lives in `registers_rp2040.rs` / `registers_rp2350.rs`.
#![allow(dead_code)]

/// IO_BANK0 base, selected per chip by `rs_rp2xxx`.
pub use rs_rp2xxx::IO_BANK0_BASE;

pub use crate::registers_chip::{GPIO_COUNT, INTR_OFFSET, IRQ_WORDS, NVIC_IO_IRQ_BANK0};

/// The four interrupt arrays of IO_BANK0 (`INTR`, `PROC0_INTE`, `PROC0_INTF`,
/// `PROC0_INTS`), one `u32` per group of 8 pads.
#[repr(C)]
pub struct GpioIrqRegisters {
    pub intr: [u32; IRQ_WORDS], // 0x0f0 / 0x230 Raw status, write 1 to clear
    pub proc0_inte: [u32; IRQ_WORDS], // 0x100 / 0x248 Interrupt enable, proc 0
    pub proc0_intf: [u32; IRQ_WORDS], // 0x110 / 0x260 Force, proc 0
    pub proc0_ints: [u32; IRQ_WORDS], // 0x120 / 0x278 Masked status (INTE & INTR)
}

const _: () = assert!(
    core::mem::offset_of!(GpioIrqRegisters, intr) == 0
        && core::mem::offset_of!(GpioIrqRegisters, proc0_inte) == size_of::<[u32; IRQ_WORDS]>()
        && core::mem::offset_of!(GpioIrqRegisters, proc0_ints) == 3 * size_of::<[u32; IRQ_WORDS]>(),
    "GpioIrqRegisters: four equal arrays, INTR first"
);

/// The block, at the chip's `INTR0` offset inside IO_BANK0.
#[inline(always)]
pub const fn registers() -> *mut GpioIrqRegisters {
    (IO_BANK0_BASE + INTR_OFFSET) as *mut GpioIrqRegisters
}

// --- Per-pad interrupt bits --------------------------------------------------
//
// Each pad owns one nibble of each array: bit 0 level-low, 1 level-high,
// 2 edge-low, 3 edge-high. The driver only ever arms edges.

/// Nibble bit: falling edge (a high-to-low transition latches it).
pub const INTE_EDGE_LOW: u32 = 1 << 2;
/// Nibble bit: rising edge.
pub const INTE_EDGE_HIGH: u32 = 1 << 3;
/// Both edges.
pub const INTE_EDGE_BOTH: u32 = INTE_EDGE_LOW | INTE_EDGE_HIGH;
/// All four bits of a pad: the write-1-to-clear value of `INTR`.
pub const INTE_MASK: u32 = 0x0f;

/// Index of the word holding `pin`.
#[inline(always)]
pub const fn word(pin: u32) -> usize {
    (pin / 8) as usize
}

/// Bit position of `pin`'s nibble inside its word.
#[inline(always)]
pub const fn shift(pin: u32) -> u32 {
    (pin % 8) * 4
}

/// Mask of `pin`'s nibble inside its word.
#[inline(always)]
pub const fn mask(pin: u32) -> u32 {
    INTE_MASK << shift(pin)
}

// EOF
