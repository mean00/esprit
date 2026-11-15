//! C entries that used to live in `esprit/mcus/common_bluepill/lnExti.cpp`.
//!
//! That file is not compiled in a Rust build (`drop_C.md` P3); this module keeps the
//! two symbols its C++ callers still reference, plus the vector entries the ARM table
//! demands.
//!
//! The C++ callers are out of reach for this change:
//! `lnSystemInit.cpp` declares `void lnExtiSysInit();` and
//! `esprit/mcus/arm_gd32fx/src/lnRunTime_arm_gd32fx.cpp` declares
//! `extern void lnExtiSWDOnly();` - both plain C++ functions, in files that this plan
//! deliberately leaves untouched. So these two exports carry the Itanium-mangled name
//! instead of forcing an `extern "C"` edit; the same mechanism
//! `rs_timer_bluepill/src/delay_timer.rs` already uses for `lnEnableInterrupt()`.
//!
//! The attach/detach/enable/disable API has no C++ caller left, so it is *not*
//! exported: a Rust user goes through [`crate::attach_interrupt`] and friends.

use crate::exti_irq_handler;

/// `void lnExtiSysInit()`
#[unsafe(export_name = "_Z13lnExtiSysInitv")]
pub extern "C" fn ln_exti_sys_init() {
    crate::sys_init();
}

/// `void lnExtiSWDOnly()` - SWJ = "SWD enabled, JTAG disabled", which frees PA15, PB3
/// and PB4. `rs_afio_bluepill::release_jtag_pins()` programs the same field.
#[unsafe(export_name = "_Z13lnExtiSWDOnlyv")]
pub extern "C" fn ln_exti_swd_only() {
    rs_afio_bluepill::release_jtag_pins();
}

/// Dispatch a range of EXTI lines, the way the C++ `EXTI_IrqHandler(maskS, maskE)` did:
/// each pending-and-enabled line is cleared and its handler called.
fn dispatch(first: usize, last: usize) {
    for line in first..=last {
        exti_irq_handler(line);
    }
}

macro_rules! exti_irq {
    ($name:ident, $first:expr, $last:expr) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn $name() {
            dispatch($first, $last);
        }
    };
}

exti_irq!(EXTI0_IRQHandler, 0, 0);
exti_irq!(EXTI1_IRQHandler, 1, 1);
exti_irq!(EXTI2_IRQHandler, 2, 2);
exti_irq!(EXTI3_IRQHandler, 3, 3);
exti_irq!(EXTI4_IRQHandler, 4, 4);
exti_irq!(EXTI5_9_IRQHandler, 5, 9);
exti_irq!(EXTI10_15_IRQHandler, 10, 15);
