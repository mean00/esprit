#![allow(dead_code)]
#![allow(unsafe_op_in_unsafe_fn)]

//! FFI bindings for lnMultiPulse_c.h (manually written because bindgen fails).

use crate::pin_types::lnPin;
use core::ffi::c_void;

/// Opaque handle to a C++ lnMultiPulse.
#[repr(C)]
pub struct ln_multi_pulse_c {
    _dummy: *mut c_void,
}

unsafe extern "C" {
    pub fn ln_multi_pulse_create(pin: lnPin, tick_fq_hz: i32) -> *mut ln_multi_pulse_c;
    pub fn ln_multi_pulse_delete(mp: *mut ln_multi_pulse_c);
    pub fn ln_multi_pulse_fire(mp: *mut ln_multi_pulse_c, pulse1_ms: i32, gap_ms: i32, pulse2_ms: i32);
    pub fn ln_multi_pulse_wait_done(mp: *mut ln_multi_pulse_c);
    pub fn ln_multi_pulse_stop(mp: *mut ln_multi_pulse_c);
}