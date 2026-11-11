#![allow(dead_code)]

//! Register offsets and constants for RP2xxx RESETS, CLOCKS, and SYSINFO blocks.
//!
//! Defined according to Section 2.14 (Subsystem Resets), Section 2.15 (Clocks),
//! and Section 2.20 (Sysinfo) of the RP2040 and RP2350 datasheets.

// ===========================================================================
//  RESETS Register Offsets (from RESETS_BASE)
// ===========================================================================
pub const RESETS_RESET_OFFSET: u32 = 0x00;
pub const RESETS_WDSEL_OFFSET: u32 = 0x04;
pub const RESETS_RESET_DONE_OFFSET: u32 = 0x08;

// ===========================================================================
//  Atomic Register Aliases (RP2xxx standard: normal, XOR, SET, CLR)
// ===========================================================================
pub const ATOMIC_NORMAL_OFFSET: u32 = 0x0000;
pub const ATOMIC_XOR_OFFSET: u32 = 0x1000;
pub const ATOMIC_SET_OFFSET: u32 = 0x2000;
pub const ATOMIC_CLR_OFFSET: u32 = 0x3000;

// ===========================================================================
//  CLOCKS Register Offsets and Bitfields (from CLOCKS_BASE)
// ===========================================================================
pub const CLOCKS_CLK_SYS_CTRL_OFFSET: u32 = 0x3c;
pub const CLOCKS_CLK_PERI_CTRL_OFFSET: u32 = 0x48;
pub const CLOCKS_CLK_PERI_CTRL_ENABLE_LSB: u32 = 11;
pub const CLOCKS_CLK_PERI_CTRL_ENABLE_MASK: u32 = 1 << CLOCKS_CLK_PERI_CTRL_ENABLE_LSB;

pub const CLOCKS_WAKE_EN0_OFFSET: u32 = 0xa0;
pub const CLOCKS_WAKE_EN1_OFFSET: u32 = 0xa4;
pub const CLOCKS_SLEEP_EN0_OFFSET: u32 = 0xa8;
pub const CLOCKS_SLEEP_EN1_OFFSET: u32 = 0xac;

// ===========================================================================
//  SYSINFO Register Offsets and Bitfields (from SYSINFO_BASE)
// ===========================================================================
pub const SYSINFO_CHIP_ID_OFFSET: u32 = 0x00;
pub const SYSINFO_CHIP_ID_PART_LSB: u32 = 12;
pub const SYSINFO_CHIP_ID_PART_MASK: u32 = 0xffff;
pub const SYSINFO_CHIP_ID_PART_RP2040: u32 = 0x0002;

// ===========================================================================
//  Clock Indices for clock_get_hz
// ===========================================================================
pub const CLK_GPOUT0_INDEX: u32 = 0;
pub const CLK_GPOUT1_INDEX: u32 = 1;
pub const CLK_GPOUT2_INDEX: u32 = 2;
pub const CLK_GPOUT3_INDEX: u32 = 3;
pub const CLK_REF_INDEX: u32 = 4;
pub const CLK_SYS_INDEX: u32 = 5;
pub const CLK_PERI_INDEX: u32 = 6;
pub const CLK_USB_INDEX_RP2040: u32 = 7;
pub const CLK_ADC_INDEX_RP2040: u32 = 8;
pub const CLK_RTC_INDEX_RP2040: u32 = 9;
pub const CLK_HSTX_INDEX_RP2350: u32 = 7;
pub const CLK_USB_INDEX_RP2350: u32 = 8;
pub const CLK_ADC_INDEX_RP2350: u32 = 9;

// ===========================================================================
//  Default Clock Frequencies (Hz)
// ===========================================================================
pub const DEFAULT_SYS_CLOCK_HZ: u32 = 125_000_000;
pub const DEFAULT_PERI_CLOCK_HZ: u32 = 125_000_000;
pub const DEFAULT_USB_CLOCK_HZ: u32 = 48_000_000;
pub const DEFAULT_ADC_CLOCK_HZ: u32 = 48_000_000;
pub const DEFAULT_TIMER_CLOCK_HZ: u32 = 1_000_000;

// ===========================================================================
//  RP2040 Peripheral Reset Bitmasks (Table 202, RP2040 Datasheet 2.14.3)
// ===========================================================================
pub mod rp2040_resets {
    pub const ADC: u32 = 1 << 0;
    pub const BUSCTRL: u32 = 1 << 1;
    pub const DMA: u32 = 1 << 2;
    pub const I2C0: u32 = 1 << 3;
    pub const I2C1: u32 = 1 << 4;
    pub const IO_BANK0: u32 = 1 << 5;
    pub const IO_QSPI: u32 = 1 << 6;
    pub const JTAG: u32 = 1 << 7;
    pub const PADS_BANK0: u32 = 1 << 8;
    pub const PADS_QSPI: u32 = 1 << 9;
    pub const PIO0: u32 = 1 << 10;
    pub const PIO1: u32 = 1 << 11;
    pub const PLL_SYS: u32 = 1 << 12;
    pub const PLL_USB: u32 = 1 << 13;
    pub const PWM: u32 = 1 << 14;
    pub const RTC: u32 = 1 << 15;
    pub const SPI0: u32 = 1 << 16;
    pub const SPI1: u32 = 1 << 17;
    pub const SYSCFG: u32 = 1 << 18;
    pub const SYSINFO: u32 = 1 << 19;
    pub const TBMAN: u32 = 1 << 20;
    pub const TIMER: u32 = 1 << 21;
    pub const UART0: u32 = 1 << 22;
    pub const UART1: u32 = 1 << 23;
    pub const USBCTRL: u32 = 1 << 24;
}

// ===========================================================================
//  RP2350 Peripheral Reset Bitmasks (Table 532, RP2350 Datasheet 7.5)
// ===========================================================================
pub mod rp2350_resets {
    pub const ADC: u32 = 1 << 0;
    pub const BUSCTRL: u32 = 1 << 1;
    pub const DMA: u32 = 1 << 2;
    pub const HSTX: u32 = 1 << 3;
    pub const I2C0: u32 = 1 << 4;
    pub const I2C1: u32 = 1 << 5;
    pub const IO_BANK0: u32 = 1 << 6;
    pub const IO_QSPI: u32 = 1 << 7;
    pub const JTAG: u32 = 1 << 8;
    pub const PADS_BANK0: u32 = 1 << 9;
    pub const PADS_QSPI: u32 = 1 << 10;
    pub const PIO0: u32 = 1 << 11;
    pub const PIO1: u32 = 1 << 12;
    pub const PIO2: u32 = 1 << 13;
    pub const PLL_SYS: u32 = 1 << 14;
    pub const PLL_USB: u32 = 1 << 15;
    pub const PWM: u32 = 1 << 16;
    pub const SHA256: u32 = 1 << 17;
    pub const SPI0: u32 = 1 << 18;
    pub const SPI1: u32 = 1 << 19;
    pub const SYSCFG: u32 = 1 << 20;
    pub const SYSINFO: u32 = 1 << 21;
    pub const TBMAN: u32 = 1 << 22;
    pub const TIMER0: u32 = 1 << 23;
    pub const TIMER1: u32 = 1 << 24;
    pub const TRNG: u32 = 1 << 25;
    pub const UART0: u32 = 1 << 26;
    pub const UART1: u32 = 1 << 27;
    pub const USBCTRL: u32 = 1 << 28;
}
