//! Legacy compatibility shim for `rn_os_helper`.
//!
//! All functions delegate to the idiomatic wrappers in [`crate::task`] and
//! [`crate::sync`] or call the underlying FreeRTOS FFI directly.  New code
//! should use the `task` / `sync` / `queue` modules directly.
#![allow(dead_code)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![deprecated(since = "0.0.0", note = "use `crate::task` and `crate::sync` instead")]

use crate::prelude::*;
use crate::rn_freertos_c;
use crate::sync;
use crate::task;
use core::ffi::c_void;

/// Default tick rate – adjust if your FreeRTOS config uses a different value.
const TICK_RATE_HZ: u32 = 1000;

// ===========================================================================
//  Time / delay
// ===========================================================================

/// Block for `ms` milliseconds.
#[inline]
pub fn delay_ms(to: u32) {
    task::delay_ms(to)
}

/// Block for `us` microseconds.
#[inline]
pub fn delay_us(to: u32) {
    task::delay_us(to)
}

/// Return a monotonic millisecond counter.
#[inline]
pub fn get_time_ms() -> u32 {
    task::time_ms()
}

/// Return a monotonic microsecond counter.
#[inline]
pub fn get_time_us() -> u32 {
    task::time_us()
}

// ===========================================================================
//  Task types
// ===========================================================================

/// Legacy task entry function pointer type.
pub type rnTaskEntry = crate::task::TaskEntry;

/// Raw FreeRTOS task handle (`TaskHandle_t`).
pub type rnTaskHandle = rn_freertos_c::TaskHandle_t;

/// Opaque handle to a FreeRTOS mutex (same as `SemaphoreHandle_t` /
/// `QueueHandle_t` in FreeRTOS internals).
pub type rnMutexHandle = rn_freertos_c::SemaphoreHandle_t;

/// Opaque handle to a FreeRTOS semaphore.
pub type rnSemaphoreHandle = rn_freertos_c::SemaphoreHandle_t;

/// Opaque handle to a FreeRTOS queue.
pub type rnQueueHandle = rn_freertos_c::QueueHandle_t;

// ===========================================================================
//  Task creation / management
// ===========================================================================

/// Create a FreeRTOS task (legacy API, discards the handle).
///
/// This is a thin wrapper around [`task::spawn_raw`] that accepts a
/// `&'static rnTaskEntry` function pointer.
pub fn rn_create_task(
    function_entry: &'static rnTaskEntry,
    name: &str,
    priority: usize,
    stack_size: u32,
    param: *mut core::ffi::c_void,
) {
    task::spawn_raw(name, stack_size, priority as u32, *function_entry, param);
}

/// Create a FreeRTOS task and return its raw handle.
///
/// The caller is responsible for eventually calling [`rn_task_delete`] on the
/// returned handle, or allowing the task to run to completion (after which the
/// handle becomes invalid).
pub fn rn_create_task_ext(
    function_entry: &'static rnTaskEntry,
    name: &str,
    priority: usize,
    stack_size: u32,
    param: *mut core::ffi::c_void,
) -> rnTaskHandle {
    task::spawn_raw(name, stack_size, priority as u32, *function_entry, param).raw()
}

/// Return the handle of the currently running task.
#[inline]
pub fn rn_get_current_task() -> rnTaskHandle {
    unsafe { rn_freertos_c::xTaskGetCurrentTaskHandle() }
}

/// Suspend a task.  The suspended task will not execute until resumed.
#[inline]
pub fn rn_task_suspend(handle: rnTaskHandle) {
    unsafe { rn_freertos_c::vTaskSuspend(handle) }
}

/// Resume a suspended task.
#[inline]
pub fn rn_task_resume(handle: rnTaskHandle) {
    unsafe { rn_freertos_c::vTaskResume(handle) }
}

/// Delete a task.  After this call the handle is invalid.
///
/// If called on the currently running task, deletion occurs after the task
/// function returns.
#[inline]
pub fn rn_task_delete(handle: rnTaskHandle) {
    unsafe { rn_freertos_c::vTaskDelete(handle) }
}

/// Yield the processor to other ready tasks at the same priority.
#[inline]
pub fn rn_task_yield() {
    unsafe { rn_freertos_c::vTaskDelay(0) }
}

/// Get a task's current priority.
#[inline]
pub fn rn_task_get_priority(handle: rnTaskHandle) -> u32 {
    unsafe { rn_freertos_c::uxTaskPriorityGet(handle) as u32 }
}

