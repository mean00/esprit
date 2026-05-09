#![allow(dead_code)]

//! Task creation, time, delay, and task notifications.
//!
//! # FreeRTOS task notifications
//!
//! Every FreeRTOS task has an array of notification values (indexed from 0).
//! Use [`TaskHandle::notify`], [`TaskHandle::notify_wait`],
//! [`TaskHandle::notify_take`] to communicate between tasks with very low
//! overhead (no queue allocation needed).

use crate::prelude::*;
use crate::rn_freertos_c;
use crate::rn_timer_c as rt;
use core::ffi::c_void;

// ---- time and delay ----

/// Block for `ms` milliseconds.
#[inline]
pub fn delay_ms(ms: u32) {
    unsafe { rt::lnDelay_C(ms) }
}

/// Block for `us` microseconds.
#[inline]
pub fn delay_us(us: u32) {
    unsafe { rt::lnDelayUs(us) }
}

/// Sleep (block) for the given duration.
///
/// This is the `fake_std`-compatible alias for [`delay_ms`].
/// Modelled after `std::thread::sleep`.
#[inline]
pub fn sleep(dur: Duration) {
    delay_ms(dur.as_millis() as u32);
}

/// Sleep (block) for `ms` milliseconds.
///
/// This is the `fake_std`-compatible alias for [`delay_ms`].
#[inline]
pub fn sleep_ms(ms: u32) {
    delay_ms(ms);
}

/// Return a monotonic millisecond counter.
///
/// Uses the hardware timer (`lnGetMs`).  For the FreeRTOS tick count
/// (which may have lower resolution) use [`tick_count`].
#[inline]
pub fn time_ms() -> u32 {
    unsafe { rt::lnGetMs() }
}

/// Return the current FreeRTOS tick count.
///
/// At the default tick rate of 1000 Hz this is equivalent to milliseconds,
/// but the actual resolution depends on `configTICK_RATE_HZ`.
#[inline]
pub fn tick_count() -> u32 {
    unsafe { rn_freertos_c::xTaskGetTickCount() }
}

/// Return a monotonic microsecond counter.
#[inline]
pub fn time_us() -> u32 {
    unsafe { rt::lnGetUs() }
}

/// Return a 64‑bit monotonic microsecond counter.
#[inline]
pub fn time_us64() -> u64 {
    unsafe { rt::lnGetUs64() }
}

// ---- task handle ----

/// Handle to a FreeRTOS task.
///
/// Obtain via [`spawn`], [`current`], or [`TaskHandle::from_raw`].
///
/// This is essentially a newtype over the C `TaskHandle_t`.  Dropping it
/// does **not** delete the task.
#[derive(Debug, Copy, Clone)]
pub struct TaskHandle {
    raw: rn_freertos_c::TaskHandle_t,
}

impl TaskHandle {
    /// Wrap a raw C `TaskHandle_t`.  The caller guarantees it points to a
    /// live task for as long as this handle is used.
    ///
    /// # Safety
    /// `raw` must be a valid, non‑null task handle.
    pub unsafe fn from_raw(raw: rn_freertos_c::TaskHandle_t) -> Self {
        Self { raw }
    }

    /// Return the raw C handle.
    pub fn raw(&self) -> rn_freertos_c::TaskHandle_t {
        self.raw
    }

    /// Suspend this task.
    pub fn suspend(&self) {
        unsafe { rn_freertos_c::vTaskSuspend(self.raw) }
    }

    /// Resume this task (can be called from any task; use `resume_from_isr`
    /// from an ISR).
    pub fn resume(&self) {
        unsafe { rn_freertos_c::vTaskResume(self.raw) }
    }

    /// Resume this task from an ISR.
    ///
    /// # Safety
    /// Must only be called from an ISR.
    pub unsafe fn resume_from_isr(&self) -> bool {
        rn_freertos_c::xTaskResumeFromISR(self.raw) != 0
    }

    /// Delete the task.  After this, the handle is invalid.
    ///
    /// If called on the running task, the deletion happens after the last
    /// `return` from the task function.
    pub fn delete(self) {
        unsafe { rn_freertos_c::vTaskDelete(self.raw) }
    }

