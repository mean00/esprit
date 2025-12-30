#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clashing_extern_declarations)]
#![allow(unsafe_op_in_unsafe_fn)]
//#![feature(lang_items)]
extern crate alloc;

pub type size_t = cty::c_uint;

// C api -> bindgen
// done manually mod rn_exti_c;
mod rn_fast_event_c;
#[cfg(not(any(feature = "rp2040", feature = "esp32c3")))]
pub mod rn_fast_gpio_bp;
#[cfg(feature = "esp32c3")]
pub mod rn_fast_gpio_esp32c3;
#[cfg(feature = "rp2040")]
pub mod rn_fast_gpio_rp2040;
#[cfg(not(any(feature = "rp2040", feature = "esp32c3")))]
pub mod rn_gpio_bp_c;
#[cfg(feature = "esp32c3")]
pub mod rn_gpio_esp32_c;
#[cfg(feature = "rp2040")]
pub mod rn_gpio_rp2040_c;
mod rn_i2c_c;
mod rn_spi_c;
mod rn_timer_c;
pub mod rust_esprit;
// internal API
pub mod rn_exti;
pub mod rn_fast_event_group;
pub mod rn_freertos_c;
pub mod rn_gpio;
pub mod rn_i2c;
pub mod rn_logger;
pub mod rn_os_helper;
pub mod rn_spi;
pub mod rn_timing_adc;
pub mod rn_timing_adc_c;
//mod rn_timer_c;
//pub use rn_timer_c::lnGetUs;
//pub use rn_timer_c::lnGetMs;
//pub use rn_timer_c::lnDelay;
//pub use rn_timer_c::lnDelayUs;
//
//pub use crate::rn_gpio::rnPin as  rnPin;
// embedded hal
//pub mod rn_hal_gpio;

#[cfg(feature = "cdc")]
pub mod rn_cdc_c;
#[cfg(feature = "cdc")]
pub mod rn_usb;
#[cfg(feature = "cdc")]
pub mod rn_usb_c;
#[cfg(feature = "cdc")]
pub mod rn_usb_cdc;

/*
 * Re-export functions
 */
pub use gpio::rnGpioMode as GpioMode;
pub use rn_fast_event_group::rnFastEventGroup as FastEventGroup;
pub use rn_gpio as gpio;
pub use rn_gpio::digital_read;
pub use rn_gpio::digital_toggle;
pub use rn_gpio::digital_write;
pub use rn_gpio::lnPin as pin;
pub use rn_gpio::pin_mode;
pub use rn_os_helper::delay_ms;
/*
pub struct FreeRtosAllocator;

unsafe impl GlobalAlloc for FreeRtosAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let res = pvPortMalloc(layout.size() as cty::c_uint);
        res as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        vPortFree(ptr as *mut cty::c_void);
    }
}
#[global_allocator] // borrowed from https://github.com/lobaro/FreeRTOS-rust
static GLOBAL: FreeRtosAllocator = FreeRtosAllocator;
unsafe extern "C" {
    pub fn pvPortMalloc(xSize: size_t) -> *mut cty::c_void;
}
unsafe extern "C" {
    pub fn vPortFree(pv: *mut cty::c_void);
}

unsafe extern "C" {
    pub fn deadEnd(code: cty::c_int);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        deadEnd(55); //: cty::c_int);
    }
    loop {}
}
*/
