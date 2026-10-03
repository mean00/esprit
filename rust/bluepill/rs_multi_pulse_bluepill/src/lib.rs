#![allow(unused_variables, unused_imports, unused_parens)]
#![allow(non_camel_case_types, non_snake_case, dead_code)]

#![no_std]

use rs_gpio_bluepill::lnPin;
use core::ffi::c_void;

#[repr(C)]
pub struct ln_multi_pulse_c {
    _dummy: *mut c_void,
}

pub fn ln_multi_pulse_create(pin: lnPin, tick_fq_hz: i32) -> *mut ln_multi_pulse_c {
    unimplemented!("Port ln_multi_pulse_create to pure Rust")
}

pub fn ln_multi_pulse_fire(
    mp: *mut ln_multi_pulse_c,
    pulse1_ms: i32,
    gap_ms: i32,
    pulse2_ms: i32,
) {
    unimplemented!("Port ln_multi_pulse_fire to pure Rust")
}

pub fn ln_multi_pulse_wait_done(mp: *mut ln_multi_pulse_c) {
    unimplemented!("Port ln_multi_pulse_wait_done to pure Rust")
}

pub fn ln_multi_pulse_stop(mp: *mut ln_multi_pulse_c) {
    unimplemented!("Port ln_multi_pulse_stop to pure Rust")
}

pub fn ln_multi_pulse_delete(mp: *mut ln_multi_pulse_c) {
    unimplemented!("Port ln_multi_pulse_delete to pure Rust")
}
