#![no_std]
#![no_main]

use rs_uart_bluepill::*;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    // Create UART0 TX in DMA mode
    let uart = lnserial_tx_create(0, true, false);
    
    lnserial_tx_set_speed(uart, 115200);
    lnserial_tx_init(uart);
    
    let msg = b"Hello from pure Rust DMA UART!\r\n";
    lnserial_tx_transmit(uart, msg.len() as u32, msg.as_ptr());
    
    loop {}
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
