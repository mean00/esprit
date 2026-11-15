//! WCH SysTick cycle & microsecond timer (`drop_c_part2.md` Phase C3).
//!
//! Replaces `esprit/cpp/mcus/riscv_ch32v3x/src/lnSystemTimer_riscv_ch32v3x.cpp`
//! in `LN_ENABLE_RUST` builds.

use core::ptr::read_volatile;
use core::sync::atomic::{AtomicU32, Ordering};

use crate::registers::{STK_CTLR_STE, STK_SR_CNTIF, SysTickRegs};
use rs_rcu_bluepill::sysclk_ch32v3x::SystemCoreClock;

/// Used by `freeRTOS_extension/port.c`.
#[unsafe(no_mangle)]
pub static mut WCH_SysTick: *mut SysTickRegs = rs_bluepill::STK_BASE as *mut SysTickRegs;

static TICK_PER_US_16: AtomicU32 = AtomicU32::new(1);

unsafe extern "C" {
    fn lnGetMs_c() -> u32;
}

#[unsafe(export_name = "_Z17lnSystemTimerInitv")]
pub extern "C" fn ln_system_timer_init() {
    let sysclk = SystemCoreClock.load(Ordering::Relaxed) as u64;
    TICK_PER_US_16.store(((sysclk * 16) / 1_000_000) as u32, Ordering::Relaxed);
}

#[unsafe(export_name = "_Z12lnGetCycle64v")]
pub extern "C" fn ln_get_cycle64() -> u64 {
    let stk = SysTickRegs::ptr();
    // SAFETY: `SysTickRegs::ptr()` points to the QingKe SysTick block (`0xE000_F000`).
    let ctlr = unsafe { read_volatile(&(*stk).ctlr) };
    assert!(ctlr & STK_CTLR_STE != 0, "SysTick not enabled");

    let mut ticks: u32;
    let mut count: u32;
    let mut is_pending: u32;

    loop {
        // SAFETY: `lnGetMs_c` reads the volatile FreeRTOS millisecond tick counter.
        ticks = unsafe { lnGetMs_c() };
        // SAFETY: `SysTickRegs::ptr()` points to the QingKe SysTick block.
        count = unsafe { read_volatile(&(*stk).cnt) } as u32;
        is_pending = unsafe { read_volatile(&(*stk).sr) } & STK_SR_CNTIF;
        // SAFETY: Re-read `lnGetMs_c` to detect wrap across millisecond boundary.
        let ticks2 = unsafe { lnGetMs_c() };
        if ticks == ticks2 {
            break;
        }
    }

    // SAFETY: `cmp` is constant after SysTick initialization.
    let cmp = unsafe { read_volatile(&(*stk).cmp) } as u32;
    if is_pending != 0 && count < (cmp / 2) {
        ticks = ticks.wrapping_add(1);
    }

    (ticks as u64) * (cmp as u64) + (count as u64)
}

#[unsafe(export_name = "_Z12lnGetCycle32v")]
pub extern "C" fn ln_get_cycle32() -> u32 {
    ln_get_cycle64() as u32
}

#[unsafe(export_name = "_Z9lnGetUs64v")]
pub extern "C" fn ln_get_us64() -> u64 {
    let tick = ln_get_cycle64();
    let per_us16 = TICK_PER_US_16.load(Ordering::Relaxed) as u64;
    (tick * 16) / per_us16
}

#[unsafe(export_name = "_Z7lnGetUsv")]
pub extern "C" fn ln_get_us() -> u32 {
    ln_get_us64() as u32
}
