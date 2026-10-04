#![allow(unused_variables, unused_imports, unused_parens)]
#![allow(non_camel_case_types, non_snake_case, dead_code)]

#![no_std]

mod registers;


/// The peripheral identifiers, mapping to lnPeripherals.h
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

    Apb1 = 100,
    Apb2,
    SysClock,
}


use registers::*;

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
        Peripheral::UsbHsCh32v3x => Some(PeripheralInfo { bus: BUS_AHB, mask: RCU_AHB_USBHS_CH32V3X }),
        Peripheral::UsbFsOtgCh32v3x => Some(PeripheralInfo { bus: BUS_AHB, mask: RCU_AHB_USBFS_OTG_CH32V3X }),
        _ => None,
    }
}

#[inline(always)]
pub fn enable(periph: Peripheral) {
    if let Some(info) = get_periph_info(periph) {
        let rcu = registers::RcuRegisters::ptr();
        unsafe {
            match info.bus {
                BUS_APB1 => {
                    let mut val = core::ptr::read_volatile(&mut (*rcu).apb1en);
                    val |= info.mask;
                    core::ptr::write_volatile(&mut (*rcu).apb1en, val);
                },
                BUS_APB2 => {
                    let mut val = core::ptr::read_volatile(&mut (*rcu).apb2en);
                    val |= info.mask;
                    core::ptr::write_volatile(&mut (*rcu).apb2en, val);
                },
                BUS_AHB => {
                    let mut val = core::ptr::read_volatile(&mut (*rcu).ahben);
                    val |= info.mask;
                    core::ptr::write_volatile(&mut (*rcu).ahben, val);
                },
                _ => {}
            }
        }
    }
}

#[inline(always)]
pub fn disable(periph: Peripheral) {
    if let Some(info) = get_periph_info(periph) {
        let rcu = registers::RcuRegisters::ptr();
        unsafe {
            match info.bus {
                BUS_APB1 => {
                    let mut val = core::ptr::read_volatile(&mut (*rcu).apb1en);
                    val &= !info.mask;
                    core::ptr::write_volatile(&mut (*rcu).apb1en, val);
                },
                BUS_APB2 => {
                    let mut val = core::ptr::read_volatile(&mut (*rcu).apb2en);
                    val &= !info.mask;
                    core::ptr::write_volatile(&mut (*rcu).apb2en, val);
                },
                BUS_AHB => {
                    let mut val = core::ptr::read_volatile(&mut (*rcu).ahben);
                    val &= !info.mask;
                    core::ptr::write_volatile(&mut (*rcu).ahben, val);
                },
                _ => {}
            }
        }
    }
}

#[inline(always)]
pub fn reset(periph: Peripheral) {
    if let Some(info) = get_periph_info(periph) {
        let rcu = registers::RcuRegisters::ptr();
        unsafe {
            match info.bus {
                BUS_APB1 => {
                    let mut val = core::ptr::read_volatile(&mut (*rcu).apb1rst);
                    val |= info.mask;
                    core::ptr::write_volatile(&mut (*rcu).apb1rst, val);
                    val &= !info.mask;
                    core::ptr::write_volatile(&mut (*rcu).apb1rst, val);
                },
                BUS_APB2 => {
                    let mut val = core::ptr::read_volatile(&mut (*rcu).apb2rst);
                    val |= info.mask;
                    core::ptr::write_volatile(&mut (*rcu).apb2rst, val);
                    val &= !info.mask;
                    core::ptr::write_volatile(&mut (*rcu).apb2rst, val);
                },
                BUS_AHB => {
                    if periph == Peripheral::Ethernet {
                        let mut val = core::ptr::read_volatile(&mut (*rcu).ahbrst);
                        val |= RCU_AHBRST_ETHMAC;
                        core::ptr::write_volatile(&mut (*rcu).ahbrst, val);
                        val &= !RCU_AHBRST_ETHMAC;
                        core::ptr::write_volatile(&mut (*rcu).ahbrst, val);
                    }
                },
                _ => {}
            }
        }
    }
}

// The clock tree is configured by the RCU system-clock init (lnInitSystemClock), which
// records the resulting frequencies in these globals. They are the single source of
unsafe extern "C" {
    #[link_name = "\u{1}_ZN13lnPeripherals8getClockE11Peripherals"]
    fn ln_peripherals_get_clock(periph: Peripheral) -> u32;
}

/// Input clock (Hz) feeding `periph`, as computed by the RCU.
#[inline(always)]
pub fn get_clock(periph: Peripheral) -> u32 {
    unsafe { ln_peripherals_get_clock(periph) }
}


// --- Legacy Bridge (Adapter Pattern) ---
pub type Peripherals = Peripheral;

#[inline(always)]
#[allow(non_snake_case)]
pub fn lnPeripherals_enable(periph: Peripherals) {
    enable(periph);
}

#[inline(always)]
#[allow(non_snake_case)]
pub fn lnPeripherals_disable(periph: Peripherals) {
    disable(periph);
}

#[inline(always)]
#[allow(non_snake_case)]
pub fn lnPeripherals_reset(periph: Peripherals) {
    reset(periph);
}

#[inline(always)]
#[allow(non_snake_case)]
pub fn lnPeripherals_getClock(periph: Peripherals) -> u32 {
    get_clock(periph)
}
