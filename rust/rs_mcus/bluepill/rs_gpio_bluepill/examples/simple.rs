//! GPIO through the MCU-agnostic super trait.
//!
//! `rs_mcu_traits::gpio::Gpio` is implemented by `rs_gpio_bluepill::Pin` for the
//! GD32F3 / CH32V3 family, so this file never names a register. The calls go
//! through `GpioTrait::…` because the pin type also has inherent methods (the
//! family's own, with a different arity for `set_mode`).
//!
//! The pin **interrupt** is not part of this trait: it belongs to
//! `rs_mcu_traits::exti::Exti` (see the `rs_exti_bluepill` example).

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use rs_gpio_bluepill::Pin;
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

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    let mut led = Pin::PA0;

    // Trait methods: every one answers a `Result` where the hardware can refuse.
    let _ = GpioTrait::set_mode(&mut led, PinMode::Output);

    loop {
        GpioTrait::write(&mut led, true);
        for _ in 0..100_000 {
            core::hint::spin_loop();
        }
        GpioTrait::toggle(&mut led);
        for _ in 0..100_000 {
            core::hint::spin_loop();
        }

        // Reading back the pad level, which is the whole input side of the trait.
        let _high = GpioTrait::read(&led);
    }
}



