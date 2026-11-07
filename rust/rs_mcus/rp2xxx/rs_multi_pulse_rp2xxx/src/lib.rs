//! RP2040 / RP2350 stub backend for the `lnMultiPulse` C ABI.
//!
//! esprit has no RP2xxx implementation of this peripheral: the C++ bridge
//! (`lnMultiPulse_c.cpp`) is deliberately *not* built for RP2040/RP2350 — see
//! the `if(NOT (USE_RP2040 OR USE_RP2350))` guard in
//! `esprit/rust/CMakeLists.txt`.
//!
//! This crate gives `rust_esprit` a linkable backend for RP2xxx anyway: it
//! exposes exactly the same public Rust entry points as
//! `rs_multi_pulse_bluepill` (and `c_api::rn_multi_pulse_c`), but every body is
//! `unimplemented!(...)`.  Reaching one therefore panics with a clear message
//! instead of producing an obscure undefined reference at link time.
//!
//! Nothing is expected to call these on RP2xxx: no C/C++ code there references
//! the `ln_multi_pulse_*` symbols.

#![no_std]
#![allow(unused_variables, non_camel_case_types, non_snake_case, dead_code)]

use core::ffi::c_void;

/// Opaque multi-pulse handle — same shape as `rs_multi_pulse_bluepill::ln_multi_pulse_c`.
#[repr(C)]
pub struct ln_multi_pulse_c {
    _dummy: *mut c_void,
}

/// Create a multi-pulse generator on `pin`, ticking at `tick_fq_hz`.
///
/// Generic over the pin type: the RP2xxx `lnPin` enum is generated inside
/// `rust_esprit`, so a sibling crate cannot name it (circular dependency).
/// The value is never used.
pub fn ln_multi_pulse_create<P>(pin: P, tick_fq_hz: i32) -> *mut ln_multi_pulse_c {
    unimplemented!("ln_multi_pulse_create is not supported on RP2040 / RP2350")
}

/// Fire a two-pulse sequence (pulse, gap, pulse), all in milliseconds.
pub fn ln_multi_pulse_fire(
    mp: *mut ln_multi_pulse_c,
    pulse1_ms: i32,
    gap_ms: i32,
    pulse2_ms: i32,
) {
    unimplemented!("ln_multi_pulse_fire is not supported on RP2040 / RP2350")
}

/// Block until the running sequence has completed.
pub fn ln_multi_pulse_wait_done(mp: *mut ln_multi_pulse_c) {
    unimplemented!("ln_multi_pulse_wait_done is not supported on RP2040 / RP2350")
}

/// Abort the running sequence.
pub fn ln_multi_pulse_stop(mp: *mut ln_multi_pulse_c) {
    unimplemented!("ln_multi_pulse_stop is not supported on RP2040 / RP2350")
}

/// Destroy a generator created by [`ln_multi_pulse_create`].
pub fn ln_multi_pulse_delete(mp: *mut ln_multi_pulse_c) {
    unimplemented!("ln_multi_pulse_delete is not supported on RP2040 / RP2350")
}
