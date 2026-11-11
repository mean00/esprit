//! One-shot microsecond delay through the MCU-agnostic super trait.
//!
//! `rs_mcu_traits::timer::DelayTimer` — `arm_us` starts the countdown and the
//! typed [`DelayHandler`] runs in the timer interrupt. No cookie, no `extern "C"`.

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::sync::atomic::{AtomicBool, Ordering};
use rs_mcu_traits::timer::{self, DelayHandler, DelayTimer as DelayTimerTrait};
use rs_timer_bluepill::DelayTimer;

struct DummyAlloc;

unsafe impl core::alloc::GlobalAlloc for DummyAlloc {
    unsafe fn alloc(&self, _layout: core::alloc::Layout) -> *mut u8 {
        core::ptr::null_mut()
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {}
}

#[global_allocator]
static ALLOCATOR: DummyAlloc = DummyAlloc;

/// Set from the timer interrupt, so it is an atomic.
struct Expiry;

impl DelayHandler for Expiry {
    fn on_elapsed(&self) {
        EXPIRED.store(true, Ordering::SeqCst);
    }
}

static EXPIRY: Expiry = Expiry;
static EXPIRED: AtomicBool = AtomicBool::new(false);

const DELAY_TIMER_INDEX: u32 = 1; // Hardware timer 1 (TIM2)
const DELAY_US: u32 = 1_000; // 1,000 µs (1 ms)

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    // Initialize DelayTimer on hardware timer 1 (TIM2)
    if let Ok(mut timer) = DelayTimer::new(DELAY_TIMER_INDEX) {
        DelayTimerTrait::set_handler(&mut timer, Some(&EXPIRY));
        DelayTimerTrait::enable_interrupt(&mut timer);
        // Arm for 1000 microseconds
        let _ = DelayTimerTrait::arm_us(&mut timer, DELAY_US);
    }

    while !EXPIRED.load(Ordering::SeqCst) {
        core::hint::spin_loop();
    }

    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

