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


struct PeripheralInfo {
    bus: u8,
    mask: u32,
}

fn get_periph_info(periph: Peripheral) -> Option<PeripheralInfo> {
    match periph {
        Peripheral::None => None,
        Peripheral::Spi0 => Some(PeripheralInfo { bus: 2, mask: 1 << 12 }),
        Peripheral::Spi1 => Some(PeripheralInfo { bus: 1, mask: 1 << 14 }),
        Peripheral::Spi2 => Some(PeripheralInfo { bus: 1, mask: 1 << 15 }),
        Peripheral::Uart0 => Some(PeripheralInfo { bus: 2, mask: 1 << 14 }),
        Peripheral::Uart1 => Some(PeripheralInfo { bus: 1, mask: 1 << 17 }),
        Peripheral::Uart2 => Some(PeripheralInfo { bus: 1, mask: 1 << 18 }),
        Peripheral::Uart3 => Some(PeripheralInfo { bus: 1, mask: 1 << 19 }),
        Peripheral::Uart4 => Some(PeripheralInfo { bus: 1, mask: 1 << 20 }),
        Peripheral::I2c0 => Some(PeripheralInfo { bus: 1, mask: 1 << 21 }),
        Peripheral::I2c1 => Some(PeripheralInfo { bus: 1, mask: 1 << 22 }),
        Peripheral::Can0 => Some(PeripheralInfo { bus: 1, mask: 1 << 25 }),
        Peripheral::Can1 => Some(PeripheralInfo { bus: 1, mask: 1 << 26 }),
        Peripheral::Dac => Some(PeripheralInfo { bus: 1, mask: 1 << 29 }),
        Peripheral::Pmu => Some(PeripheralInfo { bus: 1, mask: 1 << 28 }),
        Peripheral::Bkp => Some(PeripheralInfo { bus: 1, mask: 1 << 27 }),
        Peripheral::Wwdgt => Some(PeripheralInfo { bus: 1, mask: 1 << 11 }),
        Peripheral::Timer0 => Some(PeripheralInfo { bus: 2, mask: 1 << 11 }),
        Peripheral::Timer1 => Some(PeripheralInfo { bus: 1, mask: 1 << 0 }),
        Peripheral::Timer2 => Some(PeripheralInfo { bus: 1, mask: 1 << 1 }),
        Peripheral::Timer3 => Some(PeripheralInfo { bus: 1, mask: 1 << 2 }),
        Peripheral::Timer4 => Some(PeripheralInfo { bus: 1, mask: 1 << 3 }),
        Peripheral::Timer5 => Some(PeripheralInfo { bus: 1, mask: 1 << 4 }),
        Peripheral::Timer6 => Some(PeripheralInfo { bus: 1, mask: 1 << 5 }),
        Peripheral::Usb => Some(PeripheralInfo { bus: 1, mask: 1 << 23 }),
        Peripheral::Adc0 => Some(PeripheralInfo { bus: 2, mask: 1 << 9 }),
        Peripheral::Adc1 => Some(PeripheralInfo { bus: 2, mask: 1 << 10 }),
        Peripheral::GpioA => Some(PeripheralInfo { bus: 2, mask: 1 << 2 }),
        Peripheral::GpioB => Some(PeripheralInfo { bus: 2, mask: 1 << 3 }),
        Peripheral::GpioC => Some(PeripheralInfo { bus: 2, mask: 1 << 4 }),
        Peripheral::GpioD => Some(PeripheralInfo { bus: 2, mask: 1 << 5 }),
        Peripheral::GpioE => Some(PeripheralInfo { bus: 2, mask: 1 << 6 }),
        Peripheral::Afio => Some(PeripheralInfo { bus: 2, mask: 1 << 0 }),
        Peripheral::Dma0 => Some(PeripheralInfo { bus: 8, mask: 1 << 0 }),
        Peripheral::Dma1 => Some(PeripheralInfo { bus: 8, mask: 1 << 1 }),
        Peripheral::Ethernet => Some(PeripheralInfo { bus: 8, mask: 7 << 14 }),
        Peripheral::UsbHsCh32v3x => Some(PeripheralInfo { bus: 8, mask: 1 << 11 }),
        Peripheral::UsbFsOtgCh32v3x => Some(PeripheralInfo { bus: 8, mask: 1 << 12 }),
        _ => None,
    }
}

