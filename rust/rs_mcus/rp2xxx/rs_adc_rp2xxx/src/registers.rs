//! Register map of the RP2xxx ADC: one block, the same layout on both chips.
//!
//! Numbers are the RP2040 datasheet §4.9.4 (Tables 568-571) and the RP2350
//! datasheet §12.4.7 (Tables 1118-1126). The base address differs between the
//! chips (RP2040 `0x4004_c000`, RP2350 `0x400a_0000`), so it comes from
//! `rs_rp2xxx`'s root re-export; `CS.AINSEL` / `CS.RROBIN` are wider on the
//! RP2350, so those live in `registers_rp2040.rs` / `registers_rp2350.rs`.
//!
//! The map is complete on purpose — the driver only uses part of it.
#![allow(dead_code)]

/// ADC block base, selected per chip by `rs_rp2xxx`.
pub use rs_rp2xxx::ADC_BASE;

// --- The block (offsets are the same on the RP2040 and the RP2350) -----------

#[repr(C)]
pub struct AdcRegisters {
    pub cs: u32,     // 0x00 Control and status
    pub result: u32, // 0x04 Result of the most recent conversion
    pub fcs: u32,    // 0x08 FIFO control and status
    pub fifo: u32,   // 0x0c Conversion result FIFO
    pub div: u32,    // 0x10 Clock divider
    pub intr: u32,   // 0x14 Raw interrupts
    pub inte: u32,   // 0x18 Interrupt enable
    pub intf: u32,   // 0x1c Interrupt force
    pub ints: u32,   // 0x20 Interrupt status after masking and forcing
}

/// Offset of the result FIFO — the source address of the DMA transfer.
const ADC_FIFO_OFFSET: u32 = 0x0c;
/// Offset of the last register, i.e. one past the end of the block.
const ADC_BLOCK_END: u32 = 0x24;

const _: () = assert!(
    core::mem::offset_of!(AdcRegisters, cs) == 0x00
        && core::mem::offset_of!(AdcRegisters, result) == 0x04
        && core::mem::offset_of!(AdcRegisters, fcs) == 0x08
        && core::mem::offset_of!(AdcRegisters, fifo) == ADC_FIFO_OFFSET as usize
        && core::mem::offset_of!(AdcRegisters, div) == 0x10
        && core::mem::offset_of!(AdcRegisters, ints) == 0x20
        && core::mem::size_of::<AdcRegisters>() == ADC_BLOCK_END as usize,
    "AdcRegisters: CS/RESULT/FCS/FIFO/DIV at 0x00/0x04/0x08/0x0c/0x10, INTS at 0x20"
);

/// Address the DMA reads from: the FIFO pops one 16-bit sample per read.
#[inline(always)]
pub const fn fifo_address() -> u32 {
    ADC_BASE + core::mem::offset_of!(AdcRegisters, fifo) as u32
}

// --- CS (0x00): control and status ------------------------------------------

pub const ADC_CS_EN: u32 = 1 << 0;
/// Power on the internal temperature sensor.
pub const ADC_CS_TS_EN: u32 = 1 << 1;
/// Start one conversion (self-clearing).
pub const ADC_CS_START_ONCE: u32 = 1 << 2;
/// Free-run: a new conversion starts as soon as the previous one finishes.
pub const ADC_CS_START_MANY: u32 = 1 << 3;
/// `1` when the ADC can start a new conversion (so: no conversion in progress).
pub const ADC_CS_READY: u32 = 1 << 8;
/// The most recent conversion failed to converge; its result is unusable.
pub const ADC_CS_ERR: u32 = 1 << 9;
/// A past conversion failed; write 1 to clear.
pub const ADC_CS_ERR_STICKY: u32 = 1 << 10;

// --- RESULT (0x04) and FIFO (0x0c) ------------------------------------------

pub const ADC_VALUE_MASK: u32 = 0x0fff;
/// FIFO only: this sample's conversion failed.
pub const ADC_FIFO_ERROR: u32 = 1 << 15;

// --- FCS (0x08): FIFO control and status ------------------------------------

pub const ADC_FCS_EN: u32 = 1 << 0;
/// Shift each result down to 8 bits (only wanted for byte-wide DMA).
pub const ADC_FCS_SHIFT: u32 = 1 << 1;
/// Put the conversion-error bit into the FIFO entry.
pub const ADC_FCS_ERR: u32 = 1 << 2;
/// Assert DREQ while the FIFO holds at least `THRESH` samples.
pub const ADC_FCS_DREQ_EN: u32 = 1 << 3;
pub const ADC_FCS_EMPTY: u32 = 1 << 8;
pub const ADC_FCS_FULL: u32 = 1 << 9;
/// Read, not write: the FIFO was read while empty. Write 1 to clear.
pub const ADC_FCS_UNDER: u32 = 1 << 10;
/// The FIFO overflowed, i.e. a sample was lost. Write 1 to clear.
pub const ADC_FCS_OVER: u32 = 1 << 11;
pub const ADC_FCS_LEVEL_LSB: u32 = 16;
pub const ADC_FCS_LEVEL_MASK: u32 = 0xf << ADC_FCS_LEVEL_LSB;
pub const ADC_FCS_THRESH_LSB: u32 = 24;
pub const ADC_FCS_THRESH_MASK: u32 = 0xf << ADC_FCS_THRESH_LSB;

/// `THRESH` for a DMA transfer: hand every single sample over as it lands, so
/// the 8-entry FIFO cannot overflow at the configured rate (§4.9.2.5).
pub const ADC_FCS_DREQ_THRESHOLD: u32 = 1;

/// Samples the result FIFO can hold (§4.9.2.4).
pub const ADC_FIFO_DEPTH: u32 = 8;

// --- DIV (0x10): sampling rate ----------------------------------------------

/// Fractional part of the divider, in 1/256 of a `clk_adc` cycle.
pub const ADC_DIV_FRAC_LSB: u32 = 0;
pub const ADC_DIV_FRAC_MASK: u32 = 0xff;
/// Integer part of the divider, in `clk_adc` cycles.
pub const ADC_DIV_INT_LSB: u32 = 8;
pub const ADC_DIV_INT_MASK: u32 = 0xff << ADC_DIV_INT_LSB;
/// Largest divider: 255 + 255/256 cycles between two conversions.
pub const ADC_DIV_MAX: u32 = 0xffff;

// --- INTR / INTE / INTF / INTS (0x14 .. 0x20) -------------------------------

/// The FIFO reached its threshold.
pub const ADC_INT_FIFO: u32 = 1 << 0;
