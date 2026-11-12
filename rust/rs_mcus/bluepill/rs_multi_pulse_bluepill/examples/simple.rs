//! PWM / single-pulse generation through the MCU-agnostic super trait.
//!
//! `rs_mcu_traits::pwm::Pwm` describes a waveform on a channel and the
//! single-pulse special case (`pulse_us`), which is what esprit's "multi pulse"
//! generator is for. The pure-Rust port of the GD32 one-pulse sequence is still
//! pending, so the methods answer `Error::Unsupported` today — the *shape* of the
//! call is what this example fixes.

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use rs_gpio_bluepill::Pin;
use rs_mcu_traits::pwm::{Pwm as PwmTrait, PwmConfig, PwmHandler};
use rs_multi_pulse_bluepill::MultiPulse;

/// The `no_std` example binary needs an allocator even though nothing allocates.
struct DummyAlloc;

unsafe impl core::alloc::GlobalAlloc for DummyAlloc {
    unsafe fn alloc(&self, _layout: core::alloc::Layout) -> *mut u8 {
        core::ptr::null_mut()
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {}
}

#[global_allocator]
static ALLOCATOR: DummyAlloc = DummyAlloc;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

/// Called when the armed pulse ends.
struct Pulse;

impl PwmHandler for Pulse {
    fn on_pulse_complete(&self) {}
}

static PULSE: Pulse = Pulse;

const CHANNEL: u8 = 0;
const FREQUENCY_HZ: u32 = 1_000_000;
const DUTY_PERMILLE: u16 = 500;
const PULSE_US: u32 = 50;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    // The generator's own constructor: a pin and the tick frequency.
    let mut pwm = MultiPulse::new(Pin::PA8, FREQUENCY_HZ as i32);
    PwmTrait::set_handler(&mut pwm, Some(&PULSE));

    let _ = PwmTrait::configure(
        &mut pwm,
        CHANNEL,
        &PwmConfig {
            frequency_hz: FREQUENCY_HZ,
            duty_permille: DUTY_PERMILLE,
        },
    );
    let _ = PwmTrait::start(&mut pwm, CHANNEL);
    let _ = PwmTrait::pulse_us(&mut pwm, PULSE_US, true);
    let _ = PwmTrait::stop(&mut pwm, CHANNEL);

    loop {
        core::hint::spin_loop();
    }
}


