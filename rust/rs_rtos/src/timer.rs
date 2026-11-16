//! Software timers: [`PeriodicTimer`], the Rust form of `lnPeriodicTimer`.
//!
//! ```ignore
//! use rs_rtos::timer::PeriodicTimer;
//!
//! fn poll(cookie: &u32) {
//!     // runs in the timer daemon task, once per period
//! }
//!
//! static RX_POLL_SLOT: TimerSlot<u32> = TimerSlot::new(0, poll);
//! static RX_POLL: PeriodicTimer<u32> = PeriodicTimer::new(c"rx poll", 20, &RX_POLL_SLOT);
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
//! * `xTimerCreateStatic()` -- the control block lives in a [`TimerSlot`], sized
//!   from `crate::kernel::STATIC_TIMER_BYTES`, so a timer costs no heap.  The
//!   kernel keeps **two pointers into that slot** (the `Timer_t` it writes in
//!   place and the `pvTimerID` it hands back to the callback), which is why the
//!   slot is passed as `&'static`: the address the kernel captured can then never
//!   outlive the memory it points at, and moving the [`PeriodicTimer`] (or the
//!   object that owns it) is harmless.  Passing a slot that is not `'static` is a
//!   compile error, so the "timer moved after it was created" corruption cannot
//!   be written by accident.
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
use core::ffi::{CStr, c_void};

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

/// Everything the kernel keeps a pointer to, for one timer: the control block it writes *in
/// place* and the `pvTimerID` it hands back to the callback.
///
/// Declare one per timer, in a `static` (that is the whole point: see the module docs).
///
/// ```ignore
/// static RX_POLL_SLOT: TimerSlot<u32> = TimerSlot::new(0, poll);
/// ```
#[repr(C)]
pub struct TimerSlot<C> {
    /// Storage the kernel writes the `Timer_t` into.
    control: UnsafeCell<[u32; STATIC_TIMER_WORDS]>,
    /// What `pvTimerID` points at.
    identity: TimerCookie<C>,
}

// SAFETY: `control` is only ever written by the kernel (it is registered once, and the timer
// daemon owns the object from then on); Rust never creates a reference into it, it only hands
// the pointer over.  `identity` is read-only after creation, and `C: Sync` covers the cookie.
unsafe impl<C: Sync> Sync for TimerSlot<C> {}

impl<C> TimerSlot<C> {
    /// An empty slot that will call `callback(&cookie)`.  `const`, so it fits a `static`.
    pub const fn new(cookie: C, callback: fn(&C)) -> Self {
        Self {
            control: UnsafeCell::new([0; STATIC_TIMER_WORDS]),
            identity: TimerCookie { callback, cookie },
        }
    }

    /// The `Timer_t` storage, as the `void*` `xTimerCreateStatic()` writes through.
    fn control_ptr(&self) -> *mut rn_freertos_c::xSTATIC_TIMER {
        self.control.get().cast()
    }

    /// The `pvTimerID` the trampoline gets back from the kernel.
    ///
    /// The kernel only stores and returns this pointer, never writes through it, so handing
    /// out a `*mut` to a field behind `&self` is sound.
    fn identity_ptr(&self) -> *mut c_void {
        (&self.identity as *const TimerCookie<C> as *mut TimerCookie<C>).cast()
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
/// See the module documentation for the [`TimerSlot`] contract and for teardown.
pub struct PeriodicTimer<C: 'static> {
    /// Kernel handle, null until the timer is created on first use.
    handle: UnsafeCell<rn_freertos_c::TimerHandle_t>,
    /// Where the kernel's control block and `pvTimerID` live.  `'static` is the contract:
    /// the kernel keeps these addresses for the life of the firmware, so they must not
    /// belong to a value that can move or go away.
    slot: &'static TimerSlot<C>,
    /// Timer name, only for `xTimerGetTimerName()` / tracing.
    name: &'static CStr,
    /// Period in milliseconds, converted to ticks at creation.
    period_ms: u32,
}

// SAFETY: `handle` is initialised once under `CriticalSection::enter()` in `kernel_handle()`,
// `slot` is `Sync` when `C: Sync`, and FreeRTOS timer commands are thread-safe.
unsafe impl<C: Sync> Sync for PeriodicTimer<C> {}

impl<C: 'static> PeriodicTimer<C> {
    /// Create a periodic timer that calls the `callback(&cookie)` captured in `slot`, every
    /// `period_ms`.
    ///
    /// No kernel call and no allocation: the kernel object is created in `slot` the first
    /// time the timer is used.  `slot` is `&'static` on purpose -- the kernel keeps pointers
    /// into it forever, so a slot that could move (or be dropped) is a compile error rather
    /// than a timer list corrupted minutes later.
    ///
    /// `name` is only used for tracing (`xTimerGetTimerName()`), so a C string
    /// literal (`c"uart rx"`) is the expected argument.
    pub const fn new(name: &'static CStr, period_ms: u32, slot: &'static TimerSlot<C>) -> Self {
        assert!(
            period_ms > 0,
            "PeriodicTimer: the period must be at least 1 ms"
        );
        Self {
            handle: UnsafeCell::new(ptr::null_mut()),
            slot,
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
                    self.slot.identity_ptr(),
                    Some(call_timer_callback::<C>),
                    self.slot.control_ptr(),
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
    /// A timer that was never started has no kernel object yet: creating one just to stop
    /// it would write the control block *in place* into whatever frame this happens to be,
    /// and the kernel would keep that address for good (see the move rule above), so this
    /// is a no-op in that case.
    ///
    /// Must not be called from inside a critical section: see
    /// [`start()`](Self::start).
    pub fn stop(&self) -> bool {
        if unsafe { *self.handle.get() }.is_null() {
            return true;
        }
        self.command(TMR_COMMAND_STOP, 0, COMMAND_WAIT_TICKS)
    }

    /// [`stop()`](Self::stop) without waiting for room in the daemon's queue.
    pub fn try_stop(&self) -> bool {
        if unsafe { *self.handle.get() }.is_null() {
            return true;
        }
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
        if unsafe { *self.handle.get() }.is_null() {
            return false;
        }
        unsafe { rn_freertos_c::xTimerIsTimerActive(self.kernel_handle()) != 0 }
    }

    /// The period this timer was created with, in milliseconds.
    pub fn period_ms(&self) -> u32 {
        self.period_ms
    }

    /// The cookie the callback is called with.
    pub fn cookie(&self) -> &C {
        // The slot is `'static` and the cookie is written by `TimerSlot::new()`, i.e. before
        // the kernel is given the pointer, so only shared reads happen afterwards.
        &self.slot.identity.cookie
    }
}

