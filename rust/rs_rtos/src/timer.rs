//! Software timers: [`PeriodicTimer`], the Rust form of `lnPeriodicTimer`.
//!
//! ```ignore
//! use rs_rtos::timer::PeriodicTimer;
//!
//! fn poll(cookie: &u32) {
//!     // runs in the timer daemon task, once per period
//! }
//!
//! static RX_POLL: PeriodicTimer<u32> = PeriodicTimer::new(c"rx poll", 20, 0, poll);
//! RX_POLL.start();
//! ```
//!
//! This is not a hardware timer: FreeRTOS implements it with a control block in
//! our own storage plus a **timer daemon task**, which is what makes the
//! callback run in task context (so it may take a mutex or log) and what makes
//! `start()`/`stop()` asynchronous -- they post a command to the daemon and the
//! timer changes state when the daemon gets to it.
//!
//! # Which kernel objects are involved
//!
//! * `xTimerCreateStatic()` -- the control block lives in the [`TimerStorage`]
//!   inside the timer, sized from `crate::kernel::STATIC_TIMER_BYTES`, so a
//!   timer costs no heap.  It is created on **first use** (like the wrappers in
//!   `sync.rs`), which is what lets `new()` be a `const fn` and lets the timer
//!   move once, into a `static` or into the object that owns it, before that.
//!   After the first use the `Timer_t` and the callback cookie point into the
//!   timer: do not move it, do not `memcpy` it, do not put it in a `Vec`.
//! * `xTimerStart()` / `Stop()` / `Reset()` / `Delete()` are **macros** in
//!   `timers.h`, so the calls go through `xTimerGenericCommandFromTask()`, the
//!   function they expand to, with the `tmrCOMMAND_*` ids below.
//! * the callback: FreeRTOS calls one C function per timer with the timer
//!   handle; we hand it a `pvTimerID` pointing at a [`TimerCookie`] holding the
//!   user callback and the user cookie, and a small generic trampoline gets the
//!   cookie back out with `pvTimerGetTimerID()` and calls `callback(&cookie)`.
//!
//! # Teardown
//!
//! There is deliberately **no `Drop`**: the storage belongs to the timer, and
//! every way of removing a timer from the kernel (`xTimerDelete()`, and even
//! `Stop()`) is a command the daemon processes *later* -- it would still write
//! to the control block after the timer is gone.  `lnPeriodicTimer`'s
//! destructor is empty for the same reason.  A timer is meant to live in a
//! `static` for the life of the firmware (statics are never dropped); one that
//! really has to go away must be stopped and the stop must have been processed
//! -- poll [`PeriodicTimer::is_active()`] until it is `false` -- before its
//! storage disappears.
//!
//! # Milliseconds, not ticks
//!
//! `lnPeriodicTimer::init()` passes its `periodInMs` to the kernel as a tick
//! count (which is only correct because this kernel runs at 1 kHz); here the
//! period is converted with `crate::task::ms_to_ticks()`, the one place that
//! knows `configTICK_RATE_HZ`, and the original value is what
//! [`PeriodicTimer::period_ms()`] returns.

use crate::c_freertos as rn_freertos_c;
use crate::critical::CriticalSection;
use crate::prelude::*;
use core::ffi::CStr;

// ===========================================================================
//  Storage for one statically created timer
// ===========================================================================

/// `sizeof(StaticTimer_t)` of the kernel we link, i.e. the `.bss` one timer
/// costs.  Measured by the build, see `crate::kernel`.
const STATIC_TIMER_BYTES: usize = crate::kernel::STATIC_TIMER_BYTES;

/// Words needed for that many bytes.  A 32-bit alignment is all the control
/// block needs: a `ListItem_t`, a `TickType_t`, two `UBaseType_t` and a status
/// flag.
const STATIC_TIMER_WORDS: usize = STATIC_TIMER_BYTES / mem::size_of::<u32>();

/// A trailing partial word would mean the buffer is smaller than the object the
/// kernel writes into it, so refuse to compile.
const _: () = assert!(
    STATIC_TIMER_BYTES.is_multiple_of(mem::size_of::<u32>()),
    "STATIC_TIMER_BYTES is not a whole number of u32 words"
);

/// Storage for one statically created timer (`.bss` when the timer is a
/// `static`, part of the timer object otherwise).
#[repr(C)]
struct TimerStorage {
    words: [u32; STATIC_TIMER_WORDS],
}

impl TimerStorage {
    /// Zeroed storage: `xTimerCreateStatic()` expects uninitialised memory and
    /// sets up everything it needs.
    const fn new() -> Self {
        Self {
            words: [0; STATIC_TIMER_WORDS],
        }
    }
}

// ===========================================================================
//  Kernel constants
//
//  The command ids are `#define`s in `timers.h`, so bindgen cannot emit them
//  and they are spelled out here with the macro they name.
// ===========================================================================

