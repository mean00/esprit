//! Common IP base addresses shared across the RP2xxx family (RP2040 and RP2350).
//!
//! Defined according to Section 2.2 of the RP2040 datasheet and RP2350 address map.

// --- System & Core Peripherals ---
pub const ROM_BASE: u32 = 0x0000_0000;
pub const XIP_BASE: u32 = 0x1000_0000;
pub const SRAM_BASE: u32 = 0x2000_0000;
pub const SYSINFO_BASE: u32 = 0x4000_0000;

// --- AHB-Lite Peripherals ---
pub const DMA_BASE: u32 = 0x5000_0000;
pub const USBCTRL_BASE: u32 = 0x5010_0000;
pub const USBCTRL_DPRAM_BASE: u32 = 0x5010_0000;
pub const USBCTRL_REGS_BASE: u32 = 0x5011_0000;
pub const PIO0_BASE: u32 = 0x5020_0000;
pub const PIO1_BASE: u32 = 0x5030_0000;

// --- Single-cycle IO (IOPORT) & PPB ---
pub const SIO_BASE: u32 = 0xd000_0000;
pub const PPB_BASE: u32 = 0xe000_0000;
