//! Hardware register layout and bit definitions for the RP2040 SPI controller (PL022).
//!
//! Defined according to Section 4.4.4 of the RP2040 Datasheet.

#![allow(dead_code)]

/// Macro to calculate the byte offset of a struct field.
#[macro_export]
macro_rules! spi_reg_offset {
    ($type:ty, $field:ident) => {
        core::mem::offset_of!($type, $field) as u32
    };
}

/// Hardware register block for an RP2040 SPI instance (ARM PrimeCell PL022).
#[repr(C)]
pub struct SpiRegisters {
    pub sspcr0: u32,   // 0x000 Control register 0
    pub sspcr1: u32,   // 0x004 Control register 1
    pub sspdr: u32,    // 0x008 Data register (TX/RX FIFO)
    pub sspsr: u32,    // 0x00C Status register
    pub sspcpsr: u32,  // 0x010 Clock prescale register
    pub sspimsc: u32,  // 0x014 Interrupt mask set or clear
    pub sspris: u32,   // 0x018 Raw interrupt status
    pub sspmis: u32,   // 0x01C Masked interrupt status
    pub sspicr: u32,   // 0x020 Interrupt clear
    pub sspdmacr: u32, // 0x024 DMA control register
}

// Layout tripwires: the PL022 block is ten consecutive words from 0x000 to
// 0x024 (RP2040 datasheet §4.4.4); a reordered or padded struct fails the build.
const _: () = assert!(
    core::mem::offset_of!(SpiRegisters, sspcr0) == 0x000
        && core::mem::offset_of!(SpiRegisters, sspcr1) == 0x004
        && core::mem::offset_of!(SpiRegisters, sspdr) == 0x008
        && core::mem::offset_of!(SpiRegisters, sspsr) == 0x00c
        && core::mem::offset_of!(SpiRegisters, sspcpsr) == 0x010,
    "SpiRegisters: SSPCR0/SSPCR1/SSPDR/SSPSR/SSPCPSR are at +0x000..+0x010 (RP2040 datasheet §4.4.4)"
);
const _: () = assert!(
    core::mem::offset_of!(SpiRegisters, sspimsc) == 0x014
        && core::mem::offset_of!(SpiRegisters, sspris) == 0x018
        && core::mem::offset_of!(SpiRegisters, sspmis) == 0x01c
        && core::mem::offset_of!(SpiRegisters, sspicr) == 0x020
        && core::mem::offset_of!(SpiRegisters, sspdmacr) == 0x024
        && core::mem::size_of::<SpiRegisters>() == 0x028,
    "SpiRegisters: SSPIMSC..SSPDMACR are at +0x014..+0x024 and the block is 0x28 bytes"
);

impl SpiRegisters {
    #[inline(always)]
    pub fn ptr(instance: u32) -> *mut Self {
        if instance == 0 {
            SPI0_BASE as *mut Self
        } else {
            SPI1_BASE as *mut Self
        }
    }
}

// --- Base Addresses re-exported from rs_rp2xxx ---
//
// `SPI0_BASE` / `SPI1_BASE` differ between the chips (RP2040 `0x4003_c000` /
// `0x4004_0000`, RP2350 `0x4008_0000` / `0x4008_8000`), so they come from
// `rs_rp2xxx`'s root re-export: the chip is selected in `rs_rp2xxx/src/lib.rs`,
// once. The PL022 block *layout* below is the same on both chips, so it stays
// here.
pub use rs_rp2xxx::{SPI0_BASE, SPI1_BASE};

// --- Base-derived Register Addresses ---
pub const SPI0_SSPDR_ADDR: u32 = SPI0_BASE + spi_reg_offset!(SpiRegisters, sspdr);
pub const SPI1_SSPDR_ADDR: u32 = SPI1_BASE + spi_reg_offset!(SpiRegisters, sspdr);

// --- Register Bitfields ---

// SSPCR0 Bitfields
pub const SSPCR0_DSS_8BIT: u32 = 0x07;
pub const SSPCR0_DSS_16BIT: u32 = 0x0F;
pub const SSPCR0_DSS_MASK: u32 = 0x0F;
pub const SSPCR0_FRF_MOTOROLA: u32 = 0 << 4;
pub const SSPCR0_SPO_SHIFT: u32 = 6;
pub const SSPCR0_SPO_MASK: u32 = 1 << 6;
pub const SSPCR0_SPH_SHIFT: u32 = 7;
pub const SSPCR0_SPH_MASK: u32 = 1 << 7;
pub const SSPCR0_SCR_SHIFT: u32 = 8;
pub const SSPCR0_SCR_MASK: u32 = 0xFF << 8;

// SSPCR1 Bitfields
pub const SSPCR1_LBM: u32 = 1 << 0;
pub const SSPCR1_SSE: u32 = 1 << 1;
pub const SSPCR1_MS_SLAVE: u32 = 1 << 2;
pub const SSPCR1_SOD: u32 = 1 << 3;

// Clock divider limits (datasheet §4.4.4: `SSPCPSR.CPSDVSR` and `SSPCR0.SCR`).
/// `CPSDVSR` is 2..=254 and **even** — the PL022 divides by it, so an odd value
/// is rounded down by the hardware.
pub const CPSDVSR_MIN: u32 = 2;
/// Highest `CPSDVSR`, the reason the post-divider has to carry part of the ratio.
pub const CPSDVSR_MAX: u32 = 254;
/// `SCR` post-divides by `SCR + 1`, so the field is 0..=255.
pub const SCR_MIN: i32 = 0;
pub const SCR_MAX: i32 = 255;
/// Number of steps `SCR + 1` can take; the prescaler is aimed at half of them so
/// both halves of the divider stay near mid-scale.
pub const SCR_STEPS: u32 = 256;

// SSPSR Bitfields
pub const SSPSR_TFE: u32 = 1 << 0; // Transmit FIFO Empty
pub const SSPSR_TNF: u32 = 1 << 1; // Transmit FIFO Not Full
pub const SSPSR_RNE: u32 = 1 << 2; // Receive FIFO Not Empty
pub const SSPSR_RFF: u32 = 1 << 3; // Receive FIFO Full
pub const SSPSR_BSY: u32 = 1 << 4; // PrimeCell SSP Busy Flag

// SSPICR Bitfields
pub const SSPICR_RORIC: u32 = 1 << 0; // Clear Receive Overrun interrupt
pub const SSPICR_RTIC: u32 = 1 << 1;  // Clear Receive Timeout interrupt

// SSPDMACR Bitfields
pub const SSPDMACR_RXDMAE: u32 = 1 << 0; // Receive DMA Enable
pub const SSPDMACR_TXDMAE: u32 = 1 << 1; // Transmit DMA Enable
