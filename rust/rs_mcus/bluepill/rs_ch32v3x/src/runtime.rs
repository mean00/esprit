//! CH32V3x runtime initialization (`drop_c_part2.md` Phase C5).
//!
//! Replaces `esprit/cpp/mcus/riscv_ch32v3x/src/lnRunTime_riscv_ch32v3x.cpp`
//! in `LN_ENABLE_RUST` builds.

use crate::systick::ln_system_timer_init;

pub const CONFIG_TOTAL_HEAP_SIZE: usize = 22528;
pub const HAS_FPU: bool = true;

pub fn early_init() {
    ln_run_time_init();
    rs_rcu_bluepill::sysclk_ch32v3x::ln_init_system_clock();
    crate::irq::ln_irq_sys_init();
}

pub fn post_peripheral_init() {
    ln_run_time_init_post_peripherals();
}

#[unsafe(export_name = "_Z13lnRunTimeInitv")]
pub extern "C" fn ln_run_time_init() {}

#[unsafe(export_name = "_Z28lnRunTimeInitPostPeripheralsv")]
pub extern "C" fn ln_run_time_init_post_peripherals() {
    ln_system_timer_init();
    rs_fmc_bluepill::init_flash();
}
