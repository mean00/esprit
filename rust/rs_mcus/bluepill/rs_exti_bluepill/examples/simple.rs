//! External interrupts through the MCU-agnostic super trait.
//!
//! `rs_mcu_traits::exti::Exti` addresses the pin by number and takes a typed
//! handler, so the family's C-style callback never leaks into application code.

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use rs_exti_bluepill::Exti;
use rs_gpio_bluepill::Pin;
use rs_mcu_traits::exti::{self, Edge, ExtiHandler};
use rs_mcu_traits::gpio::{Gpio as GpioTrait, PinMode};

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

/// Called from the EXTI interrupt, so it only touches atomics.
struct ButtonHandler;

impl ExtiHandler for ButtonHandler {
    fn on_exti(&self, _pin: u32) {}
}

static BUTTON: ButtonHandler = ButtonHandler;

const BUTTON_PIN: u32 = Pin::PA0 as u32;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    // A pin interrupt needs the pad set up first (`Gpio`), then the edge armed on
    // the interrupt controller (`Exti`) — two contracts, two calls.
    let mut button = Pin::PA0;
    let _ = GpioTrait::set_mode(&mut button, PinMode::InputPullUp);

    let mut exti = Exti;

    // Attach the handler and the edge in one call: the line is enabled by it.
    if exti::Exti::attach(&mut exti, BUTTON_PIN, Edge::Falling, Some(&BUTTON)).is_ok() {
        let _ = exti::Exti::disable(&mut exti, BUTTON_PIN);
        let _ = exti::Exti::enable(&mut exti, BUTTON_PIN);
    }

    loop {
        core::hint::spin_loop();
    }
}