/// Get a task's name (NUL-terminated, up to `configMAX_TASK_NAME_LEN`).
pub fn rn_task_get_name(handle: rnTaskHandle) -> &'static str {
    unsafe {
        let ptr = rn_freertos_c::pcTaskGetName(handle);
        let len = (0..).find(|&i| *ptr.add(i) == 0).unwrap_or(0);
        core::str::from_utf8_unchecked(core::slice::from_raw_parts(ptr as *const u8, len))
    }
}

// ===========================================================================
//  Mutex (non‑recursive)
// ===========================================================================

/// Create a non‑recursive mutex.
///
/// Returns a handle, or `null` on allocation failure.  The caller must
/// eventually call [`rn_mutex_delete`] on the handle.
pub fn rn_mutex_create() -> rnMutexHandle {
    unsafe { rn_freertos_c::xQueueCreateMutex(0) }
}

/// Lock a mutex, blocking up to `timeout_ms` milliseconds.
///
/// Pass `u32::MAX` to wait indefinitely.  Returns `true` if the lock was
/// acquired.
pub fn rn_mutex_lock(handle: rnMutexHandle, timeout_ms: u32) -> bool {
    unsafe { rn_freertos_c::xQueueSemaphoreTake(handle, ms_to_ticks(timeout_ms)) != 0 }
}

/// Try to lock a mutex without blocking.
#[inline]
pub fn rn_mutex_try_lock(handle: rnMutexHandle) -> bool {
    unsafe { rn_freertos_c::xQueueSemaphoreTake(handle, 0) != 0 }
}

/// Unlock (release) a mutex.
#[inline]
pub fn rn_mutex_unlock(handle: rnMutexHandle) {
    unsafe {
        rn_freertos_c::xQueueGenericSend(handle, core::ptr::null::<c_void>(), 0, 0);
    }
}

/// Delete a mutex and free its kernel resources.
#[inline]
pub fn rn_mutex_delete(handle: rnMutexHandle) {
    unsafe { rn_freertos_c::vQueueDelete(handle) }
}

// ===========================================================================
//  Recursive mutex
// ===========================================================================

/// Create a recursive mutex (the same task can lock it repeatedly).
pub fn rn_recursive_mutex_create() -> rnMutexHandle {
    unsafe { rn_freertos_c::xQueueCreateMutex(1) }
}

/// Lock a recursive mutex, blocking up to `timeout_ms` milliseconds.
///
/// Returns `true` if the lock was acquired.
pub fn rn_recursive_mutex_lock(handle: rnMutexHandle, timeout_ms: u32) -> bool {
    unsafe { rn_freertos_c::xQueueTakeMutexRecursive(handle, ms_to_ticks(timeout_ms)) != 0 }
}

/// Unlock a recursive mutex (one level).  Each `lock` must be paired with
/// an `unlock`.
#[inline]
pub fn rn_recursive_mutex_unlock(handle: rnMutexHandle) {
    unsafe { rn_freertos_c::xQueueGiveMutexRecursive(handle); }
}

// ===========================================================================
//  Semaphore (binary & counting)
// ===========================================================================

/// Create a binary semaphore, initially **not** given (take will block).
pub fn rn_binary_semaphore_create() -> rnSemaphoreHandle {
    unsafe { rn_freertos_c::xQueueCreateCountingSemaphore(1, 0 as _) }
}

/// Create a binary semaphore that starts in the **given** (signalled) state.
pub fn rn_binary_semaphore_create_given() -> rnSemaphoreHandle {
    unsafe { rn_freertos_c::xQueueCreateCountingSemaphore(1, 1 as _) }
}

/// Create a counting semaphore with a configurable maximum count and
/// initial count.
pub fn rn_counting_semaphore_create(max_count: u32, initial_count: u32) -> rnSemaphoreHandle {
    unsafe { rn_freertos_c::xQueueCreateCountingSemaphore(max_count as _, initial_count as _) }

}

/// Give (signal / increment) a semaphore.
///
/// Returns `true` on success.
#[inline]
pub fn rn_semaphore_give(handle: rnSemaphoreHandle) -> bool {
    unsafe {
        rn_freertos_c::xQueueGenericSend(handle, core::ptr::null::<c_void>(), 0, 0) != 0
    }
}

/// Take (wait for) a semaphore, blocking up to `timeout_ms` milliseconds.
///
/// Pass `u32::MAX` to wait indefinitely.  Returns `true` if the semaphore
/// was acquired.
pub fn rn_semaphore_take(handle: rnSemaphoreHandle, timeout_ms: u32) -> bool {
    unsafe { rn_freertos_c::xQueueSemaphoreTake(handle, ms_to_ticks(timeout_ms)) != 0 }
}

