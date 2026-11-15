//! Shared Cortex-M reset, `deadEnd`, and USB-bootrom jump for RP2040 / RP2350.

use core::ffi::c_int;

const SCB_AIRCR: *mut u32 = 0xE000_ED0C as *mut u32;
const AIRCR_SYSRESETREQ: u32 = (1 << 2) | (0x5fa << 16);

#[unsafe(export_name = "_Z17lnSoftSystemResetv")]
pub extern "C" fn ln_soft_system_reset() -> ! {
    // SAFETY: Writing `AIRCR_SYSRESETREQ` triggers a Cortex-M system reset.
    unsafe {
        core::ptr::write_volatile(SCB_AIRCR, AIRCR_SYSRESETREQ);
        loop {
            core::arch::asm!("nop", options(nomem, nostack, preserves_flags));
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn deadEnd(_code: c_int) -> ! {
    // SAFETY: `bkpt #0` halts if a debugger is attached before resetting.
    unsafe { core::arch::asm!("bkpt #0", options(nomem, nostack)) };
    ln_soft_system_reset()
}

#[unsafe(export_name = "_Z20lnRp2040_reboot_to_usbv")]
pub extern "C" fn ln_rp2040_reboot_to_usb() {
    // SAFETY: Hardcoded bootrom USB-boot entry point (0x2591).
    let reset: extern "C" fn(u32, u32) = unsafe { core::mem::transmute(0x2591usize) };
    reset(0, 0);
}

#[unsafe(export_name = "_Z21Rp2040ResetToFwUploadv")]
pub extern "C" fn rp2040_reset_to_fw_upload() {
    ln_rp2040_reboot_to_usb();
}
