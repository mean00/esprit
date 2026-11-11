#![no_std]
#![no_main]

use core::panic::PanicInfo;
use rs_pinout48_bluepill::{Pin, adc_channel, timer_channel};

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

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    // Which ADC channel / timer channel is PA0 wired to ?
    let _adc = adc_channel(Pin::PA0);
    let _timer = timer_channel(Pin::PA8);
    loop {}
}