/// Try to take a semaphore without blocking.  Returns `true` if acquired.
#[inline]
pub fn rn_semaphore_try_take(handle: rnSemaphoreHandle) -> bool {
    unsafe { rn_freertos_c::xQueueSemaphoreTake(handle, 0) != 0 }
}

/// Delete a semaphore and free its kernel resources.
#[inline]
pub fn rn_semaphore_delete(handle: rnSemaphoreHandle) {
    unsafe { rn_freertos_c::vQueueDelete(handle) }
}

// ===========================================================================
//  Queue
// ===========================================================================

/// Create a queue that holds up to `capacity` items, each `item_size` bytes.
///
/// Returns a handle, or `null` on allocation failure.  The caller must
/// eventually call [`rn_queue_delete`].
pub fn rn_queue_create(item_size: u32, capacity: u32) -> rnQueueHandle {
    unsafe { rn_freertos_c::xQueueGenericCreate(capacity as _, item_size as _, 0) }
}

/// Send an item to the back of a queue, blocking up to `timeout_ms`.
///
/// `item` points to the data to copy (must be `item_size` bytes as declared
/// at creation).  Pass `u32::MAX` for infinite wait.  Returns `true` on
/// success.
pub fn rn_queue_send(handle: rnQueueHandle, item: *const c_void, timeout_ms: u32) -> bool {
    unsafe { rn_freertos_c::xQueueGenericSend(handle, item, ms_to_ticks(timeout_ms), 0) != 0 }
}

/// Try to send an item without blocking.  Returns `true` on success.
#[inline]
pub fn rn_queue_try_send(handle: rnQueueHandle, item: *const c_void) -> bool {
    unsafe { rn_freertos_c::xQueueGenericSend(handle, item, 0, 0) != 0 }
}

/// Send an item to the **back** of a queue (alias for [`rn_queue_send`]).
pub fn rn_queue_send_to_back(handle: rnQueueHandle, item: *const c_void, timeout_ms: u32) -> bool {
    unsafe { rn_freertos_c::xQueueGenericSend(handle, item, ms_to_ticks(timeout_ms), 0) != 0 }
}

/// Send an item to the **front** of a queue (like a LIFO push), blocking
/// up to `timeout_ms`.
pub fn rn_queue_send_to_front(handle: rnQueueHandle, item: *const c_void, timeout_ms: u32) -> bool {
    // xCopyPosition = 1 means "send to front"
    unsafe { rn_freertos_c::xQueueGenericSend(handle, item, ms_to_ticks(timeout_ms), 1) != 0 }
}

/// Receive an item from a queue, blocking up to `timeout_ms`.
///
/// `buffer` points to where the received data will be copied (must be at
/// least `item_size` bytes).  Returns `true` on success.
pub fn rn_queue_receive(handle: rnQueueHandle, buffer: *mut c_void, timeout_ms: u32) -> bool {
    unsafe { rn_freertos_c::xQueueReceive(handle, buffer, ms_to_ticks(timeout_ms)) != 0 }
}

/// Try to receive an item without blocking.  Returns `true` on success.
#[inline]
pub fn rn_queue_try_receive(handle: rnQueueHandle, buffer: *mut c_void) -> bool {
    unsafe { rn_freertos_c::xQueueReceive(handle, buffer, 0) != 0 }
}

/// Peek at the front item without removing it, blocking up to `timeout_ms`.
///
/// Returns `true` if an item was available.
pub fn rn_queue_peek(handle: rnQueueHandle, buffer: *mut c_void, timeout_ms: u32) -> bool {
    unsafe { rn_freertos_c::xQueuePeek(handle, buffer, ms_to_ticks(timeout_ms)) != 0 }
}

/// Return the number of items currently in the queue.
#[inline]
pub fn rn_queue_length(handle: rnQueueHandle) -> u32 {
    unsafe { rn_freertos_c::uxQueueMessagesWaiting(handle) as u32 }
}

/// Return the number of free slots in the queue.
#[inline]
pub fn rn_queue_available(handle: rnQueueHandle) -> u32 {
    unsafe { rn_freertos_c::uxQueueSpacesAvailable(handle) as u32 }
}

/// Reset the queue to its initial empty state.
#[inline]
pub fn rn_queue_reset(handle: rnQueueHandle) {
    unsafe { rn_freertos_c::xQueueGenericReset(handle, 0); }
}

