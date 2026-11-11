#![allow(unused_variables, unused_imports, unused_parens)]
#![allow(non_camel_case_types, non_snake_case, dead_code)]

#![no_std]

use rs_gpio_bluepill::Pin;
use rs_mcu_traits::pwm;
use rs_mcu_traits::Result;

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

/// The `pwm::Pwm` contract of this family.
///
/// The hardware exists (a timer channel in one-pulse mode on the pin), but the
/// pure-Rust port of `ln_multi_pulse_fire` is not written: every method that would
/// touch it answers [`Error::Unsupported`](rs_mcu_traits::Error::Unsupported)
/// instead of reaching the `unimplemented!` bodies above, so a caller gets a value
/// it can handle. The C ABI in `shim.rs` still panics for the legacy callers.
impl pwm::Pwm for MultiPulse {
    fn configure(&mut self, _channel: u8, _config: &pwm::PwmConfig) -> Result<()> {
        Err(rs_mcu_traits::Error::Unsupported)
    }

    fn start(&mut self, _channel: u8) -> Result<()> {
        Err(rs_mcu_traits::Error::Unsupported)
    }

    fn stop(&mut self, _channel: u8) -> Result<()> {
        Ok(())
    }

    fn pulse_us(&mut self, _us: u32, _active_high: bool) -> Result<()> {
        Err(rs_mcu_traits::Error::Unsupported)
    }

    fn set_handler(&mut self, _handler: Option<&'static dyn pwm::PwmHandler>) {}
}

pub mod shim;
pub use shim::*;
