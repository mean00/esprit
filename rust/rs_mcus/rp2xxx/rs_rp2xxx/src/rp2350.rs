//! Peripheral base addresses specific to RP2350.
//!
//! Defined according to Section 2.2 of the RP2350 Datasheet.

pub use super::common::*;

// --- XIP & SRAM Memory Regions ---
pub const XIP_SRAM_BASE: u32 = 0x13ff_c000;
pub const XIP_END: u32 = 0x1400_0000;
pub const XIP_NOCACHE_NOALLOC_BASE: u32 = 0x1400_0000;
pub const XIP_SRAM_END: u32 = 0x1400_0000;
pub const XIP_NOCACHE_NOALLOC_END: u32 = 0x1800_0000;
pub const XIP_MAINTENANCE_BASE: u32 = 0x1800_0000;
pub const XIP_NOCACHE_NOALLOC_NOTRANSLATE_BASE: u32 = 0x1c00_0000;
pub const XIP_NOCACHE_NOALLOC_NOTRANSLATE_END: u32 = 0x2000_0000;

pub const SRAM0_BASE: u32 = 0x2000_0000;
pub const SRAM_STRIPED_BASE: u32 = 0x2000_0000;
pub const SRAM4_BASE: u32 = 0x2004_0000;
pub const SRAM8_BASE: u32 = 0x2008_0000;
pub const SRAM_STRIPED_END: u32 = 0x2008_0000;
pub const SRAM_SCRATCH_X_BASE: u32 = 0x2008_0000;
pub const SRAM9_BASE: u32 = 0x2008_1000;
pub const SRAM_SCRATCH_Y_BASE: u32 = 0x2008_1000;
pub const SRAM_END: u32 = 0x2008_2000;

// --- APB Peripherals (RP2350 8KB/0x8000 stride) ---
pub const SYSCFG_BASE: u32 = 0x4000_8000;
pub const CLOCKS_BASE: u32 = 0x4001_0000;
pub const PSM_BASE: u32 = 0x4001_8000;
pub const RESETS_BASE: u32 = 0x4002_0000;
pub const IO_BANK0_BASE: u32 = 0x4002_8000;
pub const IO_QSPI_BASE: u32 = 0x4003_0000;
pub const PADS_BANK0_BASE: u32 = 0x4003_8000;
pub const PADS_QSPI_BASE: u32 = 0x4004_0000;
pub const XOSC_BASE: u32 = 0x4004_8000;
pub const PLL_SYS_BASE: u32 = 0x4005_0000;
pub const PLL_USB_BASE: u32 = 0x4005_8000;
pub const ACCESSCTRL_BASE: u32 = 0x4006_0000;
pub const BUSCTRL_BASE: u32 = 0x4006_8000;
pub const UART0_BASE: u32 = 0x4007_0000;
pub const UART1_BASE: u32 = 0x4007_8000;
pub const SPI0_BASE: u32 = 0x4008_0000;
pub const SPI1_BASE: u32 = 0x4008_8000;
pub const I2C0_BASE: u32 = 0x4009_0000;
pub const I2C1_BASE: u32 = 0x4009_8000;
pub const ADC_BASE: u32 = 0x400a_0000;
pub const PWM_BASE: u32 = 0x400a_8000;
pub const TIMER0_BASE: u32 = 0x400b_0000;
pub const TIMER1_BASE: u32 = 0x400b_8000;
pub const HSTX_CTRL_BASE: u32 = 0x400c_0000;
pub const XIP_CTRL_BASE: u32 = 0x400c_8000;
pub const XIP_QMI_BASE: u32 = 0x400d_0000;
pub const WATCHDOG_BASE: u32 = 0x400d_8000;
pub const BOOTRAM_BASE: u32 = 0x400e_0000;
pub const BOOTRAM_END: u32 = 0x400e_0400;
pub const ROSC_BASE: u32 = 0x400e_8000;
pub const TRNG_BASE: u32 = 0x400f_0000;
pub const SHA256_BASE: u32 = 0x400f_8000;
pub const POWMAN_BASE: u32 = 0x4010_0000;
pub const TICKS_BASE: u32 = 0x4010_8000;
pub const OTP_BASE: u32 = 0x4012_0000;
pub const OTP_DATA_BASE: u32 = 0x4013_0000;
pub const OTP_DATA_RAW_BASE: u32 = 0x4013_4000;
pub const OTP_DATA_GUARDED_BASE: u32 = 0x4013_8000;
pub const OTP_DATA_RAW_GUARDED_BASE: u32 = 0x4013_c000;
pub const CORESIGHT_PERIPH_BASE: u32 = 0x4014_0000;
pub const DFT_BASE: u32 = 0x4015_0000;
pub const GLITCH_DETECTOR_BASE: u32 = 0x4015_8000;
pub const TBMAN_BASE: u32 = 0x4016_0000;

// --- AHB-Lite Peripherals ---
pub const PIO2_BASE: u32 = 0x5040_0000;
pub const XIP_AUX_BASE: u32 = 0x5050_0000;
pub const HSTX_FIFO_BASE: u32 = 0x5060_0000;
pub const CORESIGHT_TRACE_BASE: u32 = 0x5070_0000;

// --- IOPORT & PPB Non-secure aliases ---
pub const SIO_NONSEC_BASE: u32 = 0xd002_0000;
pub const PPB_NONSEC_BASE: u32 = 0xe002_0000;
pub const EPPB_BASE: u32 = 0xe008_0000;
