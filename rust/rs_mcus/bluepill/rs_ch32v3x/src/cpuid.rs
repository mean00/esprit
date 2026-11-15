//! CH32V3x CPU identification (`drop_c_part2.md` Phase C3).
//!
//! Replaces `esprit/cpp/mcus/riscv_ch32v3x/src/lnCpuID_riscv_ch32v3x.cpp` in
//! `LN_ENABLE_RUST` builds.

use core::ffi::c_char;
use core::sync::atomic::{AtomicU32, Ordering};

use rs_rcu_bluepill::sysclk_ch32v3x::SystemCoreClock;

// lnCpuID::LN_VENDOR values (esprit/cpp/include/lnCpuID.h)
const LN_VENDOR_WCH: u32 = 3;
// lnCpuID::LN_MCU values
const LN_MCU_RISCV: u32 = 3;

static MCU: AtomicU32 = AtomicU32::new(0);
static VENDOR: AtomicU32 = AtomicU32::new(0);
static FLASH_SIZE: AtomicU32 = AtomicU32::new(0);
static RAM_SIZE: AtomicU32 = AtomicU32::new(0);
static CHIP_ID: AtomicU32 = AtomicU32::new(0);

const MCU_CH32V307_128K: u32 = 1;
const CH32V307_RAM_KB: u32 = 32;
const CH32V307_FLASH_KB: u32 = 64;
const DUMMY_SERIAL_ID: u32 = 0x1234_5678;

#[unsafe(export_name = "_ZN7lnCpuID8identifyEv")]
pub extern "C" fn identify() {
    if FLASH_SIZE.load(Ordering::Relaxed) != 0 {
        return;
    }
    MCU.store(LN_MCU_RISCV, Ordering::Relaxed);
    CHIP_ID.store(MCU_CH32V307_128K, Ordering::Relaxed);
    RAM_SIZE.store(CH32V307_RAM_KB, Ordering::Relaxed);
    FLASH_SIZE.store(CH32V307_FLASH_KB, Ordering::Relaxed);
    VENDOR.store(LN_VENDOR_WCH, Ordering::Relaxed);
}

#[unsafe(export_name = "_ZN7lnCpuID6vendorEv")]
pub extern "C" fn vendor() -> u32 {
    VENDOR.load(Ordering::Relaxed)
}

#[unsafe(export_name = "_ZN7lnCpuID3mcuEv")]
pub extern "C" fn mcu() -> u32 {
    MCU.load(Ordering::Relaxed)
}

#[unsafe(export_name = "_ZN7lnCpuID11mcuAsStringEv")]
pub extern "C" fn mcu_as_string() -> *const c_char {
    if MCU.load(Ordering::Relaxed) == LN_MCU_RISCV {
        c"RISCV".as_ptr()
    } else {
        c"????".as_ptr()
    }
}

#[unsafe(export_name = "_ZN7lnCpuID9flashSizeEv")]
pub extern "C" fn flash_size() -> i32 {
    FLASH_SIZE.load(Ordering::Relaxed) as i32
}

#[unsafe(export_name = "_ZN7lnCpuID7ramSizeEv")]
pub extern "C" fn ram_size() -> i32 {
    RAM_SIZE.load(Ordering::Relaxed) as i32
}

#[unsafe(export_name = "_ZN7lnCpuID10idAsStringEv")]
pub extern "C" fn id_as_string() -> *const c_char {
    match CHIP_ID.load(Ordering::Relaxed) {
        MCU_CH32V307_128K => c"CH32V307/128kB".as_ptr(),
        _ => c"????".as_ptr(),
    }
}

#[unsafe(export_name = "_ZN7lnCpuID10clockSpeedEv")]
pub extern "C" fn clock_speed() -> i32 {
    SystemCoreClock.load(Ordering::Relaxed) as i32
}

#[unsafe(export_name = "_ZN7lnCpuID11getSerialIDEv")]
pub extern "C" fn get_serial_id() -> u32 {
    DUMMY_SERIAL_ID
}
