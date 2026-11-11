//! RCU values specific to the **RP2350**: its reset-bit table, the USB/ADC clock
//! indices and the `Peripheral` → reset-bit mapping.
//!
//! Tables are RP2350 datasheet §7.5 Table 532 and §7.6. Selected by
//! `registers.rs`.

#![allow(dead_code)]

use super::Peripheral;

/// `CLOCKS` index of `clk_usb` on this chip — one higher than on the RP2040, which
/// is where `clk_hstx` sits here.
pub const CLK_USB_INDEX: u32 = 8;
/// `CLOCKS` index of `clk_adc`.
pub const CLK_ADC_INDEX: u32 = 9;

// --- Peripheral reset bitmasks (Table 532, §7.5) -------------------------------

pub mod resets {
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

/// This chip has `TIMER0` and `TIMER1` as separate blocks.
pub fn get_reset_bit(periph: Peripheral) -> Option<u32> {
    match periph {
        Peripheral::Spi0 => Some(resets::SPI0),
        Peripheral::Spi1 => Some(resets::SPI1),
        Peripheral::Dma0 | Peripheral::Dma1 => Some(resets::DMA),
        Peripheral::Adc0 | Peripheral::Adc1 => Some(resets::ADC),
        Peripheral::I2c0 => Some(resets::I2C0),
        Peripheral::I2c1 => Some(resets::I2C1),
        Peripheral::Uart0 => Some(resets::UART0),
        Peripheral::Uart1 => Some(resets::UART1),
        Peripheral::Timer0 => Some(resets::TIMER0),
        Peripheral::Timer1 => Some(resets::TIMER1),
        Peripheral::Usb => Some(resets::USBCTRL),
        Peripheral::GpioA | Peripheral::GpioB | Peripheral::GpioC | Peripheral::GpioD
        | Peripheral::GpioE => Some(resets::IO_BANK0),
        _ => None,
    }
}
