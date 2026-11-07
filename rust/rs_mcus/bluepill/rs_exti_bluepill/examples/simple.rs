#![no_std]
#![no_main]

use rs_exti_bluepill::{Edge, attach_interrupt, enable_interrupt};
use rs_gpio_bluepill::Pin;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

unsafe extern "C" fn my_callback(_pin: Pin, _cookie: *mut core::ffi::c_void) {
    // Handle interrupt
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    // Attach an interrupt to PA0 on rising edge
    attach_interrupt(Pin::PA0, Edge::Rising, Some(my_callback), core::ptr::null_mut());
    
    // Enable the interrupt
    enable_interrupt(Pin::PA0);
    
    loop {}
}