#[inline(always)]
pub fn enable(periph: Peripheral) {
    if let Some(info) = get_periph_info(periph) {
        let rcu = registers::RcuRegisters::ptr();
        unsafe {
            match info.bus {
                1 => {
                    let mut val = core::ptr::read_volatile(&mut (*rcu).apb1en);
                    val |= info.mask;
                    core::ptr::write_volatile(&mut (*rcu).apb1en, val);
                },
                2 => {
                    let mut val = core::ptr::read_volatile(&mut (*rcu).apb2en);
                    val |= info.mask;
                    core::ptr::write_volatile(&mut (*rcu).apb2en, val);
                },
                8 => {
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
                1 => {
                    let mut val = core::ptr::read_volatile(&mut (*rcu).apb1en);
                    val &= !info.mask;
                    core::ptr::write_volatile(&mut (*rcu).apb1en, val);
                },
                2 => {
                    let mut val = core::ptr::read_volatile(&mut (*rcu).apb2en);
                    val &= !info.mask;
                    core::ptr::write_volatile(&mut (*rcu).apb2en, val);
                },
                8 => {
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
                1 => {
                    let mut val = core::ptr::read_volatile(&mut (*rcu).apb1rst);
                    val |= info.mask;
                    core::ptr::write_volatile(&mut (*rcu).apb1rst, val);
                    val &= !info.mask;
                    core::ptr::write_volatile(&mut (*rcu).apb1rst, val);
                },
                2 => {
                    let mut val = core::ptr::read_volatile(&mut (*rcu).apb2rst);
                    val |= info.mask;
                    core::ptr::write_volatile(&mut (*rcu).apb2rst, val);
                    val &= !info.mask;
                    core::ptr::write_volatile(&mut (*rcu).apb2rst, val);
                },
                8 => {
                    if periph == Peripheral::Ethernet {
                        let mut val = core::ptr::read_volatile(&mut (*rcu).ahbrst);
                        val |= 1 << 14;
                        core::ptr::write_volatile(&mut (*rcu).ahbrst, val);
                        val &= !(1 << 14);
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
// truth for the peripheral input clocks: we query them, we never assume a frequency.
unsafe extern "C" {
    static _rcuClockApb1: u32;
    static _rcuClockApb2: u32;
    static SystemCoreClock: u32;
}

/// Timers on APB1 run at 2x the APB1 clock when the APB1 prescaler is not 1.
const APB1_TIMER_MULTIPLIER: u32 = 2;

/// Input clock (Hz) feeding `periph`, as computed by the RCU.
pub fn get_clock(periph: Peripheral) -> u32 {
    let (apb1, apb2, sysclk) = unsafe {
        (
            core::ptr::read_volatile(&raw const _rcuClockApb1),
            core::ptr::read_volatile(&raw const _rcuClockApb2),
            core::ptr::read_volatile(&raw const SystemCoreClock),
        )
    };
    match periph {
        Peripheral::Timer1
        | Peripheral::Timer2
        | Peripheral::Timer3
        | Peripheral::Timer4
        | Peripheral::Timer5
        | Peripheral::Timer6 => apb1 * APB1_TIMER_MULTIPLIER,
        Peripheral::Uart0
        | Peripheral::Timer0
        | Peripheral::Spi0
        | Peripheral::Afio
        | Peripheral::Adc0
        | Peripheral::Adc1
        | Peripheral::Apb2 => apb2,
        Peripheral::SysClock => sysclk,
        // APB1 peripherals (UART1..4, SPI1/2, I2C, ...)
        _ => apb1,
    }
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
