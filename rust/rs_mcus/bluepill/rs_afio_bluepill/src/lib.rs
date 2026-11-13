#![allow(unused_variables, unused_imports, unused_parens)]
#![allow(non_camel_case_types, non_snake_case, dead_code)]

#![no_std]
pub mod registers;

use registers::*;
use rs_rcu_bluepill::{Peripheral, enable};

rs_mcu_traits::define_mmio_accessors!(AfioRegisters::ptr());

/// Ensure AFIO clock is enabled.
pub fn enable_afio() {
    enable(Peripheral::Afio);
}

/// Configure SWJ to SWD enabled and JTAG disabled (freeing PA15, PB3, PB4).
pub fn release_jtag_pins() {
    enable_afio();
    critical_section::with(|_| unsafe {
        modify_reg!(pcf0, |v| (v & !AFIO_PCF0_SWJ_CFG_MASK) | AFIO_PCF0_SWJ_CFG_SWD_NO_JTAG);
    });
}

/// Apply partial remap to Timer 2 (TIM3): PB4 = CH1, PB5 = CH2.
pub fn remap_timer2_partial() {
    enable_afio();
    critical_section::with(|_| unsafe {
        modify_reg!(pcf0, |v| (v & !AFIO_PCF0_TIMER2_REMAP_MASK) | AFIO_PCF0_TIMER2_REMAP_PARTIAL);
    });
}

/// Select GPIO port as EXTI trigger source for the given pin index (0..15).
pub fn select_exti_source(port: u32, source: u32) {
    enable_afio();
    critical_section::with(|_| unsafe {
        let idx = (source >> 2) as usize;
        let shift = 4 * (source & 3);
        modify_reg!(extiss[idx], |v| (v & !(0xF << shift)) | (port << shift));
    });
}

/// Idiomatic struct interface for Alternate Function I/O (AFIO).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Afio;

impl Afio {
    #[inline(always)]
    pub fn enable() {
        enable_afio();
    }

    #[inline(always)]
    pub fn release_jtag_pins() {
        release_jtag_pins();
    }

    #[inline(always)]
    pub fn remap_timer2_partial() {
        remap_timer2_partial();
    }

    #[inline(always)]
    pub fn select_exti_source(port: u32, source: u32) {
        select_exti_source(port, source);
    }
}

pub mod shim;
pub use shim::*;
