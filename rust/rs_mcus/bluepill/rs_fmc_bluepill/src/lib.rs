//! Flash memory controller driver for the Bluepill family (GD32F303 / CH32V307).
//!
//! The probe's own flash. This is the Rust counterpart of
//! `esprit/mcus/common_bluepill/lnFMC.cpp` (the 90 byte shim that `#include`s the
//! family `lnFMC_impl.cpp`), which is not compiled in a Rust build - see `drop_C.md`,
//! P7. Only the parts a Rust build still needs are here:
//!
//! * [`has_fast_unlock`] - `bool FMC_hasFastUnlock()`: the GD32 unlock probe
//!   `lnCpuID_arm_gd32fx.cpp` uses to tell a real GD32 from a CH32F103 clone (the
//!   clones do not implement the "fast unlock" key).
//! * [`fast_unlock`], [`flash_enhance_mode`], [`init_flash`] - the CH32V3 "enhanced
//!   mode" the flash needs to run at 144 MHz, called once from
//!   `lnRunTime_riscv_ch32v3x.cpp`.
//!
//! The C++ callers are family files with plain C++ declarations, so the four exports
//! carry the Itanium-mangled name (same mechanism as `rs_exti_bluepill/src/irq.rs`).

#![no_std]
#![allow(dead_code)]

use core::ptr::{addr_of_mut, read_volatile, write_volatile};

/// `struct LN_FMCx` of `esprit/private_include/lnFMC_priv.h`.
#[repr(C)]
pub struct FmcRegisters {
    pub ws: u32,
    pub key: u32,
    pub obkey: u32,
    pub stat: u32,
    pub ctl: u32,
    pub addr: u32,
    pub dummy1: u32,
    pub obstat: u32,
    pub wp: u32,
}

impl FmcRegisters {
    #[inline(always)]
    pub fn ptr() -> *mut Self {
        rs_bluepill::FMC_BASE as *mut Self
    }
}

// --- CTL bits (`lnFMC_priv.h` + the CH32 additions of `lnFMC_impl.cpp`) ---------
pub const CTL_START: u32 = 1 << 6;
pub const CTL_LK: u32 = 1 << 7;
pub const CTL_CH32_FASTUNLOCK: u32 = 1 << 15;
pub const CTL_CH32_FASTPROGRAM: u32 = 1 << 16;
pub const CTL_CH32_FASTERASE: u32 = 1 << 17;
pub const CTL_CH32_FASTSTART: u32 = 1 << 21;
pub const CTL_CH32_RSENACT: u32 = 1 << 22;
pub const CTL_CH32_EHMOD: u32 = 1 << 24;

// --- STAT bits -----------------------------------------------------------------
pub const STAT_BUSY: u32 = 1 << 0;
pub const STAT_CH32_WR_BUSY: u32 = 1 << 1;
pub const STAT_CH32_EHMODS: u32 = 1 << 7;

// --- Unlock keys ---------------------------------------------------------------
/// The two words the controller expects, in this order, in `KEY`.
pub const KEY1: u32 = 0x4567_0123;
pub const KEY2: u32 = 0xCDEF_89AB;

/// `CHF103[9]` in the C++: the "fast unlock" key register of the WCH parts.
const FAST_KEY_WORD_INDEX: usize = 9;

/// Nops the C++ puts between the two key sequences (`for (int i = 0; i < 50; i++)`).
const KEY_DELAY_NOPS: u32 = 50;

#[inline(always)]
fn key_delay() {
    for _ in 0..KEY_DELAY_NOPS {
        unsafe { core::arch::asm!("nop") };
    }
}

/// Write the plain unlock pair, then the fast-unlock pair, exactly as the C++ does
/// (`CH32_fastUnlock`, and the second half of `FMC_hasFastUnlock`).
#[inline(always)]
unsafe fn write_unlock_keys() {
    let fmc = FmcRegisters::ptr();
    unsafe {
        write_volatile(addr_of_mut!((*fmc).key), KEY1);
        write_volatile(addr_of_mut!((*fmc).key), KEY2);
        key_delay();
        let words = FmcRegisters::ptr() as *mut u32;
        write_volatile(words.add(FAST_KEY_WORD_INDEX), KEY1);
        write_volatile(words.add(FAST_KEY_WORD_INDEX), KEY2);
        key_delay();
    }
}