/// Delete a queue and free its kernel resources.
#[inline]
pub fn rn_queue_delete(handle: rnQueueHandle) {
    unsafe { rn_freertos_c::vQueueDelete(handle) }
}

// ===========================================================================
//  Task notifications (FreeRTOS direct‑to‑task notifications, index 0)
// ===========================================================================

/// Send a notification value to a task (sets bits via `eSetBits`).
///
/// Returns `true` on success.
#[inline]
pub fn rn_notify_send(handle: rnTaskHandle, value: u32) -> bool {
    unsafe {
        rn_freertos_c::xTaskGenericNotify(
            handle,
            0,
            value,
            rn_freertos_c::eNotifyAction_eSetBits,
            core::ptr::null_mut(),
        ) != 0
    }
}

/// Send a notification value to a task with overwrite
/// (`eSetValueWithOverwrite` – always succeeds).
#[inline]
pub fn rn_notify_value(handle: rnTaskHandle, value: u32) {
    unsafe {
        rn_freertos_c::xTaskGenericNotify(
            handle,
            0,
            value,
            rn_freertos_c::eNotifyAction_eSetValueWithOverwrite,
            core::ptr::null_mut(),
        );
    }
}

/// Wait for a notification (index 0).
///
/// * `clear_on_entry` – bits to clear on entry.
/// * `clear_on_exit`  – bits to clear on exit.
/// * `timeout_ms`     – maximum wait (or `u32::MAX` for infinite).
///
/// Returns `Ok(value)` on success, `Err(())` on timeout.
pub fn rn_notify_wait(
    clear_on_entry: u32,
    clear_on_exit: u32,
    timeout_ms: u32,
) -> Result<u32, ()> {
    let ticks = ms_to_ticks(timeout_ms);
    let mut val: u32 = 0;
    let ret = unsafe {
        rn_freertos_c::xTaskGenericNotifyWait(0, clear_on_entry, clear_on_exit, &mut val, ticks)
    };
    if ret != 0 {
        Ok(val)
    } else {
        Err(())
    }
}

/// Binary notification take – decrements the notification count if > 0,
/// otherwise blocks up to `timeout_ms`.
///
/// Returns `Ok(value)` on success, `Err(())` on timeout.
pub fn rn_notify_take(clear_count: bool, timeout_ms: u32) -> Result<u32, ()> {
    let ticks = ms_to_ticks(timeout_ms);
    let clear = if clear_count { 1 } else { 0 };
    let ret = unsafe { rn_freertos_c::ulTaskGenericNotifyTake(0, clear, ticks) };
    if ret != 0 {
        Ok(ret)
    } else {
        Err(())
    }
}

// ===========================================================================
//  Critical sections
// ===========================================================================

/// Enter a critical section (disables interrupts on this core).
///
/// Must be paired with [`rn_exit_critical`].  Critical sections may nest.
#[inline]
pub fn rn_enter_critical() {
    unsafe { rn_freertos_c::vPortEnterCritical() }
}

/// Exit a critical section (re‑enables interrupts).
#[inline]
pub fn rn_exit_critical() {
    unsafe { rn_freertos_c::vPortExitCritical() }
}

// ===========================================================================
//  Scheduler control
// ===========================================================================

/// Suspend the RTOS scheduler (does **not** disable interrupts).
///
/// While suspended, no context switches will occur.  Must be paired with
/// [`rn_resume_scheduler`].
#[inline]
pub fn rn_suspend_scheduler() {
    unsafe { rn_freertos_c::vTaskSuspendAll() }
}

/// Resume the RTOS scheduler after [`rn_suspend_scheduler`].
///
/// Returns `true` if a context switch is pending (a yield was requested
/// while the scheduler was suspended).
#[inline]
pub fn rn_resume_scheduler() -> bool {
    unsafe { rn_freertos_c::xTaskResumeAll() != 0 }
}

/// Return the scheduler state: 0 = not started, 1 = running, 2 = suspended.
#[inline]
pub fn rn_scheduler_state() -> i32 {
    unsafe { rn_freertos_c::xTaskGetSchedulerState() as i32 }
}

// ===========================================================================
//  Internal helpers
// ===========================================================================

/// Convert milliseconds to FreeRTOS ticks.
///
/// `u32::MAX` is interpreted as "infinite wait" (`portMAX_DELAY`).
fn ms_to_ticks(ms: u32) -> rn_freertos_c::TickType_t {
    if ms == u32::MAX {
        u32::MAX
    } else {
        (ms as u64 * TICK_RATE_HZ as u64 / 1000) as u32
    }
}
