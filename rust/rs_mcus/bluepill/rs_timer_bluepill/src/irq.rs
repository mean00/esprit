//! Vector entries the ARM interrupt table still demands for the timer drivers.
//!
//! `esprit/mcus/arm_gd32fx/include/lnIRQ_arm_vector.h` references
//! `TIMER1_IRQHandler` .. `TIMER6_IRQHandler` unconditionally and the table is not
//! weak. The C++ files that used to define them
//! (`esprit/mcus/common_bluepill/lnDelayTimer.cpp`, `lnBasicDelayTimer.cpp`) are not
//! compiled in a Rust build, so the crate owns the symbols - see `drop_C.md`, P2.
//!
//! `TIMER1..3` are the general-purpose delay timers `lnDelayTimer` drove, i.e. the
//! same hardware the Rust [`DelayTimer`](crate::delay_timer::DelayTimer) arms, so the
//! entry is the crate's own dispatcher (the handler table is indexed by hardware
//! timer number, exactly like `DelayTimer::new()`).
//!
//! `TIMER5/6` are the *basic* timers (`lnBasicDelayTimer`). Nothing in a Rust build
//! instantiates that class and nothing arms those interrupts - the entries exist only
//! to satisfy the table - and `rs_timer_bluepill` has no basic-timer driver, so the
//! handlers are empty on purpose. A future basic-timer user has to replace them.

use crate::delay_timer::delay_timer_interrupt_handler;
use crate::registers::TIMER_MAX_INSTANCES;

/// The general-purpose timer register block table, `LN_Timers_Registers *abTimers[5]`
/// of `lnTimer.cpp` (not compiled in a Rust build - `drop_C.md` P4).
///
/// Its only reader is `lnTimerWatch` (`include/lnTimerWatch.h`), the lock-free
/// tick-counter poller the LN SDI tap uses; it takes `abTimers[index]` once in its
/// constructor and then works on the raw `CNT` register. The table is `static mut` and
/// in `.data` on purpose: C++ declares it `LN_Timers_Registers *abTimers[]` (not
/// `const`), and the addresses are the ones `rs_timer_bluepill::registers` uses.
#[unsafe(no_mangle)]
pub static mut abTimers: [*mut u32; TIMER_MAX_INSTANCES] = [
    rs_bluepill::TIM1_BASE as *mut u32,
    rs_bluepill::TIM2_BASE as *mut u32,
    rs_bluepill::TIM3_BASE as *mut u32,
    rs_bluepill::TIM4_BASE as *mut u32,
    rs_bluepill::TIM5_BASE as *mut u32,
];

macro_rules! delay_timer_irq {
    ($name:ident, $id:expr) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn $name() {
            delay_timer_interrupt_handler($id);
        }
    };
}

delay_timer_irq!(TIMER1_IRQHandler, 1);
delay_timer_irq!(TIMER2_IRQHandler, 2);
delay_timer_irq!(TIMER3_IRQHandler, 3);

/// Basic timer 5: no Rust driver and no user, entry kept for the vector table.
#[unsafe(no_mangle)]
pub extern "C" fn TIMER5_IRQHandler() {}

/// Basic timer 6: no Rust driver and no user, entry kept for the vector table.
#[unsafe(no_mangle)]
pub extern "C" fn TIMER6_IRQHandler() {}
