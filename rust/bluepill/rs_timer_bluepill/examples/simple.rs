#![no_std]
#![no_main]

use core::ffi::c_void;
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

static mut EXPIRED: bool = false;

unsafe extern "C" fn on_delay_timer_irq(_cookie: *mut c_void) {
    unsafe {
        EXPIRED = true;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    // Initialize DelayTimer on hardware timer 1 (TIM2)
    if let Ok(mut dt) = DelayTimer::new(1) {
        dt.set_interrupt(Some(on_delay_timer_irq), core::ptr::null_mut());
        // Arm for 1000 microseconds (1 ms)
        dt.arm(1000);
    }

    loop {}
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
