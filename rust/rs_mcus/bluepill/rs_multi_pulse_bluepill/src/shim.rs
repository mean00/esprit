use crate::*;
use rs_gpio_bluepill::lnPin;
use core::ffi::c_void;

#[repr(C)]
pub struct ln_multi_pulse_c {
    _dummy: *mut c_void,
}

pub fn ln_multi_pulse_create(_pin: lnPin, _tick_fq_hz: i32) -> *mut ln_multi_pulse_c {
    core::ptr::null_mut()
}

pub fn ln_multi_pulse_fire(
    _mp: *mut ln_multi_pulse_c,
    _pulse1_ms: i32,
    _gap_ms: i32,
    _pulse2_ms: i32,
) {}

pub fn ln_multi_pulse_wait_done(_mp: *mut ln_multi_pulse_c) {}

pub fn ln_multi_pulse_stop(_mp: *mut ln_multi_pulse_c) {}

pub fn ln_multi_pulse_delete(_mp: *mut ln_multi_pulse_c) {}
