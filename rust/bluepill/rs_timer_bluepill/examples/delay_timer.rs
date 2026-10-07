//! Hardware microsecond DelayTimer example.
//!
//! Follows Esprit Rust Rule 1 (No Magic Numbers), Rule 2 (Two-Tier Architecture),
//! Rule 4 (Mandatory Examples), and Rule 6 (Idiomatic Rust & C++ Parity).
//!
//! Demonstrates configuring a one-shot microsecond alarm with an interrupt callback.

#![no_std]
#![no_main]

use core::ffi::c_void;
use core::panic::PanicInfo;
use core::sync::atomic::{AtomicBool, Ordering};
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

static EXPIRED: AtomicBool = AtomicBool::new(false);

const DELAY_TIMER_INDEX: u32 = 1; // Hardware timer 1 (TIM2)
const DELAY_US: u32 = 1_000; // 1,000 µs (1 ms)

unsafe extern "C" fn on_delay_timer_irq(_cookie: *mut c_void) {
    EXPIRED.store(true, Ordering::SeqCst);
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    // Initialize DelayTimer on hardware timer 1 (TIM2)
    if let Ok(mut dt) = DelayTimer::new(DELAY_TIMER_INDEX) {
        dt.set_interrupt(Some(on_delay_timer_irq), core::ptr::null_mut());
        // Arm for 1000 microseconds
        dt.arm(DELAY_US);
    }

    loop {
        if EXPIRED.load(Ordering::SeqCst) {
            // Delay expired
            break;
        }
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
