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

/// Apply the `LnRemapTimer` remap to general-purpose timer `timer` (0 = TIMER0, 1 =
/// TIMER1, 2 = TIMER2 - the AFIO `PCF0` layout), which is what
/// `mcus/common_bluepill/lnGPIO.cpp::lnRemapTimerPin()` did. `map` is the C++
/// `LnRemapTimer` value: `NoRemap = 0`, `PartialRemap = 1`, `FullRemap = 3`; TIMER2
/// encodes its partial remap as `0b10`, which is the one special case in
/// `lnRemapTimerPin()` and is kept here.
pub fn remap_timer(timer: u32, map: u32) {
    let pos = match timer {
        0 => AFIO_PCF0_TIMER0_REMAP_POS,
        1 => AFIO_PCF0_TIMER1_REMAP_POS,
        2 => AFIO_PCF0_TIMER2_REMAP_POS,
        _ => return,
    };
    // TIMER2's partial remap is not "1" but "2" in the register (lnRemapTimerPin).
    let value = if timer == 2 && map == LnRemapTimer::PartialRemap as u32 {
        AFIO_PCF0_TIMER2_REMAP_PARTIAL >> AFIO_PCF0_TIMER2_REMAP_POS
    } else {
        map
    };
    enable_afio();
    critical_section::with(|_| unsafe {
        modify_reg!(pcf0, |v| (v & !(AFIO_PCF0_TIMER_REMAP_MASK_2BIT << pos))
            | ((value & AFIO_PCF0_TIMER_REMAP_MASK_2BIT) << pos));
    });
}

/// `LnRemapTimer` of `esprit/include/lnGPIO.h`.
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LnRemapTimer {
    NoRemap = 0,
    PartialRemap = 1,
    FullRemap = 3,
}

/// `void lnRemapTimerPin(uint32_t timer, LnRemapTimer map)` - the one GPIO entry that
/// belongs to the AFIO block. `esprit/include/lnGPIO.h` declares it `extern "C"` when
/// `LN_RUST_BLUEPILL` is defined (its only caller, `lnRunTime_arm_gd32fx.cpp`, reaches
/// it through that header), which is why this export carries the plain C name.
#[unsafe(no_mangle)]
pub extern "C" fn lnRemapTimerPin(timer: u32, map: i32) {
    remap_timer(timer, map as u32);
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
