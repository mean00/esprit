//! RP2350 system clock, startup init, and shared reset/bootrom helpers.

pub use rs_rp2xxx::sys::{
    deadEnd, ln_rp2040_reboot_to_usb, ln_soft_system_reset, rp2040_reset_to_fw_upload,
};

const LN_IRQ_SVCALL: i32 = -5;
const LN_IRQ_PENDSV: i32 = -2;
const LN_IRQ_SYSTICK: i32 = -1;

#[unsafe(no_mangle)]
pub static mut SystemCoreClock: u32 = 150_000_000;

unsafe extern "C" {
    fn isr_systick();
    fn isr_pendsv();
    fn isr_svcall();
    #[link_name = "\u{1}_Z14lnRpDmaSysInitv"]
    fn lnRpDmaSysInit();
    #[link_name = "\u{1}_Z21lnSetInterruptHandlerRK5LnIRQPFvvE"]
    fn lnSetInterruptHandler(irq: *const i32, handler: unsafe extern "C" fn());
}

/// Initialize RP2350 UART0 pin, DMA, and FreeRTOS core exception handlers.
pub unsafe fn sys_init() {
    rs_gpio_rp2xxx::shim::ln_pin_mode(17, 11, 0);
    unsafe {
        lnRpDmaSysInit();
        lnSetInterruptHandler(&LN_IRQ_SYSTICK, isr_systick);
        lnSetInterruptHandler(&LN_IRQ_PENDSV, isr_pendsv);
        lnSetInterruptHandler(&LN_IRQ_SVCALL, isr_svcall);
    }
}
