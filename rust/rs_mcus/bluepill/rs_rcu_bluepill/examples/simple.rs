//! Reset and clock control through the MCU-agnostic super trait.
//!
//! `rs_mcu_traits::rcu::Rcu` takes the *family's own* `Peripheral` enum
//! ([`rs_rcu_bluepill::Peripheral`]) as an associated type, so the contract never
//! names a chip and the backend keeps its full gate list.

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use rs_mcu_traits::rcu::Rcu as RcuTrait;
use rs_rcu_bluepill::{Peripheral, Rcu};

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
    // Gate a peripheral on, ask what clock it runs from, then put it back in
    // reset (which is what a driver does before programming it). `Rcu` is the
    // family's zero-sized implementor, so the trait's associated functions are
    // reached through it.
    Rcu::enable(Peripheral::GpioA);
    let _hz = <Rcu as RcuTrait>::clock_hz(Peripheral::GpioA);
    Rcu::reset(Peripheral::GpioA);
    Rcu::disable(Peripheral::GpioA);

    // The USART's clock, which is what a baud divisor is computed from.
    let _uart_hz = rs_rcu_bluepill::get_clock(Peripheral::Uart1);

    loop {
        core::hint::spin_loop();
    }
}


