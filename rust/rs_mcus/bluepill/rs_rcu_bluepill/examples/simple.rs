#![no_std]
#![no_main]

use rs_rcu_bluepill::{Peripheral, enable, disable};
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    // Enable GPIOA clock
    enable(Peripheral::GpioA);
    
    // Disable GPIOA clock
    disable(Peripheral::GpioA);
    
    loop {}
}
