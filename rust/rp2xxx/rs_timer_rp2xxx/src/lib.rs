//! RP2040 / RP2350 stub backend for the `lnTimer` and hardware-stopwatch C ABI.
//!
//! esprit has no RP2xxx implementation of these peripherals: the C++ bridges
//! that provide them (`lnTimer_c.cpp`) are deliberately *not* built for
//! RP2040/RP2350 — see the `if(NOT (USE_RP2040 OR USE_RP2350))` guard in
//! `esprit/rust/CMakeLists.txt`.
//!
//! This crate gives `rust_esprit` a linkable backend for RP2xxx anyway: it
//! exposes exactly the same public Rust entry points as the real backends
//! (`rs_timer_bluepill` on GD32, `c_api::rn_timer_c` on ESP32), but every body
//! is `unimplemented!(...)`.  Reaching one therefore panics with a clear
//! message instead of producing an obscure `undefined reference to
//! '_Z18ln_timer_create_from_pin...'` at link time.
//!
//! Nothing is expected to call these on RP2xxx: no C/C++ code there references
//! the `ln_timer_*` / `ln_hw_stopwatch_*` symbols (the C++ consumers live in
//! `swindle/src/platform/ln`, which is not used for RP2040/RP2350 —
//! `swindle/src/platform/rp2040` is).

#![no_std]
#![allow(unused_variables, non_camel_case_types, non_snake_case, dead_code)]

use core::ffi::c_void;

/// Opaque timer handle — same shape as the C++ `ln_timer_c` and as
/// `rs_timer_bluepill::ln_timer_c`.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ln_timer_c {
    pub dummy: *mut c_void,
}

/// Create a timer from a pin.
///
/// Generic over the pin type: the RP2xxx `lnPin` enum is generated inside
/// `rust_esprit`, so a sibling crate cannot name it (circular dependency).
/// The value is never used, no panic if called.
pub fn ln_timer_create_from_pin<P>(_pin: P) -> *mut ln_timer_c {
    unimplemented!("ln_timer_create_from_pin is not supported on RP2040 / RP2350")
}

/// Create a timer from a timer index and channel.
#[unsafe(no_mangle)]
pub extern "C" fn ln_timer_create(timer: u32, channel: u32) -> *mut ln_timer_c {
    unimplemented!("ln_timer_create is not supported on RP2040 / RP2350")
}

/// Destroy a timer created by [`ln_timer_create_from_pin`] / [`ln_timer_create`].
#[unsafe(no_mangle)]
pub extern "C" fn ln_timer_delete(timer: *mut ln_timer_c) {
    unimplemented!("ln_timer_delete is not supported on RP2040 / RP2350")
}

/// Generate a single-shot pulse of `durationMs` milliseconds.
#[unsafe(no_mangle)]
pub extern "C" fn ln_timer_single_shot(timer: *mut ln_timer_c, durationMs: u32, up: bool) {
    unimplemented!("ln_timer_single_shot is not supported on RP2040 / RP2350")
}

// ---------------------------------------------------------------------------
//  Hardware stopwatch (same backend crate as the timer, mirroring
//  rs_timer_bluepill which provides both)
// ---------------------------------------------------------------------------

/// Create a hardware stopwatch on the given timer index.
#[unsafe(no_mangle)]
pub extern "C" fn ln_hw_stopwatch_create(timer_index: u32) -> *mut c_void {
    unimplemented!("ln_hw_stopwatch_create is not supported on RP2040 / RP2350")
}

/// Configure the stopwatch for one-shot-ish use.
#[unsafe(no_mangle)]
pub extern "C" fn ln_hw_stopwatch_setup(sw: *mut c_void) {
    unimplemented!("ln_hw_stopwatch_setup is not supported on RP2040 / RP2350")
}

/// Start the stopwatch.
#[unsafe(no_mangle)]
pub extern "C" fn ln_hw_stopwatch_start(sw: *mut c_void) {
    unimplemented!("ln_hw_stopwatch_start is not supported on RP2040 / RP2350")
}

/// Busy-wait `ticks` stopwatch ticks.
#[unsafe(no_mangle)]
pub extern "C" fn ln_hw_stopwatch_wait(sw: *mut c_void, ticks: u16) {
    unimplemented!("ln_hw_stopwatch_wait is not supported on RP2040 / RP2350")
}

/// Destroy a stopwatch created by [`ln_hw_stopwatch_create`].
#[unsafe(no_mangle)]
pub extern "C" fn ln_hw_stopwatch_destroy(sw: *mut c_void) {
    unimplemented!("ln_hw_stopwatch_destroy is not supported on RP2040 / RP2350")
}

// ---------------------------------------------------------------------------
//  Delay Timer (DelayTimer C ABI stubs)
// ---------------------------------------------------------------------------

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ln_delay_timer_c {
    pub dummy: *mut c_void,
}

pub type DelayTimerCallback = Option<unsafe extern "C" fn(cookie: *mut c_void)>;

#[unsafe(no_mangle)]
pub extern "C" fn ln_delay_timer_create(timer: i32, channel: i32) -> *mut ln_delay_timer_c {
    unimplemented!("ln_delay_timer_create is not supported on RP2040 / RP2350")
}

#[unsafe(no_mangle)]
pub extern "C" fn ln_delay_timer_delete(timer: *mut ln_delay_timer_c) {
    unimplemented!("ln_delay_timer_delete is not supported on RP2040 / RP2350")
}

#[unsafe(no_mangle)]
pub extern "C" fn ln_delay_timer_arm(timer: *mut ln_delay_timer_c, delay_us: i32) {
    unimplemented!("ln_delay_timer_arm is not supported on RP2040 / RP2350")
}

#[unsafe(no_mangle)]
pub extern "C" fn ln_delay_timer_set_interrupt(
    timer: *mut ln_delay_timer_c,
    handler: DelayTimerCallback,
    cookie: *mut c_void,
) {
    unimplemented!("ln_delay_timer_set_interrupt is not supported on RP2040 / RP2350")
}

#[unsafe(no_mangle)]
pub extern "C" fn ln_delay_timer_enable_interrupt(timer: *mut ln_delay_timer_c) {
    unimplemented!("ln_delay_timer_enable_interrupt is not supported on RP2040 / RP2350")
}

#[unsafe(no_mangle)]
pub extern "C" fn ln_delay_timer_disable_interrupt(timer: *mut ln_delay_timer_c) {
    unimplemented!("ln_delay_timer_disable_interrupt is not supported on RP2040 / RP2350")
}

#[unsafe(no_mangle)]
pub extern "C" fn ln_delay_timer_irq(timer: *mut ln_delay_timer_c) {
    unimplemented!("ln_delay_timer_irq is not supported on RP2040 / RP2350")
}

#[unsafe(no_mangle)]
pub extern "C" fn ln_delay_timer_interrupt_handler(timer: i32) {
    unimplemented!("ln_delay_timer_interrupt_handler is not supported on RP2040 / RP2350")
}
