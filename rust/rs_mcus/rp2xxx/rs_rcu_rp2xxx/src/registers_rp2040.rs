//! RCU values specific to the **RP2040**: its reset-bit table, the USB/ADC clock
//! indices and the `Peripheral` → reset-bit mapping.
//!
//! Tables are RP2040 datasheet §2.14.3 Table 202 and §2.15. Selected by
//! `registers.rs`.

#![allow(dead_code)]

use super::Peripheral;

/// `CLOCKS` index of `clk_usb` on this chip (`clk_sys`+1; the RP2350 puts
/// `clk_hstx` where the RP2040 has `clk_usb`).
pub const CLK_USB_INDEX: u32 = 7;
/// `CLOCKS` index of `clk_adc`.
pub const CLK_ADC_INDEX: u32 = 8;

// --- Peripheral reset bitmasks (Table 202, §2.14.3) ----------------------------

pub mod resets {
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

/// The RP2040 has one `TIMER` block, so both timer peripherals map to it.
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
        Peripheral::Timer0 | Peripheral::Timer1 => Some(resets::TIMER),
        Peripheral::Usb => Some(resets::USBCTRL),
        Peripheral::GpioA | Peripheral::GpioB | Peripheral::GpioC | Peripheral::GpioD
        | Peripheral::GpioE => Some(resets::IO_BANK0),
        _ => None,
    }
}
