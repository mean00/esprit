#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::sync::atomic::{AtomicU32, Ordering};
use rs_esprit::timer::{Timer, TimerHandler};

/// Application handler responding to periodic timer interrupts using safe interior mutability.
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

/// Demonstrates configuring and starting a periodic timer using `rs_esprit::Timer`.
pub fn run_timer_demo<T: Timer>(mut timer: T) {
    // Configure timer to tick at 1 kHz (1 ms period)
    let _ = timer.set_frequency(1_000);

    // Completely safe registration - zero unsafe!
    timer.set_handler(Some(&TIMER_HANDLER));

    timer.enable();
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