/// `tmrCOMMAND_START`, what `xTimerStart()` expands to.
const TMR_COMMAND_START: rn_freertos_c::BaseType_t = 1;
/// `tmrCOMMAND_RESET`, what `xTimerReset()` expands to.
const TMR_COMMAND_RESET: rn_freertos_c::BaseType_t = 2;
/// `tmrCOMMAND_STOP`, what `xTimerStop()` expands to.
const TMR_COMMAND_STOP: rn_freertos_c::BaseType_t = 3;
/// `tmrCOMMAND_RESET_FROM_ISR`, what `xTimerResetFromISR()` expands to.
const TMR_COMMAND_RESET_FROM_ISR: rn_freertos_c::BaseType_t = 7;

/// `pdTRUE`.
const PD_TRUE: rn_freertos_c::BaseType_t = 1;

/// Ticks a command from a task waits for room in the daemon's queue (the `1`
/// `lnPeriodicTimer` passes: enough to ride out a momentarily full queue, short
/// enough never to be a real delay).
const COMMAND_WAIT_TICKS: rn_freertos_c::TickType_t = 1;

// ===========================================================================
//  The cookie the kernel hands back to the trampoline
// ===========================================================================

/// What `pvTimerID` points at: the user's cookie **and** the user's callback.
///
/// Both have to be reachable from the timer, because FreeRTOS calls one C
/// function per timer with nothing but the timer handle -- so the trampoline is
/// generic over `C` (to get the layout back) and this structure carries the
/// rest.
struct TimerCookie<C> {
    /// Function to call on every expiry.
    callback: fn(&C),
    /// Value handed to it.
    cookie: C,
}

/// The C callback FreeRTOS calls: recover the cookie, call the user callback.
///
/// One monomorphised copy per cookie type; `xTimerCreateStatic()` is given the
/// pointer to the right one.
unsafe extern "C" fn call_timer_callback<C>(x_timer: rn_freertos_c::TimerHandle_t) {
    let identity = unsafe { rn_freertos_c::pvTimerGetTimerID(x_timer) }.cast::<TimerCookie<C>>();
    assert!(
        !identity.is_null(),
        "software timer fired without a cookie (was it created by PeriodicTimer?)"
    );
    let identity = unsafe { &*identity };
    (identity.callback)(&identity.cookie);
}

// ===========================================================================
//  PeriodicTimer
// ===========================================================================

/// A periodic software timer: calls `callback(&cookie)` in the timer daemon
/// task every `period_ms`, while started.
///
/// See the module documentation for the lifetime rule (created on first use,
/// do not move it afterwards) and for teardown.
pub struct PeriodicTimer<C> {
    /// Kernel handle, null until the timer is created on first use.
    handle: UnsafeCell<rn_freertos_c::TimerHandle_t>,
    /// Storage the kernel control block lives in (sized from
    /// `crate::kernel::STATIC_TIMER_BYTES`, `.bss` when the timer is a
    /// `static` -- no heap either way).
    storage: UnsafeCell<TimerStorage>,
    /// The `pvTimerID` the kernel keeps: user callback + user cookie.
    identity: UnsafeCell<TimerCookie<C>>,
    /// Timer name, only for `xTimerGetTimerName()` / tracing.
    name: &'static CStr,
    /// Period in milliseconds, converted to ticks at creation.
    period_ms: u32,
}

impl<C> PeriodicTimer<C> {
    /// Create a periodic timer that calls `callback(&cookie)` every `period_ms`.
    ///
    /// No kernel call and no allocation: the kernel object is created in the
    /// timer's own storage the first time the timer is used (see the module
    /// documentation for the move rule that makes that necessary).
    ///
    /// `name` is only used for tracing (`xTimerGetTimerName()`), so a C string
    /// literal (`c"uart rx"`) is the expected argument.
    pub const fn new(name: &'static CStr, period_ms: u32, cookie: C, callback: fn(&C)) -> Self {
        assert!(
            period_ms > 0,
            "PeriodicTimer: the period must be at least 1 ms"
        );
        Self {
            handle: UnsafeCell::new(ptr::null_mut()),
            storage: UnsafeCell::new(TimerStorage::new()),
            identity: UnsafeCell::new(TimerCookie { callback, cookie }),
            name,
            period_ms,
        }
    }