    /// Get the task's current priority.
    pub fn priority(&self) -> u32 {
        unsafe { rn_freertos_c::uxTaskPriorityGet(self.raw) as u32 }
    }

    /// Get the task name.
    pub fn name(&self) -> &'static str {
        unsafe {
            let ptr = rn_freertos_c::pcTaskGetName(self.raw);
            let len = (0..).find(|&i| *ptr.add(i) == 0).unwrap_or(0);
            core::str::from_utf8_unchecked(core::slice::from_raw_parts(ptr as *const u8, len))
        }
    }

    // -- Task notifications (index 0) --------------------------------------

    /// Send a notification to this task, setting bits.
    ///
    /// Equivalent to `xTaskGenericNotify(task, 0, value, eSetBits, NULL)`.
    /// Returns `true` on success.
    pub fn notify(&self, value: u32) -> bool {
        let ret = unsafe {
            rn_freertos_c::xTaskGenericNotify(
                self.raw,
                0,
                value,
                rn_freertos_c::eNotifyAction_eSetBits,
                core::ptr::null_mut(),
            )
        };
        ret != 0
    }

    /// Send a notification from an ISR.
    ///
    /// Returns `(true, woken)` if sent, where `woken` means a higher-priority
    /// task was woken (caller should pend a yield).
    ///
    /// # Safety
    /// Must only be called from an ISR.
    pub unsafe fn notify_from_isr(&self, value: u32) -> (bool, bool) {
        let mut higher_woken: rn_freertos_c::BaseType_t = 0;
        let ret = rn_freertos_c::xTaskGenericNotifyFromISR(
            self.raw,
            0,
            value,
            rn_freertos_c::eNotifyAction_eSetBits,
            core::ptr::null_mut(),
            &mut higher_woken,
        );
        (ret != 0, higher_woken != 0)
    }

    /// Send a notification value with overwrite.
    pub fn notify_value(&self, value: u32) {
        unsafe {
            rn_freertos_c::xTaskGenericNotify(
                self.raw,
                0,
                value,
                rn_freertos_c::eNotifyAction_eSetValueWithOverwrite,
                core::ptr::null_mut(),
            );
        }
    }

    /// Wait for a notification, clearing bits on entry and exit.
    ///
    /// * `clear_on_entry` – bits to clear on entry.
    /// * `clear_on_exit`  – bits to clear on exit.
    /// * `timeout_ms`     – maximum wait time (or `u32::MAX` for infinite).
    ///
    /// Returns the notification value.
    pub fn notify_wait(
        &self,
        clear_on_entry: u32,
        clear_on_exit: u32,
        timeout_ms: u32,
    ) -> Result<u32, ()> {
        let ticks = ms_to_ticks(timeout_ms);
        let mut val: u32 = 0;
        let ret = unsafe {
            rn_freertos_c::xTaskGenericNotifyWait(0, clear_on_entry, clear_on_exit, &mut val, ticks)
        };
        if ret != 0 { Ok(val) } else { Err(()) }
    }

    /// Binary notification take – decrements the notification count if > 0,
    /// otherwise blocks.
    ///
    /// Returns the notification value.
    pub fn notify_take(&self, clear_count: bool, timeout_ms: u32) -> Result<u32, ()> {
        let ticks = ms_to_ticks(timeout_ms);
        let clear = if clear_count { 1 } else { 0 };
        let ret = unsafe { rn_freertos_c::ulTaskGenericNotifyTake(0, clear, ticks) };
        if ret != 0 { Ok(ret) } else { Err(()) }
    }
}

/// Return the handle of the currently running task.
pub fn current() -> TaskHandle {
    let raw = unsafe { rn_freertos_c::xTaskGetCurrentTaskHandle() };
    unsafe { TaskHandle::from_raw(raw) }
}

/// Yield the processor to other ready tasks at the same priority.
#[inline]
pub fn yield_now() {
    // FreeRTOS: vTaskDelay(0) yields to equal-priority tasks.
    unsafe { rn_freertos_c::vTaskDelay(0) }
}

