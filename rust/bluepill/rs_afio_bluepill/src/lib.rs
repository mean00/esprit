#![allow(unused_variables, unused_imports, unused_parens)]
#![allow(non_camel_case_types, non_snake_case, dead_code)]

#![no_std]
pub mod registers;

use core::ptr::{read_volatile, write_volatile};
use registers::*;
use rs_rcu_bluepill::{Peripheral, enable};

/// Ensure AFIO clock is enabled.
pub fn enable_afio() {
    enable(Peripheral::Afio);
}

/// Configure SWJ to SWD enabled and JTAG disabled (freeing PA15, PB3, PB4).
pub fn release_jtag_pins() {
    enable_afio();
    let afio = AfioRegisters::ptr();
    unsafe {
        let mut pcf0 = read_volatile(&mut (*afio).pcf0);
        pcf0 &= !AFIO_PCF0_SWJ_CFG_MASK;
        pcf0 |= AFIO_PCF0_SWJ_CFG_SWD_NO_JTAG;
        write_volatile(&mut (*afio).pcf0, pcf0);
    }
}

/// Apply partial remap to Timer 2 (TIM3): PB4 = CH1, PB5 = CH2.
pub fn remap_timer2_partial() {
    enable_afio();
    let afio = AfioRegisters::ptr();
    unsafe {
        let mut pcf0 = read_volatile(&mut (*afio).pcf0);
        pcf0 &= !AFIO_PCF0_TIMER2_REMAP_MASK;
        pcf0 |= AFIO_PCF0_TIMER2_REMAP_PARTIAL;
        write_volatile(&mut (*afio).pcf0, pcf0);
    }
}
