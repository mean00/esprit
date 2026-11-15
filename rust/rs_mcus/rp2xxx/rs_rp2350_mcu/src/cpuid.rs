//! C++ `lnCpuID::*` ABI for RP2350.

pub use rs_rp2xxx::cpuid::{flash_size, get_serial_id, identify, vendor};

const LN_MCU_RP2350: i32 = 6;
const RP2350_RAM_SIZE_KB: i32 = 520;
const RP2350_SYS_CLOCK_HZ: i32 = 150_000_000;

#[unsafe(export_name = "_ZN7lnCpuID3mcuEv")]
pub extern "C" fn mcu() -> i32 {
    LN_MCU_RP2350
}

#[unsafe(export_name = "_ZN7lnCpuID11mcuAsStringEv")]
pub extern "C" fn mcu_as_string() -> *const u8 {
    c"RP2350".as_ptr().cast()
}

#[unsafe(export_name = "_ZN7lnCpuID7ramSizeEv")]
pub extern "C" fn ram_size() -> i32 {
    RP2350_RAM_SIZE_KB
}

#[unsafe(export_name = "_ZN7lnCpuID10idAsStringEv")]
pub extern "C" fn id_as_string() -> *const u8 {
    mcu_as_string()
}

#[unsafe(export_name = "_ZN7lnCpuID10clockSpeedEv")]
pub extern "C" fn clock_speed() -> i32 {
    RP2350_SYS_CLOCK_HZ
}
