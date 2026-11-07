//! High-level trait API periodic timer example.
//!
//! Follows Esprit Rust Rule 1 (No Magic Numbers), Rule 2 (Two-Tier Architecture),
//! Rule 4 (Mandatory Examples), and Rule 6 (Idiomatic Rust).
//!
//! Demonstrates configuring a hardware timer using the high-level `rs_esprit::Timer` trait.

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::sync::atomic::{AtomicU32, Ordering};
use rs_esprit::timer::{Timer as _, TimerHandler};
use rs_timer_bluepill::Timer;

struct DummyAlloc;
unsafe impl core::alloc::GlobalAlloc for DummyAlloc {
    unsafe fn alloc(&self, _layout: core::alloc::Layout) -> *mut u8 {
        core::ptr::null_mut()
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {}
}

#[global_allocator]
static ALLOCATOR: DummyAlloc = DummyAlloc;

/// Application handler responding to periodic timer interrupts.
struct AppTimerHandler {
    tick_count: AtomicU32,
}

impl TimerHandler for AppTimerHandler {
    fn on_timer_tick(&self) {
        self.tick_count.fetch_add(1, Ordering::Relaxed);
    }
}

static TIMER_HANDLER: AppTimerHandler = AppTimerHandler {
    tick_count: AtomicU32::new(0),
};

const TIMER_INDEX: u32 = 1; // Timer 1 (TIM2)
const TICK_FREQUENCY_HZ: u32 = 1_000; // 1 kHz (1 ms period)

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    if let Ok(mut timer) = Timer::new(TIMER_INDEX) {
        // Use the high-level `rs_esprit::Timer` trait API:
        let _ = timer.set_frequency(TICK_FREQUENCY_HZ);
        timer.set_handler(Some(&TIMER_HANDLER));
        timer.enable();
    }

    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
