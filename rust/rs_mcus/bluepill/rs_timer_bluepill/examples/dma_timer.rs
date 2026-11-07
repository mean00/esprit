//! High-throughput PWM with circular DMA transfer example (`DmaTimer`).
//!
//! Follows Esprit Rust Rule 1 (No Magic Numbers), Rule 2 (Two-Tier Architecture),
//! Rule 4 (Mandatory Examples), and Rule 6 (Idiomatic Rust & C++ Parity).
//!
//! Demonstrates streaming compare values to a timer channel via DMA to generate
//! arbitrary pulse sequences (e.g. for WS2812B LEDs or stepped waveforms).

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use rs_gpio_bluepill::Pin;
use rs_timer_bluepill::DmaTimer;

struct DummyAlloc;
unsafe impl core::alloc::GlobalAlloc for DummyAlloc {
    unsafe fn alloc(&self, _layout: core::alloc::Layout) -> *mut u8 {
        core::ptr::null_mut()
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {}
}

#[global_allocator]
static ALLOCATOR: DummyAlloc = DummyAlloc;

const PWM_FREQUENCY_HZ: u32 = 800_000; // 800 kHz (standard WS2812 carrier frequency)
const BUFFER_LEN: usize = 16;

// Example compare value buffer to modulate PWM pulse width
static COMPARE_BUFFER: [u8; BUFFER_LEN] = [
    30, 60, 30, 60, 30, 60, 30, 60,
    30, 60, 30, 60, 30, 60, 30, 60,
];

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    // PB8 automatically maps to Timer 3, Channel 2, DMA0 Channel 4 via pinout resolution
    if let Ok(mut dma_timer) = DmaTimer::new(Pin::PB8) {
        // Configure PWM frequency
        if dma_timer.pwm_setup(PWM_FREQUENCY_HZ).is_ok() {
            // Start streaming the compare buffer to the timer compare register via DMA
            dma_timer.start_dma(COMPARE_BUFFER.as_ptr(), BUFFER_LEN);
        }
    }

    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
