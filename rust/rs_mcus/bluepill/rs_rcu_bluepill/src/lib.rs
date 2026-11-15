#![allow(non_camel_case_types, non_snake_case)]

#![no_std]

mod registers;

// Family-specific bits of the AHB map: CH32V3 decodes `AHBENR` 9..12 for
// RNG/SDIO/USBHS/USB-FS-OTG, the ARM parts leave them reserved. One selection
// point; `ch32v3x` wins if both features are set (see `rs_bluepill`).
#[cfg(feature = "ch32v3x")]
#[path = "registers_ch32v3x.rs"]
mod registers_chip;

#[cfg(not(feature = "ch32v3x"))]
#[path = "registers_stm32.rs"]
mod registers_chip;

pub use rs_mcu_traits::rcu::Rcu as RcuTrait;

use registers::*;
use registers_chip::*;

rs_mcu_traits::define_mmio_accessors!(RcuRegisters::ptr());

/// C entries for the C++ side that still calls the driver (`drop_C.md` P6).
pub mod c_api;

#[cfg(feature = "ch32v3x")]
pub mod sysclk_ch32v3x;
#[cfg(feature = "ch32v3x")]
pub mod usb_ch32v3x;
#[cfg(not(feature = "ch32v3x"))]
pub mod sysclk_gd32fx;
#[cfg(not(feature = "ch32v3x"))]
pub mod usb_stm_gd32;

struct PeripheralInfo {
    bus: u8,
    mask: u32,
}

