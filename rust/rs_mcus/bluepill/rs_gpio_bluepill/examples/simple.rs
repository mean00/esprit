#![no_std]
#![no_main]

use rs_gpio_bluepill::{Pin, Mode, set_mode, write, toggle};
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    // Configure PA0 as push-pull output
    set_mode(Pin::PA0, Mode::Output, 50);
    
    // Write high
    write(Pin::PA0, true);
    
    loop {
        // Toggle pin
        toggle(Pin::PA0);
        
        // Dummy delay
        for _ in 0..100000 {
            unsafe { core::arch::asm!("nop") };
        }
    }
}
