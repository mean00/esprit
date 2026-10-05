#![no_std]
#![no_main]

use core::panic::PanicInfo;
use rs_esprit::adc::{AdcHandler, SimpleAdc, TimingAdc};
use rs_esprit::gpio::{EdgeTrigger, GpioInterruptHandler, GpioPin};

/// Application handler responding to external pin edge interrupts.
struct ButtonHandler;

impl GpioInterruptHandler for ButtonHandler {
    fn on_edge(&self, pin_state: bool) {
        let _ = pin_state;
    }
}

/// Application handler responding to ADC conversion events.
struct AdcDataHandler;

impl AdcHandler for AdcDataHandler {
    fn on_conversion_complete(&self, sample: u16) {
        let _ = sample;
    }

    fn on_sequence_complete(&self, samples: &[u16]) {
        let _ = samples.len();
    }
}

static BTN_HANDLER: ButtonHandler = ButtonHandler;
static ADC_HANDLER: AdcDataHandler = AdcDataHandler;

/// Demonstrates using `GpioPin` and `SimpleAdc` / `TimingAdc` traits.
pub fn run_gpio_adc_demo<P: GpioPin, A: SimpleAdc, T: TimingAdc>(
    mut led_pin: P,
    mut btn_pin: P,
    mut simple_adc: A,
    mut timing_adc: T,
) {
    // GPIO demo
    led_pin.write(true);
    led_pin.toggle();
    let is_high = led_pin.read();
    let _ = is_high;

    btn_pin.enable_interrupt(EdgeTrigger::Falling);
    // Completely safe registration - zero unsafe!
    btn_pin.set_handler(Some(&BTN_HANDLER));

    // ADC demo - completely safe registration!
    simple_adc.set_handler(Some(&ADC_HANDLER));
    timing_adc.set_handler(Some(&ADC_HANDLER));

    let val = simple_adc.read();
    let _ = val;

    let mut samples = [0u16; 16];
    timing_adc.multi_read(4, &mut samples);
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