fn get_periph_info(periph: Peripheral) -> Option<PeripheralInfo> {
    match periph {
        Peripheral::None => None,
        Peripheral::Spi0 => Some(PeripheralInfo { bus: BUS_APB2, mask: RCU_APB2_SPI0 }),
        Peripheral::Spi1 => Some(PeripheralInfo { bus: BUS_APB1, mask: RCU_APB1_SPI1 }),
        Peripheral::Spi2 => Some(PeripheralInfo { bus: BUS_APB1, mask: RCU_APB1_SPI2 }),
        Peripheral::Uart0 => Some(PeripheralInfo { bus: BUS_APB2, mask: RCU_APB2_USART0 }),
        Peripheral::Uart1 => Some(PeripheralInfo { bus: BUS_APB1, mask: RCU_APB1_USART1 }),
        Peripheral::Uart2 => Some(PeripheralInfo { bus: BUS_APB1, mask: RCU_APB1_USART2 }),
        Peripheral::Uart3 => Some(PeripheralInfo { bus: BUS_APB1, mask: RCU_APB1_USART3 }),
        Peripheral::Uart4 => Some(PeripheralInfo { bus: BUS_APB1, mask: RCU_APB1_USART4 }),
        Peripheral::I2c0 => Some(PeripheralInfo { bus: BUS_APB1, mask: RCU_APB1_I2C0 }),
        Peripheral::I2c1 => Some(PeripheralInfo { bus: BUS_APB1, mask: RCU_APB1_I2C1 }),
        Peripheral::Can0 => Some(PeripheralInfo { bus: BUS_APB1, mask: RCU_APB1_CAN0 }),
        Peripheral::Can1 => Some(PeripheralInfo { bus: BUS_APB1, mask: RCU_APB1_CAN1 }),
        Peripheral::Dac => Some(PeripheralInfo { bus: BUS_APB1, mask: RCU_APB1_DAC }),
        Peripheral::Pmu => Some(PeripheralInfo { bus: BUS_APB1, mask: RCU_APB1_PMU }),
        Peripheral::Bkp => Some(PeripheralInfo { bus: BUS_APB1, mask: RCU_APB1_BKPI }),
        Peripheral::Wwdgt => Some(PeripheralInfo { bus: BUS_APB1, mask: RCU_APB1_WWDGT }),
        Peripheral::Timer0 => Some(PeripheralInfo { bus: BUS_APB2, mask: RCU_APB2_TIMER0 }),
        Peripheral::Timer1 => Some(PeripheralInfo { bus: BUS_APB1, mask: RCU_APB1_TIMER1 }),
        Peripheral::Timer2 => Some(PeripheralInfo { bus: BUS_APB1, mask: RCU_APB1_TIMER2 }),
        Peripheral::Timer3 => Some(PeripheralInfo { bus: BUS_APB1, mask: RCU_APB1_TIMER3 }),
        Peripheral::Timer4 => Some(PeripheralInfo { bus: BUS_APB1, mask: RCU_APB1_TIMER4 }),
        Peripheral::Timer5 => Some(PeripheralInfo { bus: BUS_APB1, mask: RCU_APB1_TIMER5 }),
        Peripheral::Timer6 => Some(PeripheralInfo { bus: BUS_APB1, mask: RCU_APB1_TIMER6 }),
        Peripheral::Usb => Some(PeripheralInfo { bus: BUS_APB1, mask: RCU_APB1_USBD }),
        Peripheral::Adc0 => Some(PeripheralInfo { bus: BUS_APB2, mask: RCU_APB2_ADC0 }),
        Peripheral::Adc1 => Some(PeripheralInfo { bus: BUS_APB2, mask: RCU_APB2_ADC1 }),
        Peripheral::GpioA => Some(PeripheralInfo { bus: BUS_APB2, mask: RCU_APB2_PA }),
        Peripheral::GpioB => Some(PeripheralInfo { bus: BUS_APB2, mask: RCU_APB2_PB }),
        Peripheral::GpioC => Some(PeripheralInfo { bus: BUS_APB2, mask: RCU_APB2_PC }),
        Peripheral::GpioD => Some(PeripheralInfo { bus: BUS_APB2, mask: RCU_APB2_PD }),
        Peripheral::GpioE => Some(PeripheralInfo { bus: BUS_APB2, mask: RCU_APB2_PE }),
        Peripheral::Afio => Some(PeripheralInfo { bus: BUS_APB2, mask: RCU_APB2_AF }),
        Peripheral::Dma0 => Some(PeripheralInfo { bus: BUS_AHB, mask: RCU_AHB_DMA0 }),
        Peripheral::Dma1 => Some(PeripheralInfo { bus: BUS_AHB, mask: RCU_AHB_DMA1 }),
        Peripheral::Ethernet => Some(PeripheralInfo { bus: BUS_AHB, mask: RCU_AHB_ETHMAC }),
        Peripheral::UsbHsCh32v3x => AHB_USBHS.map(|mask| PeripheralInfo { bus: BUS_AHB, mask }),
        Peripheral::UsbFsOtgCh32v3x => {
            AHB_USBFS_OTG.map(|mask| PeripheralInfo { bus: BUS_AHB, mask })
        }
        _ => None,
    }
}

#[inline(always)]
pub fn enable(periph: Peripheral) {
    if matches!(periph, Peripheral::Adc0 | Peripheral::Adc1) {
        // Divide APB2 by 8 for the SAR ADC (reset default 00 is /2 = 72 MHz at
        // 144 MHz sysclk, well above the 14 MHz CH32V307 / 40 MHz GD32F303 limit).
        unsafe { modify_reg!(cfg0, |v| (v & !LN_RCU_CFG0_ADCPSC_MASK) | LN_RCU_CFG0_ADCPSC_DIV8) };
    }
    if let Some(info) = get_periph_info(periph) {
        unsafe {
            match info.bus {
                BUS_APB1 => modify_reg!(apb1en, |v| v | info.mask),
                BUS_APB2 => modify_reg!(apb2en, |v| v | info.mask),
                BUS_AHB => modify_reg!(ahben, |v| v | info.mask),
                _ => {}
            }
        }
    }
}

#[inline(always)]
pub fn disable(periph: Peripheral) {
    if let Some(info) = get_periph_info(periph) {
        unsafe {
            match info.bus {
                BUS_APB1 => modify_reg!(apb1en, |v| v & !info.mask),
                BUS_APB2 => modify_reg!(apb2en, |v| v & !info.mask),
                BUS_AHB => modify_reg!(ahben, |v| v & !info.mask),
                _ => {}
            }
        }
    }
}