/// `bool FMC_hasFastUnlock()` - GD32 (ARM) side of `lnFMC_impl.cpp`.
///
/// Reset the fast-unlock bit first (it reads back as 1 once cleared), then send both
/// key pairs: the bit clears again when the part implements fast unlock.
pub fn has_fast_unlock() -> bool {
    unsafe {
        let fmc = FmcRegisters::ptr();
        let ctl = addr_of_mut!((*fmc).ctl);
        write_volatile(ctl, read_volatile(ctl) | CTL_CH32_FASTUNLOCK);
        if (read_volatile(ctl) & CTL_CH32_FASTUNLOCK) == 0 {
            return false;
        }
        write_unlock_keys();
        (read_volatile(ctl) & CTL_CH32_FASTUNLOCK) == 0
    }
}

/// `bool CH32_fastUnlock()` - CH32V3 side of `lnFMC_impl.cpp`: the key pairs, without
/// the leading reset of [`has_fast_unlock`].
pub fn fast_unlock() -> bool {
    unsafe {
        write_unlock_keys();
        let fmc = FmcRegisters::ptr();
        (read_volatile(addr_of_mut!((*fmc).ctl)) & CTL_CH32_FASTUNLOCK) == 0
    }
}

/// `bool CH32_fastLock()`: lock the controller again.
pub fn fast_lock() -> bool {
    unsafe {
        let fmc = FmcRegisters::ptr();
        let ctl = addr_of_mut!((*fmc).ctl);
        write_volatile(ctl, read_volatile(ctl) | CTL_LK);
        true
    }
}

/// `void CH32V3_flashEnhanceMode(bool)` - the flash's "enhanced mode", which the
/// CH32V307 needs at 144 MHz. Turning it off activates the "reset enable" bit; turning
/// it on reads `STAT` once so the mode change has landed.
pub fn flash_enhance_mode(enable: bool) {
    unsafe {
        let fmc = FmcRegisters::ptr();
        let ctl = addr_of_mut!((*fmc).ctl);
        if enable {
            fast_unlock();
            write_volatile(ctl, read_volatile(ctl) | CTL_CH32_EHMOD);
            let _ = read_volatile(addr_of_mut!((*fmc).stat));
        } else {
            fast_unlock();
            write_volatile(ctl, read_volatile(ctl) & !CTL_CH32_EHMOD);
            write_volatile(ctl, read_volatile(ctl) | CTL_CH32_RSENACT);
        }
    }
}

/// `void lnInitFlash_ch32v3x()` - called once by `lnRunTime_riscv_ch32v3x.cpp`.
pub fn init_flash() {
    flash_enhance_mode(true);
}

// --- The entries the family C++ still calls -------------------------------------

/// `bool FMC_hasFastUnlock()`
#[unsafe(export_name = "_Z17FMC_hasFastUnlockv")]
pub extern "C" fn fmc_has_fast_unlock() -> bool {
    has_fast_unlock()
}

/// `void lnInitFlash_ch32v3x()`
#[unsafe(export_name = "_Z19lnInitFlash_ch32v3xv")]
pub extern "C" fn ln_init_flash_ch32v3x() {
    init_flash();
}

/// `bool CH32_fastUnlock()`
#[unsafe(export_name = "_Z15CH32_fastUnlockv")]
pub extern "C" fn ch32_fast_unlock() -> bool {
    fast_unlock()
}

/// `void CH32V3_flashEnhanceMode(bool)`
#[unsafe(export_name = "_Z23CH32V3_flashEnhanceModeb")]
pub extern "C" fn ch32v3_flash_enhance_mode(enable: bool) {
    flash_enhance_mode(enable);
}
