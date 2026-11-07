#![allow(unused_variables, unused_imports, unused_parens)]
#![allow(non_camel_case_types, non_snake_case, dead_code)]

#![no_std]

use rs_gpio_bluepill::Pin;

/// Idiomatic struct representing a multi-pulse pulse generator on a GPIO pin.
pub struct MultiPulse {
    pub pin: Pin,
    pub tick_fq_hz: i32,
}

impl MultiPulse {
    pub fn new(pin: Pin, tick_fq_hz: i32) -> Self {
        Self { pin, tick_fq_hz }
    }

    pub fn fire(&mut self, pulse1_ms: i32, gap_ms: i32, pulse2_ms: i32) {
        unimplemented!("Port ln_multi_pulse_fire to pure Rust")
    }

    pub fn wait_done(&mut self) {
        unimplemented!("Port ln_multi_pulse_wait_done to pure Rust")
    }

    pub fn stop(&mut self) {
        unimplemented!("Port ln_multi_pulse_stop to pure Rust")
    }
}

pub mod shim;
pub use shim::*;