// ---- task creation ----

/// Spawn a FreeRTOS task from a `FnOnce` closure.
///
/// # Parameters
/// * `name`     – printable task name (for debugging).
/// * `stack`    – stack depth in *words* (typically 256..1024).
/// * `priority` – FreeRTOS priority (higher = more urgent).
/// * `f`        – the closure to execute.  Must be `Send + 'static`.
///
/// Returns a [`TaskHandle`] for the newly created task.
pub fn spawn<F>(name: &str, stack: u32, priority: u32, f: F) -> TaskHandle
where
    F: FnOnce() + Send + 'static,
{
    // Box the closure so it lives on the heap.
    let closure: Box<dyn FnOnce() + Send> = Box::new(f);
    let raw = Box::into_raw(Box::new(closure));

    extern "C" fn trampoline(param: *mut c_void) {
        let closure: Box<Box<dyn FnOnce() + Send>> = unsafe { Box::from_raw(param as *mut _) };
        closure();
        // FreeRTOS task functions must never return – delete ourselves.
        unsafe { rn_freertos_c::vTaskDelete(core::ptr::null_mut()) };
    }

    let mut name_buf = [0u8; 32];
    let copy_len = name.len().min(31);
    name_buf[..copy_len].copy_from_slice(&name.as_bytes()[..copy_len]);

    let mut handle: rn_freertos_c::TaskHandle_t = core::ptr::null_mut();

    unsafe {
        let ret = rn_freertos_c::xTaskCreate(
            Some(trampoline),
            name_buf.as_ptr() as *const cty::c_char,
            stack,
            raw as *mut c_void,
            priority as rn_freertos_c::UBaseType_t,
            &mut handle,
        );
        assert!(ret != 0, "spawn: xTaskCreate failed");
        TaskHandle::from_raw(handle)
    }
}

/// Legacy task entry function pointer type (used by `rn_os_helper`).
pub type TaskEntry = fn(param: *mut core::ffi::c_void);

/// Spawn a task from a raw C‑compatible function pointer.
///
/// This is the low‑level building block that [`spawn`] and
/// [`rn_os_helper::rn_create_task`] use internally.
///
/// # Arguments
/// * `name`     – task name (up to 31 chars).
/// * `stack`    – stack depth in *words*.
/// * `priority` – FreeRTOS priority.
/// * `entry`    – the function to run.
/// * `param`    – opaque pointer passed to `entry`.
///
/// Returns a [`TaskHandle`].
pub fn spawn_raw(
    name: &str,
    stack: u32,
    priority: u32,
    entry: TaskEntry,
    param: *mut core::ffi::c_void,
) -> TaskHandle {
    let mut name_buf = [0u8; 32];
    let copy_len = name.len().min(31);
    name_buf[..copy_len].copy_from_slice(&name.as_bytes()[..copy_len]);

    // Box the entry+param pair so the trampoline can find them.
    struct RawTask {
        entry: TaskEntry,
        param: *mut core::ffi::c_void,
    }
    let raw_task = Box::into_raw(Box::new(RawTask { entry, param }));

    extern "C" fn raw_trampoline(p: *mut core::ffi::c_void) {
        let task: Box<RawTask> = unsafe { Box::from_raw(p as *mut RawTask) };
        (task.entry)(task.param);
        // RawTask dropped here, freeing the box.
        // FreeRTOS task functions must never return – delete ourselves.
        unsafe { rn_freertos_c::vTaskDelete(core::ptr::null_mut()) };
    }

    let mut handle: rn_freertos_c::TaskHandle_t = core::ptr::null_mut();
    unsafe {
        let ret = rn_freertos_c::xTaskCreate(
            Some(raw_trampoline),
            name_buf.as_ptr() as *const cty::c_char,
            stack,
            raw_task as *mut c_void,
            priority as rn_freertos_c::UBaseType_t,
            &mut handle,
        );
        assert!(ret != 0, "spawn_raw: xTaskCreate failed");
        TaskHandle::from_raw(handle)
    }
}

