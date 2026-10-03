#![no_std]
#![no_main]

use rs_spi_bluepill::*;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    let spi = lnspi_create(0, -1);
    lnspi_begin(spi, 8);
    lnspi_set_speed(spi, 1000000);
    
    // Transfer a byte
    lnspi_transfer8(spi, 1, [0xAA].as_ptr(), core::ptr::null_mut());
    
    lnspi_end(spi);
    loop {}
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