    /// Handle of the kernel timer, created on first use.
    ///
    /// The creation runs in a critical section for the same reason as
    /// `sync.rs::handle_or_create()`: two tasks must not build the object in
    /// the same storage.  `xTimerCreateStatic()` neither blocks nor allocates.
    fn kernel_handle(&self) -> rn_freertos_c::TimerHandle_t {
        let mut created = unsafe { *self.handle.get() };
        if !created.is_null() {
            return created;
        }

        let _section = CriticalSection::enter();
        created = unsafe { *self.handle.get() };
        if created.is_null() {
            let ticks = crate::task::ms_to_ticks(self.period_ms);
            assert!(
                ticks > 0,
                "PeriodicTimer: period is shorter than one kernel tick"
            );
            created = unsafe {
                rn_freertos_c::xTimerCreateStatic(
                    self.name.as_ptr(),
                    ticks,
                    PD_TRUE, // auto-reload: this is a *periodic* timer
                    self.identity.get().cast(),
                    Some(call_timer_callback::<C>),
                    self.storage.get().cast(),
                )
            };
            assert!(
                !created.is_null(),
                "static FreeRTOS timer creation returned NULL"
            );
            unsafe { *self.handle.get() = created };
        }
        created
    }

    /// Post one `tmrCOMMAND_*` to the timer daemon and say whether it was
    /// accepted (a command queue still full after `wait` ticks is the failure).
    fn command(
        &self,
        command: rn_freertos_c::BaseType_t,
        value: rn_freertos_c::TickType_t,
        wait: rn_freertos_c::TickType_t,
    ) -> bool {
        let ret = unsafe {
            rn_freertos_c::xTimerGenericCommandFromTask(
                self.kernel_handle(),
                command,
                value,
                ptr::null_mut(),
                wait,
            )
        };
        ret != 0
    }

    /// Start the timer: the first expiry is one period from now.
    ///
    /// Only the first `start()` of a stopped timer does anything -- starting a
    /// running timer leaves its expiry time alone (that is `restart()`).
    /// Returns `false` when the command could not be posted.
    ///
    /// Must not be called from inside a critical section: the command may wait
    /// up to [`COMMAND_WAIT_TICKS`] for the daemon, which cannot run while the
    /// scheduler is masked (`try_start()` is the non-blocking form).
    pub fn start(&self) -> bool {
        self.command(
            TMR_COMMAND_START,
            crate::task::tick_count(),
            COMMAND_WAIT_TICKS,
        )
    }

    /// [`start()`](Self::start) without waiting for room in the daemon's queue.
    pub fn try_start(&self) -> bool {
        self.command(TMR_COMMAND_START, crate::task::tick_count(), 0)
    }

    /// Stop the timer.  It keeps its period and can be started again.
    ///
    /// Must not be called from inside a critical section: see
    /// [`start()`](Self::start).
    pub fn stop(&self) -> bool {
        self.command(TMR_COMMAND_STOP, 0, COMMAND_WAIT_TICKS)
    }

    /// [`stop()`](Self::stop) without waiting for room in the daemon's queue.
    pub fn try_stop(&self) -> bool {
        self.command(TMR_COMMAND_STOP, 0, 0)
    }

    /// Restart the timer: the next expiry is one full period from now.
    pub fn restart(&self) -> bool {
        self.command(
            TMR_COMMAND_RESET,
            crate::task::tick_count(),
            COMMAND_WAIT_TICKS,
        )
    }

    /// Restart the timer from an interrupt handler.
    ///
    /// The timer must have been used from a task first, so that it exists: the
    /// kernel object cannot be created from an ISR.
    ///
    /// This does not switch tasks by itself: like `BinarySemaphore::give_from_isr`
    /// in `sync.rs`, the "higher priority task woken" flag the kernel reports is
    /// dropped, so the daemon task starts running at the next scheduling point
    /// instead of immediately.  For a timer that is harmless -- the expiry is
    /// computed from the tick count, not from when the daemon runs.
    pub fn restart_from_isr(&self) -> bool {
        let mut higher_priority_task_woken: rn_freertos_c::BaseType_t = 0;
        let ret = unsafe {
            rn_freertos_c::xTimerGenericCommandFromISR(
                self.kernel_handle(),
                TMR_COMMAND_RESET_FROM_ISR,
                rn_freertos_c::xTaskGetTickCountFromISR(),
                &mut higher_priority_task_woken,
                0,
            )
        };
        ret != 0
    }

    /// Whether the timer is currently running.
    ///
    /// The answer is the daemon's view, so it lags `start()` / `stop()` by one
    /// daemon iteration.
    pub fn is_active(&self) -> bool {
        unsafe { rn_freertos_c::xTimerIsTimerActive(self.kernel_handle()) != 0 }
    }

    /// The period this timer was created with, in milliseconds.
    pub fn period_ms(&self) -> u32 {
        self.period_ms
    }

    /// The cookie the callback is called with.
    pub fn cookie(&self) -> &C {
        // SAFETY: the cookie is written once by `new()` and only read after
        // that, so a shared reference to it is all anyone may take.
        unsafe { &(*self.identity.get()).cookie }
    }
}

