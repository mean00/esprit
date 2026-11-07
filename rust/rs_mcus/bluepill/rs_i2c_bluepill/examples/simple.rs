#![no_std]
#![no_main]

use rs_i2c_bluepill::*;
use core::ptr;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    let i2c = lni2c_create(0, 100000);
    
    let data = [0x12, 0x34];
    let data_ptr = data.as_ptr();
    let len = 2;
    
    lni2c_multi_write_to(
        i2c,
        0x50, // EEPROM address
        1,
        &len,
        &data_ptr as *const _ as *mut *const u8,
    );
    
    loop {}
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
