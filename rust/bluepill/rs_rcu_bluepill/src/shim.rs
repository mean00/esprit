use crate::*;

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
