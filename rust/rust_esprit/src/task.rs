#![allow(dead_code)]

use crate::rn_timer_c as rt;
use alloc::boxed::Box;
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

/// Return a monotonic millisecond counter.
#[inline]
pub fn time_ms() -> u32 {
    unsafe { rt::lnGetMs() }
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

// ---- task creation ----

/// Spawn a FreeRTOS task from a `FnOnce` closure.
///
/// # Parameters
/// * `name`     – printable task name (for debugging).
/// * `stack`    – stack depth in *words* (typically 256..1024).
/// * `priority` – FreeRTOS priority (higher = more urgent).
/// * `f`        – the closure to execute.  Must be `Send + 'static`.
pub fn spawn<F>(name: &str, stack: u32, priority: u32, f: F)
where
    F: FnOnce() + Send + 'static,
{
    // Box the closure so it lives on the heap.
    let closure: Box<dyn FnOnce() + Send> = Box::new(f);
    let raw = Box::into_raw(Box::new(closure));

    extern "C" fn trampoline(param: *mut c_void) {
        // Reconstruct the closure from the raw pointer and call it.
        let closure: Box<Box<dyn FnOnce() + Send>> = unsafe { Box::from_raw(param as *mut _) };
        closure();
    }

    // Null-terminate the name (C string).
    // heapless::String would be cleaner but we want to avoid pulling it in
    // for just name handling.  A stack buffer works since FreeRTOS copies
    // the name internally.
    let mut name_buf = [0u8; 32];
    let copy_len = name.len().min(31);
    name_buf[..copy_len].copy_from_slice(&name.as_bytes()[..copy_len]);

    unsafe {
        crate::rn_freertos_c::lnCreateTask(
            Some(trampoline),
            name_buf.as_ptr() as *const cty::c_char,
            stack as i32,
            raw as *mut c_void,
            priority as crate::rn_freertos_c::UBaseType_t,
        );
    }
}

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