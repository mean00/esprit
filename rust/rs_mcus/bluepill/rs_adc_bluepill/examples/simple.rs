//! ADC through the MCU-agnostic super traits.
//!
//! * `rs_mcu_traits::adc::Adc` — one pin, one conversion ([`SimpleAdc`]).
//! * `rs_mcu_traits::adc::ScanAdc` — N sources × M samples, DMA driven
//!   ([`TimingAdc`]).
//!
//! No `ln*_c` handle anywhere: the driver *is* the ADC.

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use rs_adc_bluepill::{SimpleAdc, TimingAdc};
use rs_gpio_bluepill::Pin;
use rs_mcu_traits::adc::{Adc, AdcConfig, AdcHandler, ScanAdc, ScanConfig};

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

/// The completion of a scan runs in the DMA interrupt, so the handler only touches
/// atomics.
struct Samples;

impl AdcHandler for Samples {
    fn on_scan_complete(&self, _samples: &[u16]) {}
}

static SAMPLES: Samples = Samples;

const SAMPLE_TIME_NS: u32 = 4_000; // the 239.5-cycle GD32 window at 72 MHz
const SAMPLE_RATE_HZ: u32 = 10_000;
const SAMPLES_PER_CHANNEL: usize = 64;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    // ---- one pin, one conversion -------------------------------------------
    let mut sense = SimpleAdc::new(0, Pin::PA0 as u32);
    let _ = Adc::configure(
        &mut sense,
        &AdcConfig {
            sample_time_ns: SAMPLE_TIME_NS,
            averaging: 4,
        },
    );
    let _level = Adc::read(&mut sense).unwrap_or(0);

    // ---- one source, 64 samples, blocking ----------------------------------
    let mut scan = TimingAdc::new(0);
    let sources = [Pin::PA1 as u32]; // ascending channel order is the contract
    if ScanAdc::configure(
        &mut scan,
        &ScanConfig {
            sources: &sources,
            fq_hz: SAMPLE_RATE_HZ,
        },
    )
    .is_ok()
    {
        let mut buffer = [0u16; SAMPLES_PER_CHANNEL];
        let filled = ScanAdc::read(&mut scan, &mut buffer).unwrap_or(0);
        let _first = buffer.get(filled.min(1));

        // ---- the same read, asynchronous: the handler above runs in the DMA
        // interrupt once the buffer is full.
        ScanAdc::set_handler(&mut scan, Some(&SAMPLES));
        let mut async_buffer = [0u16; SAMPLES_PER_CHANNEL];
        let _ = ScanAdc::start_async(&mut scan, &mut async_buffer);
    }

    loop {
        core::hint::spin_loop();
    }
}

