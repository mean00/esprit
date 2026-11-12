//! RCU offsets, clock indices and defaults that are **the same on the RP2040 and
//! the RP2350**.
//!
//! `RESETS` / `CLOCKS` / `SYSINFO` sit at different addresses on the two chips, so
//! the bases come from `rs_rp2xxx`'s root re-export. The reset-bit tables, the
//! USB/ADC clock indices and the `get_reset_bit` mapping differ, so they live in
//! `registers_rp2040.rs` / `registers_rp2350.rs`, selected by `registers.rs`.
//!
//! Numbers are RP2040 datasheet §2.14 / §2.15 / §2.20 and RP2350 §7.5 / §7.6.

#![allow(dead_code)]

// --- Bases (selected per chip by rs_rp2xxx) ------------------------------------

/// Subsystem resets block.
pub use rs_rp2xxx::RESETS_BASE;
/// Clock generators / dividers block.
pub use rs_rp2xxx::CLOCKS_BASE;

// --- RESETS register offsets ---------------------------------------------------

pub const RESETS_RESET_OFFSET: u32 = 0x00;
pub const RESETS_WDSEL_OFFSET: u32 = 0x04;
pub const RESETS_RESET_DONE_OFFSET: u32 = 0x08;

// --- Atomic register aliases (the RP2xxx standard set) -------------------------

pub const ATOMIC_NORMAL_OFFSET: u32 = 0x0000;
pub const ATOMIC_XOR_OFFSET: u32 = 0x1000;
pub const ATOMIC_SET_OFFSET: u32 = 0x2000;
pub const ATOMIC_CLR_OFFSET: u32 = 0x3000;

// --- CLOCKS register offsets and fields ---------------------------------------

pub const CLOCKS_CLK_SYS_CTRL_OFFSET: u32 = 0x3c;
pub const CLOCKS_CLK_PERI_CTRL_OFFSET: u32 = 0x48;
pub const CLOCKS_CLK_PERI_CTRL_ENABLE_LSB: u32 = 11;
pub const CLOCKS_CLK_PERI_CTRL_ENABLE_MASK: u32 = 1 << CLOCKS_CLK_PERI_CTRL_ENABLE_LSB;

pub const CLOCKS_WAKE_EN0_OFFSET: u32 = 0xa0;
pub const CLOCKS_WAKE_EN1_OFFSET: u32 = 0xa4;
pub const CLOCKS_SLEEP_EN0_OFFSET: u32 = 0xa8;
pub const CLOCKS_SLEEP_EN1_OFFSET: u32 = 0xac;

// --- SYSINFO ------------------------------------------------------------------

pub const SYSINFO_CHIP_ID_OFFSET: u32 = 0x00;
pub const SYSINFO_CHIP_ID_PART_LSB: u32 = 12;
pub const SYSINFO_CHIP_ID_PART_MASK: u32 = 0xffff;
pub const SYSINFO_CHIP_ID_PART_RP2040: u32 = 0x0002;

// --- Clock indices for `clock_get_hz` (identical up to `clk_peri`) ------------
//
// The chip files add the USB / ADC indices: the RP2350 inserted `clk_hstx` at 7.

pub const CLK_GPOUT0_INDEX: u32 = 0;
pub const CLK_GPOUT1_INDEX: u32 = 1;
pub const CLK_GPOUT2_INDEX: u32 = 2;
pub const CLK_GPOUT3_INDEX: u32 = 3;
pub const CLK_REF_INDEX: u32 = 4;
pub const CLK_SYS_INDEX: u32 = 5;
pub const CLK_PERI_INDEX: u32 = 6;

// --- Fallbacks, used only when `clock_get_hz` reports 0 ------------------------

pub const DEFAULT_SYS_CLOCK_HZ: u32 = 125_000_000;
pub const DEFAULT_PERI_CLOCK_HZ: u32 = 125_000_000;
pub const DEFAULT_USB_CLOCK_HZ: u32 = 48_000_000;
pub const DEFAULT_ADC_CLOCK_HZ: u32 = 48_000_000;
pub const DEFAULT_TIMER_CLOCK_HZ: u32 = 1_000_000;
