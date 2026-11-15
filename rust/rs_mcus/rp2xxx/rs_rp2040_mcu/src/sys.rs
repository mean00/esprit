//! RP2040 system clock, startup init, `main`, and shared reset/bootrom helpers.

use core::ffi::c_int;

pub use rs_rp2xxx::sys::{
    deadEnd, ln_rp2040_reboot_to_usb, ln_soft_system_reset, rp2040_reset_to_fw_upload,
};

const LN_IRQ_SVCALL: i32 = -5;
const LN_IRQ_PENDSV: i32 = -2;
const LN_IRQ_SYSTICK: i32 = -1;

#[unsafe(no_mangle)]
pub static mut SystemCoreClock: u32 = 125_000_000;

unsafe extern "C" {
    fn SysTick_Handler();
    fn PendSV_Handler();
    fn SVC_Handler();
    #[link_name = "\u{1}_Z14lnRpDmaSysInitv"]
    fn lnRpDmaSysInit();
    #[link_name = "\u{1}_Z13lnExtiSysInitv"]
    fn lnExtiSysInit();
    #[link_name = "\u{1}_Z21lnSetInterruptHandlerRK5LnIRQPFvvE"]
    fn lnSetInterruptHandler(irq: *const i32, handler: unsafe extern "C" fn());
    fn rp_rust_main() -> !;
}

/// Initialize RP2040 UART0 pins, DMA, EXTI, and FreeRTOS core exception handlers.
pub unsafe fn sys_init() {
    rs_gpio_rp2xxx::shim::ln_pin_mode(17, 11, 0);
    rs_gpio_rp2xxx::shim::ln_pin_mode(16, 11, 0);
    unsafe {
        lnRpDmaSysInit();
        lnExtiSysInit();
        lnSetInterruptHandler(&LN_IRQ_SYSTICK, SysTick_Handler);
        lnSetInterruptHandler(&LN_IRQ_PENDSV, PendSV_Handler);
        lnSetInterruptHandler(&LN_IRQ_SVCALL, SVC_Handler);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> c_int {
    unsafe { rp_rust_main() }
}
