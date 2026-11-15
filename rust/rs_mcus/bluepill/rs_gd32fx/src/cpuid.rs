//! GD32F3x / STM32F1 CPU identification and FPU enablement (`drop_c_part3.md` Phase G3).
//!
//! Replaces `esprit/cpp/mcus/arm_gd32fx/src/lnCpuID_arm_gd32fx.cpp` in
//! `LN_ENABLE_RUST` builds while leaving the C++ file untouched for
//! `LN_ENABLE_RUST=OFF`.

use core::ffi::c_char;
use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicI32, AtomicU32, Ordering};

use crate::registers::{
    FpuMvfrRegisters, MVFR0_HW_MUL_DIV, MVFR0_SIMD_MASK, MVFR0_SIMD_SHIFT,
    SCB_CPACR_FPU_FULL_ACCESS, SCB_CPUID_PARTNO_CM4, SCB_CPUID_PARTNO_MASK,
    SCB_CPUID_PARTNO_SHIFT, ScbRegisters,
};

const LN_VENDOR_UNKNOWN: u32 = 0;
const LN_VENDOR_STM: u32 = 1;
const LN_VENDOR_GD: u32 = 2;
const LN_VENDOR_WCH: u32 = 3;

const LN_MCU_UNKNOWN: u32 = 0;
const LN_MCU_ARM_F3: u32 = 1;
const LN_MCU_ARM_F4: u32 = 2;

const MCU_NONE: u32 = 0;
const MCU_STM32_64K: u32 = 1;
const MCU_STM32_128K: u32 = 2;
const MCU_GD32_64K: u32 = 3;
const MCU_GD32_128K: u32 = 4;
const MCU_GD32F3_256K: u32 = 5;
const MCU_CH32F1_64K: u32 = 6;

const DENSITY_RAM_ABSENT: u32 = 0xFFFF;
const STM32_DEFAULT_RAM_KB: i32 = 20;
const DUMMY_SERIAL_ID: u32 = 0x1234_5678;

static MCU: AtomicU32 = AtomicU32::new(LN_MCU_UNKNOWN);
static VENDOR: AtomicU32 = AtomicU32::new(LN_VENDOR_UNKNOWN);
static CPU_ID: AtomicU32 = AtomicU32::new(0);
static FLASH_SIZE_KB: AtomicI32 = AtomicI32::new(0);
static RAM_SIZE_KB: AtomicI32 = AtomicI32::new(0);
static CHIP_ID: AtomicU32 = AtomicU32::new(MCU_NONE);

#[unsafe(no_mangle)]
pub static MVFR0: AtomicU32 = AtomicU32::new(0);
#[unsafe(no_mangle)]
pub static MVFR1: AtomicU32 = AtomicU32::new(0);

/// `void lnCpuID::identify()`
#[unsafe(export_name = "_ZN7lnCpuID8identifyEv")]
pub extern "C" fn identify() {
    if FLASH_SIZE_KB.load(Ordering::Acquire) != 0 {
        return;
    }

    let scb = ScbRegisters::ptr();
    // SAFETY: `ScbRegisters::ptr()` points to the Cortex-M System Control Block.
    let cpuid = unsafe { read_volatile(&(*scb).cpuid) };
    CPU_ID.store(cpuid, Ordering::Relaxed);

    if ((cpuid >> SCB_CPUID_PARTNO_SHIFT) & SCB_CPUID_PARTNO_MASK) == SCB_CPUID_PARTNO_CM4 {
        MCU.store(LN_MCU_ARM_F4, Ordering::Relaxed);
        let fp = FpuMvfrRegisters::ptr();
        // SAFETY: `FpuMvfrRegisters::ptr()` points to Cortex-M4 MVFR0/MVFR1 (`0xE000_EF40`).
        let (mvfr0, mvfr1) = unsafe { (read_volatile(&(*fp).mvfr0), read_volatile(&(*fp).mvfr1)) };
        MVFR0.store(mvfr0, Ordering::Relaxed);
        MVFR1.store(mvfr1, Ordering::Relaxed);
        if ((mvfr0 >> MVFR0_SIMD_SHIFT) & MVFR0_SIMD_MASK) == MVFR0_HW_MUL_DIV {
            // SAFETY: Enables privileged & unprivileged CP10/CP11 access in SCB->CPACR.
            unsafe {
                let c = read_volatile(&(*scb).cpacr);
                write_volatile(&mut (*scb).cpacr, c | SCB_CPACR_FPU_FULL_ACCESS);
            }
        }
    } else {
        MCU.store(LN_MCU_ARM_F3, Ordering::Relaxed);
    }

    let density_ptr = rs_bluepill::MCU_DENSITY_ADDR as *const u32;
    // SAFETY: `MCU_DENSITY_ADDR` (`0x1FFF_F7E0`) is the read-only flash/RAM density register.
    let density = unsafe { read_volatile(density_ptr) };
    let ram_kb = density >> 16;
    let flash_kb = (density & 0xFFFF) as i32;

    if ram_kb != DENSITY_RAM_ABSENT {
        VENDOR.store(LN_VENDOR_GD, Ordering::Relaxed);
        let chip = match flash_kb {
            128 => MCU_GD32_128K,
            64 => MCU_GD32_64K,
            256 => MCU_GD32F3_256K,
            _ => MCU_NONE,
        };
        CHIP_ID.store(chip, Ordering::Relaxed);
        RAM_SIZE_KB.store(ram_kb as i32, Ordering::Relaxed);
    } else if rs_fmc_bluepill::has_fast_unlock() {
        VENDOR.store(LN_VENDOR_WCH, Ordering::Relaxed);
        CHIP_ID.store(MCU_CH32F1_64K, Ordering::Relaxed);
    } else {
        VENDOR.store(LN_VENDOR_STM, Ordering::Relaxed);
        let chip = match flash_kb {
            128 => MCU_STM32_128K,
            64 => MCU_STM32_64K,
            _ => MCU_NONE,
        };
        CHIP_ID.store(chip, Ordering::Relaxed);
        RAM_SIZE_KB.store(STM32_DEFAULT_RAM_KB, Ordering::Relaxed);
    }

    FLASH_SIZE_KB.store(flash_kb, Ordering::Release);
}

