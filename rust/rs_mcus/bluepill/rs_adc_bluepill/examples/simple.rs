#![no_std]
#![no_main]

use rs_adc_bluepill::*;
use rs_gpio_bluepill::lnPin;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    let adc = ln_timing_adc_create(0);
    
    let pins = [lnPin::PA0];
    ln_timing_adc_set_source(adc, 0, 0, 1000, 1, pins.as_ptr());
    
    let mut buffer: [u16; 64] = [0; 64];
    
    ln_timing_adc_multi_read(adc, 64, buffer.as_mut_ptr());
    
    loop {}
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
