//! UART through the MCU-agnostic super trait.
//!
//! `rs_mcu_traits::uart::Uart` sees one port — configuration, blocking writes and
//! a draining read — and the completion handler is per instance (UART0 and UART1
//! never share one).

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use rs_mcu_traits::uart::{self, Uart as UartTrait, UartConfig, UartHandler};
use rs_uart_bluepill::Uart;

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

/// Received bytes arrive from the receiver's interrupt, so this only records
/// counts.
struct Bridge;

impl UartHandler for Bridge {
    fn on_rx(&self, _byte: u8) {}
    fn on_tx_complete(&self) {}
}

static BRIDGE: Bridge = Bridge;

const BAUD_RATE: u32 = 115_200;
const RX_BUFFER: usize = 64;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    // 8N1 at the requested rate; `Err(Unsupported)` would mean the frame is not
    // reachable on this USART.
    let config = UartConfig {
        baudrate: BAUD_RATE,
        ..UartConfig::default()
    };
    let Ok(mut uart) = Uart::new(0, &config) else {
        loop {}
    };

    // The handler is per instance: `Uart::new(1, …)` would get its own slot.
    UartTrait::set_handler(&mut uart, Some(&BRIDGE));

    let greeting = b"Hello from the esprit super trait!\r\n";
    let _ = UartTrait::write(&mut uart, greeting);

    let mut buffer = [0u8; RX_BUFFER];
    loop {
        // Non-blocking drain of what the ring holds; `read_until_idle` would block
        // until the buffer is full.
        if let Ok(read) = UartTrait::read(&mut uart, &mut buffer) {
            let _ = read;
        }
        core::hint::spin_loop();
    }
}