/// `lnCpuID::LN_VENDOR lnCpuID::vendor()`
#[unsafe(export_name = "_ZN7lnCpuID6vendorEv")]
pub extern "C" fn vendor() -> u32 {
    VENDOR.load(Ordering::Relaxed)
}

/// `lnCpuID::LN_MCU lnCpuID::mcu()`
#[unsafe(export_name = "_ZN7lnCpuID3mcuEv")]
pub extern "C" fn mcu() -> u32 {
    MCU.load(Ordering::Relaxed)
}

/// `const char *lnCpuID::mcuAsString()`
#[unsafe(export_name = "_ZN7lnCpuID11mcuAsStringEv")]
pub extern "C" fn mcu_as_string() -> *const c_char {
    match MCU.load(Ordering::Relaxed) {
        LN_MCU_ARM_F3 => c"ARMF3".as_ptr(),
        LN_MCU_ARM_F4 => c"ARMF4".as_ptr(),
        _ => c"????".as_ptr(),
    }
}

/// `int lnCpuID::flashSize()`
#[unsafe(export_name = "_ZN7lnCpuID9flashSizeEv")]
pub extern "C" fn flash_size() -> i32 {
    FLASH_SIZE_KB.load(Ordering::Relaxed)
}

/// `int lnCpuID::ramSize()`
#[unsafe(export_name = "_ZN7lnCpuID7ramSizeEv")]
pub extern "C" fn ram_size() -> i32 {
    RAM_SIZE_KB.load(Ordering::Relaxed)
}

/// `const char *lnCpuID::idAsString()`
#[unsafe(export_name = "_ZN7lnCpuID10idAsStringEv")]
pub extern "C" fn id_as_string() -> *const c_char {
    match CHIP_ID.load(Ordering::Relaxed) {
        MCU_STM32_64K => c"STM32F1/64kB".as_ptr(),
        MCU_STM32_128K => c"STM32F1/128kB".as_ptr(),
        MCU_GD32_64K => c"GD32F1/64kB".as_ptr(),
        MCU_GD32_128K => c"GD32F1/128kB".as_ptr(),
        MCU_GD32F3_256K => c"GD32F3/256kB".as_ptr(),
        MCU_CH32F1_64K => c"CH32F1/64kB".as_ptr(),
        _ => c"????".as_ptr(),
    }
}

/// `int lnCpuID::clockSpeed()`
#[unsafe(export_name = "_ZN7lnCpuID10clockSpeedEv")]
pub extern "C" fn clock_speed() -> i32 {
    rs_rcu_bluepill::sysclk_gd32fx::SystemCoreClock.load(Ordering::Relaxed) as i32
}

/// `uint32_t lnCpuID::getSerialID()`
#[unsafe(export_name = "_ZN7lnCpuID11getSerialIDEv")]
pub extern "C" fn get_serial_id() -> u32 {
    DUMMY_SERIAL_ID
}