// ---------------------------------------------------------------------------
//  Instant (std‑like monotonic time)
// ---------------------------------------------------------------------------

/// A measurement of a monotonically‑non‑decreasing clock, modelled after
/// [`std::time::Instant`].
///
/// The underlying resolution is microseconds (provided by the hardware
/// timer via `lnGetUs64`).
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant {
    micros: u64,
}

impl Instant {
    /// Return a snapshot of the monotonic clock.
    pub fn now() -> Self {
        let micros = unsafe { rt::lnGetUs64() };
        Self { micros }
    }

    /// The elapsed time since this `Instant` was created.
    pub fn elapsed(&self) -> Duration {
        Duration::from_micros(unsafe { rt::lnGetUs64() } - self.micros)
    }

    /// Duration between `self` and `earlier` (`self - earlier`).
    ///
    /// # Panics
    /// Panics if `earlier` is later than `self`.
    pub fn duration_since(&self, earlier: &Instant) -> Duration {
        let diff = self
            .micros
            .checked_sub(earlier.micros)
            .expect("Instant::duration_since: earlier is later than self");
        Duration::from_micros(diff)
    }

    /// Checked version of `duration_since`.
    pub fn checked_duration_since(&self, earlier: &Instant) -> Option<Duration> {
        self.micros
            .checked_sub(earlier.micros)
            .map(Duration::from_micros)
    }

    /// Saturating version of `duration_since`.
    pub fn saturating_duration_since(&self, earlier: &Instant) -> Duration {
        if self.micros > earlier.micros {
            Duration::from_micros(self.micros - earlier.micros)
        } else {
            Duration::from_micros(0)
        }
    }

    /// Add a duration to this instant (panics on overflow).
    pub fn checked_add(&self, duration: Duration) -> Option<Self> {
        self.micros
            .checked_add(duration.as_micros() as u64)
            .map(|micros| Self { micros })
    }

    /// Subtract a duration from this instant (panics on overflow).
    pub fn checked_sub(&self, duration: Duration) -> Option<Self> {
        self.micros
            .checked_sub(duration.as_micros() as u64)
            .map(|micros| Self { micros })
    }

    /// Return the raw microsecond timestamp.
    pub fn as_micros(&self) -> u64 {
        self.micros
    }
}

impl core::ops::Add<Duration> for Instant {
    type Output = Instant;
    fn add(self, other: Duration) -> Instant {
        Self {
            micros: self.micros + other.as_micros() as u64,
        }
    }
}

impl core::ops::Sub<Duration> for Instant {
    type Output = Instant;
    fn sub(self, other: Duration) -> Instant {
        Self {
            micros: self.micros - other.as_micros() as u64,
        }
    }
}

/// A simple `Duration` type mirroring [`core::time::Duration`].
///
/// We re‑use `core::time::Duration` if available, but on older nightly
/// builds (edition 2024) we define it ourselves.
pub use core::time::Duration;

// ---- deprecated aliases for backward compatibility ----

/// Obsolete alias for [`delay_ms`].
#[deprecated(note = "use `delay_ms` instead")]
#[inline]
pub fn get_time_ms() -> u32 {
    time_ms()
}

/// Obsolete alias for [`time_us`].
#[deprecated(note = "use `time_us` instead")]
#[inline]
pub fn get_time_us() -> u32 {
    time_us()
}

// ---------------------------------------------------------------------------
//  Internal helpers
// ---------------------------------------------------------------------------

/// Default tick rate – adjust if your FreeRTOS config uses a different value.
const TICK_RATE_HZ: u32 = 1000;

/// Convert milliseconds to FreeRTOS ticks.
/// `u32::MAX` is interpreted as "infinite wait" (portMAX_DELAY).
pub(crate) fn ms_to_ticks(ms: u32) -> rn_freertos_c::TickType_t {
    if ms == u32::MAX {
        u32::MAX
    } else {
        (ms as u64 * TICK_RATE_HZ as u64 / 1000) as u32
    }
}
