#![no_std]
#![no_main]

use core::panic::PanicInfo;
use rs_pinout48_bluepill::{Pin, adc_channel, timer_channel};

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
