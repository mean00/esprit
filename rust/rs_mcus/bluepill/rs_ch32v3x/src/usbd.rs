//! CH32V3x USBFS platform glue (`dcd_usbfs_platform.cpp` + `lnUsbStubs.cpp`).

use rs_rcu_bluepill::Peripheral;

const LN_IRQ_OTG_FS: u32 = 65;

unsafe extern "C" {
    fn dcd_int_handler(port: u8);
    fn lnDelay_C(ms: u32);
}

#[unsafe(no_mangle)]
pub extern "C" fn dcd_int_enable(_rhport: u8) {
    crate::irq::ln_enable_interrupt(&LN_IRQ_OTG_FS);
}

#[unsafe(no_mangle)]
pub extern "C" fn dcd_int_disable(_rhport: u8) {
    crate::irq::ln_disable_interrupt(&LN_IRQ_OTG_FS);
}

#[unsafe(no_mangle)]
pub extern "C" fn OTG_FS_IRQHandler() {
    // SAFETY: `dcd_int_handler(0)` is TinyUSB's CH32 USBFS DCD ISR.
    unsafe { dcd_int_handler(0) };
}

#[unsafe(no_mangle)]
pub extern "C" fn dcd_fs_platform_init() {
    rs_rcu_bluepill::usb_ch32v3x::enable_usb_48mhz();
    rs_rcu_bluepill::enable(Peripheral::UsbFsOtgCh32v3x);
}

#[unsafe(no_mangle)]
pub extern "C" fn dcd_fs_platform_delay(ms: i32) {
    if ms > 0 {
        // SAFETY: `lnDelay_C` is provided by `rust_esprit::runtime`.
        unsafe { lnDelay_C(ms as u32) };
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lnUsbDcdInit() {}
