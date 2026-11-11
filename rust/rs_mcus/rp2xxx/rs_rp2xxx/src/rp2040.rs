//! Peripheral base addresses specific to RP2040.
//!
//! Defined according to Section 2.2 of the RP2040 Datasheet.

pub use super::common::*;

// --- XIP & SRAM Memory Regions ---
pub const XIP_NOALLOC_BASE: u32 = 0x1100_0000;
pub const XIP_NOCACHE_BASE: u32 = 0x1200_0000;
pub const XIP_NOCACHE_NOALLOC_BASE: u32 = 0x1300_0000;
pub const XIP_CTRL_BASE: u32 = 0x1400_0000;
pub const XIP_SRAM_BASE: u32 = 0x1500_0000;
pub const XIP_SRAM_END: u32 = 0x1500_4000;
pub const XIP_SSI_BASE: u32 = 0x1800_0000;

pub const SRAM_STRIPED_BASE: u32 = 0x2000_0000;
pub const SRAM_STRIPED_END: u32 = 0x2004_0000;
pub const SRAM4_BASE: u32 = 0x2004_0000;
pub const SRAM5_BASE: u32 = 0x2004_1000;
pub const SRAM_END: u32 = 0x2004_2000;
pub const SRAM0_BASE: u32 = 0x2100_0000;
pub const SRAM1_BASE: u32 = 0x2101_0000;
pub const SRAM2_BASE: u32 = 0x2102_0000;
pub const SRAM3_BASE: u32 = 0x2103_0000;

// --- APB Peripherals (RP2040 4KB stride) ---
pub const SYSCFG_BASE: u32 = 0x4000_4000;
pub const CLOCKS_BASE: u32 = 0x4000_8000;
pub const RESETS_BASE: u32 = 0x4000_c000;
pub const PSM_BASE: u32 = 0x4001_0000;
pub const IO_BANK0_BASE: u32 = 0x4001_4000;
pub const IO_QSPI_BASE: u32 = 0x4001_8000;
pub const PADS_BANK0_BASE: u32 = 0x4001_c000;
pub const PADS_QSPI_BASE: u32 = 0x4002_0000;
pub const XOSC_BASE: u32 = 0x4002_4000;
pub const PLL_SYS_BASE: u32 = 0x4002_8000;
pub const PLL_USB_BASE: u32 = 0x4002_c000;
pub const BUSCTRL_BASE: u32 = 0x4003_0000;
pub const UART0_BASE: u32 = 0x4003_4000;
pub const UART1_BASE: u32 = 0x4003_8000;
pub const SPI0_BASE: u32 = 0x4003_c000;
pub const SPI1_BASE: u32 = 0x4004_0000;
pub const I2C0_BASE: u32 = 0x4004_4000;
pub const I2C1_BASE: u32 = 0x4004_8000;
pub const ADC_BASE: u32 = 0x4004_c000;
pub const PWM_BASE: u32 = 0x4005_0000;
pub const TIMER_BASE: u32 = 0x4005_4000;
pub const WATCHDOG_BASE: u32 = 0x4005_8000;
pub const RTC_BASE: u32 = 0x4005_c000;
pub const ROSC_BASE: u32 = 0x4006_0000;
pub const VREG_AND_CHIP_RESET_BASE: u32 = 0x4006_4000;
pub const TBMAN_BASE: u32 = 0x4006_c000;

// --- AHB-Lite Peripherals ---
pub const XIP_AUX_BASE: u32 = 0x5040_0000;