#[inline(always)]
pub fn reset(periph: Peripheral) {
    if let Some(info) = get_periph_info(periph) {
        unsafe {
            match info.bus {
                BUS_APB1 => {
                    modify_reg!(apb1rst, |v| v | info.mask);
                    modify_reg!(apb1rst, |v| v & !info.mask);
                }
                BUS_APB2 => {
                    modify_reg!(apb2rst, |v| v | info.mask);
                    modify_reg!(apb2rst, |v| v & !info.mask);
                }
                BUS_AHB => {
                    if periph == Peripheral::Ethernet {
                        modify_reg!(ahbrst, |v| v | RCU_AHBRST_ETHMAC);
                        modify_reg!(ahbrst, |v| v & !RCU_AHBRST_ETHMAC);
                    }
                }
                _ => {}
            }
        }
    }
}

// The clock tree is configured by the RCU system-clock init (lnInitSystemClock), which
// records the resulting frequencies in these globals. They are the single source of
/// Input clock (Hz) feeding `periph`, as computed by the RCU.
#[inline(always)]
pub fn get_clock(periph: Peripheral) -> u32 {
    #[cfg(feature = "ch32v3x")]
    {
        sysclk_ch32v3x::get_clock_c(periph)
    }
    #[cfg(not(feature = "ch32v3x"))]
    {
        sysclk_gd32fx::get_clock_c(periph)
    }
}


/// Idiomatic struct interface for Reset and Clock Unit (RCU/RCC).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Rcu;

/// The peripherals of the GD32F3 / CH32V3 family that have a reset gate and/or a
/// clock of their own.
///
/// This enum belongs to the *family*, not to the contract: `rs_mcu_traits::rcu::Rcu`
/// has no idea what a `UsbHsCh32v3x` is (see `supertraits.md` §2, rule R2).
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Peripheral {
    None = 0,
    Spi0,
    Spi1,
    Spi2,
    Uart0,
    Uart1,
    Uart2,
    Uart3,
    Uart4,
    I2c0,
    I2c1,
    Can0,
    Can1,
    Dac,
    Pmu,
    Bkp,
    Wwdgt,
    Timer0,
    Timer1,
    Timer2,
    Timer3,
    Timer4,
    Timer5,
    Timer6,
    Usb,
    Adc0,
    Adc1,
    GpioA,
    GpioB,
    GpioC,
    GpioD,
    GpioE,
    Afio,
    Dma0,
    Dma1,
    Ethernet,
    UsbHsCh32v3x,
    UsbFsOtgCh32v3x,

    /// Clock-only entries: they have a frequency but no reset bit.
    Apb1 = 100,
    Apb2,
    SysClock,
}

impl rs_mcu_traits::rcu::Rcu for Rcu {
    type Peripheral = Peripheral;

    #[inline(always)]
    fn enable(periph: Peripheral) {
        enable(periph);
    }

    #[inline(always)]
    fn disable(periph: Peripheral) {
        disable(periph);
    }

    #[inline(always)]
    fn reset(periph: Peripheral) {
        reset(periph);
    }

    #[inline(always)]
    fn clock_hz(periph: Peripheral) -> u32 {
        get_clock(periph)
    }
}

impl Rcu {
    #[inline(always)]
    pub fn enable(periph: Peripheral) {
        enable(periph);
    }

    #[inline(always)]
    pub fn disable(periph: Peripheral) {
        disable(periph);
    }

    #[inline(always)]
    pub fn reset(periph: Peripheral) {
        reset(periph);
    }

    #[inline(always)]
    pub fn get_clock(periph: Peripheral) -> u32 {
        get_clock(periph)
    }
}

pub mod shim;
pub use shim::*;

/// Parse a decimal `u32` from an optional compile-time env string, falling back to `default`.
pub(crate) const fn parse_u32(s: Option<&str>, default: u32) -> u32 {
    let Some(s) = s else { return default };
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return default;
    }
    let mut val = 0u32;
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b < b'0' || b > b'9' {
            return default;
        }
        val = val * 10 + (b - b'0') as u32;
        i += 1;
    }
    val
}
