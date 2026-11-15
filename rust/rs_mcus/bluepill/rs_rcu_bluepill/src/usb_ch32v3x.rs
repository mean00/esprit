//! CH32V3x USB clock and EXTEN_CTR helpers (`drop_c_part2.md` Phase C1).
//!
//! Replaces `esprit/cpp/mcus/riscv_ch32v3x/src/lnRCU_ch32v3x_usb.cpp` in
//! `LN_ENABLE_RUST` builds while leaving the C++ file untouched for
//! `LN_ENABLE_RUST=OFF`.

use core::sync::atomic::{AtomicBool, Ordering};

use crate::registers::{
    LN_RCU_CFG0_USBPSC_BITS, LN_RCU_CFG0_USBPSC_MASK, LN_RCU_CFG0_USBPSC_SHIFT, RcuRegisters,
};
use crate::registers_chip::{
    CH32V3_USBHS_CFG1_KEEP_MASK, CH32V3_USBHS_CKREF_SEL_4MHZ, CH32V3_USBHS_CKREF_SRC_USB_PHY,
    CH32V3_USBHS_PLL_SRC_HSE, CH32V3_USBHS_PREFIX_DIV2, CH32V3_USBHS_USB_PHY_ENABLED,
    CH32_EXTEN_CTR_ETH_10M_ENABLE, CH32_EXTEN_CTR_USBD_PULLUP_ENABLE,
};
use crate::{get_clock, Peripheral};

const USB_48MHZ_HZ: u32 = 48_000_000;

static USB48M_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// `void lnCh32_enableUsbPullUp()`
#[unsafe(export_name = "_Z22lnCh32_enableUsbPullUpv")]
pub extern "C" fn ln_ch32_enable_usb_pull_up() {
    let exten = rs_bluepill::EXTEN_CTR_BASE as *mut u32;
    // SAFETY: `EXTEN_CTR_BASE` (`0x4002_3800`) is the CH32V3x extended control register.
    unsafe {
        let v = core::ptr::read_volatile(exten);
        core::ptr::write_volatile(exten, v | CH32_EXTEN_CTR_USBD_PULLUP_ENABLE);
    }
}

/// `void lnCh32_enableInternalEth()`
#[unsafe(export_name = "_Z24lnCh32_enableInternalEthv")]
pub extern "C" fn ln_ch32_enable_internal_eth() {
    let exten = rs_bluepill::EXTEN_CTR_BASE as *mut u32;
    // SAFETY: `EXTEN_CTR_BASE` (`0x4002_3800`) is the CH32V3x extended control register.
    unsafe {
        let v = core::ptr::read_volatile(exten);
        core::ptr::write_volatile(exten, v | CH32_EXTEN_CTR_ETH_10M_ENABLE);
    }
}

/// `void lnPeripherals::enableUsbHS48Mhz_ch32v3x()`
#[unsafe(export_name = "_ZN13lnPeripherals24enableUsbHS48Mhz_ch32v3xEv")]
pub extern "C" fn enable_usb_hs_48mhz_ch32v3x() {
    // SAFETY: `RcuRegisters::ptr()` points to the memory-mapped RCU block.
    unsafe {
        modify_reg!(cfg1, |v| {
            (v & CH32V3_USBHS_CFG1_KEEP_MASK)
                | CH32V3_USBHS_PLL_SRC_HSE
                | CH32V3_USBHS_PREFIX_DIV2
                | CH32V3_USBHS_CKREF_SEL_4MHZ
                | CH32V3_USBHS_CKREF_SRC_USB_PHY
                | CH32V3_USBHS_USB_PHY_ENABLED
        });
    }
}

/// `void lnPeripherals::enableUsb48Mhz()`
#[unsafe(export_name = "_ZN13lnPeripherals14enableUsb48MhzEv")]
pub extern "C" fn enable_usb_48mhz() {
    if USB48M_INITIALIZED.swap(true, Ordering::AcqRel) {
        return;
    }

    let scaler = (2 * get_clock(Peripheral::SysClock)) / USB_48MHZ_HZ;
    let x: u32 = match scaler {
        2 => 0, // 48 MHz
        4 => 1, // 96 MHz
        6 => 2, // 144 MHz
        _ => return,
    };

    ln_ch32_enable_usb_pull_up();
    // SAFETY: `RcuRegisters::ptr()` points to the memory-mapped RCU block.
    unsafe {
        modify_reg!(cfg0, |v| {
            (v & LN_RCU_CFG0_USBPSC_MASK)
                | ((x & LN_RCU_CFG0_USBPSC_BITS) << LN_RCU_CFG0_USBPSC_SHIFT)
        });
    }
}
