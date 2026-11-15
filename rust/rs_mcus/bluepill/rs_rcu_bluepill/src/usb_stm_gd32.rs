//! STM32F1 / GD32F3x USB 48 MHz clock setup (`drop_c_part3.md` Phase G1).
//!
//! Replaces `esprit/cpp/mcus/arm_gd32fx/src/lnRCU_armgd32.cpp` in
//! `LN_ENABLE_RUST` builds while leaving the C++ file untouched for
//! `LN_ENABLE_RUST=OFF`.

use core::sync::atomic::{AtomicBool, Ordering};

use crate::registers::{
    LN_GD_RCU_ADDCTL_IRC48M_EN, LN_GD_RCU_ADDCTL_IRC48M_SEL, LN_GD_RCU_ADDCTL_IRC48M_STB,
    LN_RCU_CFG0_USBPSC_BITS, LN_RCU_CFG0_USBPSC_MASK, LN_RCU_CFG0_USBPSC_SHIFT, RcuRegisters,
};
use crate::{get_clock, Peripheral};

const LN_VENDOR_STM: u32 = 1;
const LN_VENDOR_GD: u32 = 2;
const USB_48MHZ_HZ: u32 = 48_000_000;

static USB48M_INITIALIZED: AtomicBool = AtomicBool::new(false);

unsafe extern "C" {
    #[link_name = "\u{1}_ZN7lnCpuID6vendorEv"]
    fn ln_cpu_id_vendor() -> u32;
}

/// `void lnPeripherals::enableUsb48Mhz()`
#[unsafe(export_name = "_ZN13lnPeripherals14enableUsb48MhzEv")]
pub extern "C" fn enable_usb_48mhz() {
    if USB48M_INITIALIZED.swap(true, Ordering::AcqRel) {
        return;
    }

    // SAFETY: `lnCpuID::vendor()` reads a static initialized during boot.
    let vendor = unsafe { ln_cpu_id_vendor() };

    if option_env!("LN_USE_INTERNAL_CLOCK").is_some() && vendor == LN_VENDOR_GD {
        // SAFETY: `RcuRegisters::ptr()` points to the memory-mapped RCU block;
        // single-core / boot-time init of ADDCTL for GD32 IRC48M.
        unsafe {
            modify_reg!(addctl, |v| v | LN_GD_RCU_ADDCTL_IRC48M_EN);
            while (read_reg!(addctl) & LN_GD_RCU_ADDCTL_IRC48M_STB) == 0 {}
            modify_reg!(addctl, |v| v | LN_GD_RCU_ADDCTL_IRC48M_SEL);
        }
        return;
    }

    let scaler = (2 * get_clock(Peripheral::SysClock)) / USB_48MHZ_HZ;
    let psc: u32 = match (vendor, scaler) {
        (LN_VENDOR_STM, 3) => 0, // 3/2 = 1.5
        (LN_VENDOR_STM, 2) => 1, // 2/2 = 1
        (LN_VENDOR_GD, 3) => 0,  // 3/2 = 1.5
        (LN_VENDOR_GD, 2) => 1,  // 2/2 = 1
        (LN_VENDOR_GD, 5) => 2,  // 5/2 = 2.5
        (LN_VENDOR_GD, 4) => 3,  // 4/2 = 2
        _ => panic!("invalid sys clock or vendor for USB 48 MHz"),
    };

    // SAFETY: `RcuRegisters::ptr()` points to the memory-mapped RCU block;
    // updates CFG0 USBPSC bits before USB peripheral is enabled.
    unsafe {
        modify_reg!(cfg0, |v| {
            (v & LN_RCU_CFG0_USBPSC_MASK)
                | ((psc & LN_RCU_CFG0_USBPSC_BITS) << LN_RCU_CFG0_USBPSC_SHIFT)
        });
    }
}
