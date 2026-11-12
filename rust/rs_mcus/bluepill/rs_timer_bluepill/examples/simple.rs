//! Periodic timer through the MCU-agnostic super trait.
//!
//! `rs_mcu_traits::timer::Timer` — `start_periodic(hz)` returns the frequency the
//! hardware actually reaches, the handler is per channel and `on_tick` runs in the
//! timer interrupt.

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::sync::atomic::{AtomicU32, Ordering};
use rs_mcu_traits::timer::{self, Timer as TimerTrait, TimerHandler};
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
    fn on_tick(&self) {
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
        // `start_periodic` returns the rounded frequency the counter reaches.
        let _reached_hz = TimerTrait::start_periodic(&mut timer, TICK_FREQUENCY_HZ);
        TimerTrait::set_handler(&mut timer, Some(&TIMER_HANDLER));
    }

    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

