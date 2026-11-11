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
pub use rs_rp2xxx::rp2040::{SPI0_BASE, SPI1_BASE};

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
