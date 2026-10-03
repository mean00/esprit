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

#[inline(always)]
pub fn enable(_periph: Peripheral) {
    // To be fully implemented based on lnPeripherals::enable
    // This will route the peripheral to the correct RCU register bit (AHBEN, APB1EN, APB2EN)
    unimplemented!("Port lnPeripherals::enable to pure Rust")
}

#[inline(always)]
pub fn disable(_periph: Peripheral) {
    unimplemented!("Port lnPeripherals::disable to pure Rust")
}

#[inline(always)]
pub fn reset(_periph: Peripheral) {
    unimplemented!("Port lnPeripherals::reset to pure Rust")
}

pub fn get_clock(_periph: Peripheral) -> u32 {
    unimplemented!("Port lnPeripherals::getClock to pure Rust")
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
