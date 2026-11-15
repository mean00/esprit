//! GD32F3x runtime early and post-peripheral initialization (`drop_c_part3.md` Phase G5).
//!
//! Replaces `esprit/cpp/mcus/arm_gd32fx/src/lnRunTime_arm_gd32fx.cpp` in
//! `LN_ENABLE_RUST` builds while leaving the C++ file untouched for
//! `LN_ENABLE_RUST=OFF`.

use core::arch::asm;
use core::ptr::write_volatile;

use crate::registers::ScbRegisters;

const TIMER1_INSTANCE: i32 = 1;
const PARTIAL_REMAP: i32 = 1;

unsafe extern "C" {
    fn lnRemapTimerPin(timer: i32, remap: i32);
}

pub const CONFIG_TOTAL_HEAP_SIZE: usize = 2048;
pub const HAS_FPU: bool = true;

pub fn early_init() {
    ln_run_time_init();
    rs_rcu_bluepill::sysclk_gd32fx::ln_init_system_clock();
    crate::irq::ln_irq_sys_init();
}

pub fn post_peripheral_init() {
    ln_run_time_init_post_peripherals();
}

/// `void lnRunTimeInit()`
#[unsafe(export_name = "_Z13lnRunTimeInitv")]
pub extern "C" fn ln_run_time_init() {
    rs_afio_bluepill::release_jtag_pins();
    // SAFETY: Disables maskable interrupts before resetting VTOR to 0.
    unsafe {
        asm!("cpsid if", options(nomem, nostack));
    }
    let scb = ScbRegisters::ptr();
    // SAFETY: `ScbRegisters::ptr()` points to SCB; resets VTOR to 0 before IRQ init.
    unsafe {
        write_volatile(&mut (*scb).vtor, 0);
    }
    rs_rcu_bluepill::sysclk_gd32fx::clear_reset_flags();
}

/// `void lnRunTimeInitPostPeripherals()`
#[unsafe(export_name = "_Z28lnRunTimeInitPostPeripheralsv")]
pub extern "C" fn ln_run_time_init_post_peripherals() {
    rs_afio_bluepill::release_jtag_pins();
    // SAFETY: Applies TIMER1 partial remap after peripherals are initialized.
    unsafe {
        lnRemapTimerPin(TIMER1_INSTANCE, PARTIAL_REMAP);
    }
}
