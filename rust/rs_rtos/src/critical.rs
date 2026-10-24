//! Critical sections: run a closure (or a scope) with the scheduler frozen.
//!
//! This is the Rust spelling of the `taskENTER_CRITICAL()` /
//! `taskEXIT_CRITICAL()` pair the C++ side uses (and of `lnCriticalSection` in
//! `lnFreeRTOS_pp.h`): interrupts at and below `configMAX_SYSCALL_INTERRUPT_PRIORITY`
//! are masked and the scheduler cannot switch tasks, so the code in between sees
//! a shared structure change under it.
//!
//! Two shapes, same thing:
//!
//! * [`CriticalSection::enter()`] returns a guard that unmasks on `Drop`, so the
//!   section can span statements and every early return (or `?`) is correct
//!   without repeating the exit call;
//! * [`with_critical()`] is the one-expression form, `with_critical(|| ...)`.
//!
//! ```ignore
//! use rs_rtos::critical::with_critical;
//!
//! with_critical(|| {
//!     STATE.head = new_head;
//! });
//! ```
//!
//! # Rules
//!
//! * **Keep it short.**  The whole point is that nothing else runs, so no
//!   blocking call belongs inside: no `delay_ms`, no `take()`, no
//!   `xQueueSend()` with a timeout, no `Logger()` on a big buffer.  Nothing here
//!   stops you, but a blocked task inside a critical section is a hang.
//! * **Do not send the guard.**  A section belongs to the task that opened it,
//!   so [`CriticalSection`] is `!Send` (it holds a `PhantomData<*mut ()>`).
//! * **Nesting is fine.**  The port keeps a nesting counter (`uxCriticalNesting`),
//!   so a helper that takes a critical section can be called from inside one.
//! * **From an ISR** the equivalent is the `*_from_isr` API of the object you are
//!   protecting (see `sync.rs`), not this module: masking here would only change
//!   what the ISR itself may do, and the ISR already runs above any task.
//!
//! # Before the scheduler starts
//!
//! `vPortEnterCritical()` is only usable once the scheduler runs: the ports start
//! with `uxCriticalNesting = 0xaaaaaaaa`, so a `vPortExitCritical()` before that
//! would never unmask interrupts again.  So the section checks
//! `xTaskGetSchedulerState()` first and does nothing (`is_masked() == false`)
//! while the scheduler is not running -- which is also correct, because before
//! the scheduler there is no other task to race with.

use crate::c_freertos as rn_freertos_c;
use crate::prelude::*;

/// `taskSCHEDULER_RUNNING` from FreeRTOS `task.h`.
const TASK_SCHEDULER_RUNNING: rn_freertos_c::BaseType_t = 1;

/// True while the scheduler runs, i.e. while a critical section can be used.
///
/// `false` before `vTaskStartScheduler()` and after `vTaskEndScheduler()` /
/// `vTaskSuspendAll()`-style teardown, where the kernel must not be asked to
/// mask and unmask interrupts.
pub fn scheduler_running() -> bool {
    unsafe { rn_freertos_c::xTaskGetSchedulerState() == TASK_SCHEDULER_RUNNING }
}

/// An open critical section; the interrupts it masked are restored on `Drop`.
///
/// Created by [`CriticalSection::enter()`].  Dropping the guard is the only way
/// to close the section, which is what makes every exit path -- including a
/// `return` or a panic -- restore the interrupt state.
pub struct CriticalSection {
    /// Whether this section actually masked interrupts.  It did not when the
    /// scheduler was not running (see the module documentation), and then there
    /// is nothing to undo on `Drop` either: unmasking from the wrong nesting
    /// level is worse than not masking at all.
    masked: bool,
    /// A section belongs to the task that opened it.
    _not_send: PhantomData<*mut ()>,
}

impl CriticalSection {
    /// Enter a critical section, returning the guard that leaves it.
    pub fn enter() -> Self {
        let masked = scheduler_running();
        if masked {
            unsafe { rn_freertos_c::vPortEnterCritical() };
        }
        Self {
            masked,
            _not_send: PhantomData,
        }
    }

    /// True when this section is really masking interrupts.
    ///
    /// `false` before the scheduler runs; there is nothing else to synchronise
    /// with then, so the code inside the section is safe either way.
    pub fn is_masked(&self) -> bool {
        self.masked
    }
}

impl Drop for CriticalSection {
    fn drop(&mut self) {
        if self.masked {
            unsafe { rn_freertos_c::vPortExitCritical() };
        }
    }
}

/// Run `f` inside a critical section and return its result.
///
/// See the module documentation for the rules (`f` must not block).
pub fn with_critical<R>(f: impl FnOnce() -> R) -> R {
    let _section = CriticalSection::enter();
    f()
}
