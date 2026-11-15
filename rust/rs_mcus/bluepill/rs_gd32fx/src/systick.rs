//! ARM Cortex-M SysTick microsecond and cycle counters (`drop_c_part3.md` Phase G3).
//!
//! Replaces `esprit/cpp/mcus/arm_gd32fx/src/lnSystemTimer_arm_gd32fx.cpp` in
//! `LN_ENABLE_RUST` builds while leaving the C++ file untouched for
//! `LN_ENABLE_RUST=OFF`.

use core::ptr::read_volatile;
use core::sync::atomic::Ordering;

use crate::registers::SysTickRegisters;

unsafe extern "C" {
    fn lnGetMs_c() -> u32;
}

/// `uint32_t lnGetCycle32()`
#[unsafe(export_name = "_Z12lnGetCycle32v")]
pub extern "C" fn ln_get_cycle32() -> u32 {
    let systick = SysTickRegisters::ptr();
    // SAFETY: `SysTickRegisters::ptr()` points to the Cortex-M SysTick block (`0xE000_E010`).
    u32::MAX - unsafe { read_volatile(&(*systick).cvr) }
}

/// `uint64_t lnGetUs64()`
#[unsafe(export_name = "_Z9lnGetUs64v")]
pub extern "C" fn ln_get_us64() -> u64 {
    let systick = SysTickRegisters::ptr();
    let (before, cvr, rvr) = loop {
        // SAFETY: `lnGetMs_c` reads the volatile FreeRTOS millisecond tick counter.
        let before = unsafe { lnGetMs_c() } as u64;
        // SAFETY: `SysTickRegisters::ptr()` points to the Cortex-M SysTick block.
        let cvr = unsafe { read_volatile(&(*systick).cvr) } as u64;
        // SAFETY: Re-read `lnGetMs_c` to detect wrap across millisecond boundary.
        let after = unsafe { lnGetMs_c() } as u64;
        if before == after {
            // SAFETY: `rvr` is constant after SysTick initialization.
            let rvr = unsafe { read_volatile(&(*systick).rvr) } as u64;
            break (before, cvr, rvr);
        }
    };
    let elapsed_ticks = rvr + 1 - cvr;
    let scale = (rs_rcu_bluepill::sysclk_gd32fx::SystemCoreClock.load(Ordering::Relaxed) / 1_000_000) as u64;
    (elapsed_ticks / scale) + before * 1000
}

/// `uint32_t lnGetUs()`
#[unsafe(export_name = "_Z7lnGetUsv")]
pub extern "C" fn ln_get_us() -> u32 {
    ln_get_us64() as u32
}

/// `extern "C" uint32_t lnGetUs_C()`
#[unsafe(no_mangle)]
pub extern "C" fn lnGetUs_C() -> u32 {
    ln_get_us()
}

/// `extern "C" uint32_t lnGetCycle32_C()`
#[unsafe(no_mangle)]
pub extern "C" fn lnGetCycle32_C() -> u32 {
    ln_get_cycle32()
}
