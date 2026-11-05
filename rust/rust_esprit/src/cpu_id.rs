//! Host CPU / MCU identification.

use crate::c_api::rn_cpuid_c;
use core::ffi::CStr;

/// Silicon vendor.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum Vendor {
    Unknown = 0,
    Stm     = 1,
    Gd      = 2,
    Wch     = 3,
    Esp     = 4,
    Rp      = 5,
}

impl From<u32> for Vendor {
    fn from(val: u32) -> Self {
        match val {
            1 => Vendor::Stm,
            2 => Vendor::Gd,
            3 => Vendor::Wch,
            4 => Vendor::Esp,
            5 => Vendor::Rp,
            _ => Vendor::Unknown,
        }
    }
}

/// MCU core architecture / family.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum Mcu {
    Unknown = 0,
    ArmF3   = 1,
    ArmF4   = 2,
    RiscV   = 3,
    Xtensa  = 4,
    Rp2040  = 5,
    Rp2350  = 6,
}

impl From<u32> for Mcu {
    fn from(val: u32) -> Self {
        match val {
            1 => Mcu::ArmF3,
            2 => Mcu::ArmF4,
            3 => Mcu::RiscV,
            4 => Mcu::Xtensa,
            5 => Mcu::Rp2040,
            6 => Mcu::Rp2350,
            _ => Mcu::Unknown,
        }
    }
}

/// Host CPU / MCU identification and capabilities.
pub struct CpuId;

impl CpuId {
    /// Return the detected silicon vendor.
    #[inline(always)]
    pub fn vendor() -> Vendor {
        unsafe { rn_cpuid_c::ln_cpuid_vendor().into() }
    }

    /// Return the MCU architecture / family.
    #[inline(always)]
    pub fn mcu() -> Mcu {
        unsafe { rn_cpuid_c::ln_cpuid_mcu().into() }
    }

    /// Return MCU architecture name as a string slice (e.g. "ARMF3", "RISCV", "RP2040").
    pub fn mcu_name() -> &'static str {
        unsafe {
            let ptr = rn_cpuid_c::ln_cpuid_mcu_as_string();
            if ptr.is_null() {
                ""
            } else {
                CStr::from_ptr(ptr).to_str().unwrap_or("???")
            }
        }
    }

    /// Return flash size in KiB.
    #[inline(always)]
    pub fn flash_size_kb() -> usize {
        unsafe { rn_cpuid_c::ln_cpuid_flash_size() as usize }
    }

    /// Return RAM size in KiB.
    #[inline(always)]
    pub fn ram_size_kb() -> usize {
        unsafe { rn_cpuid_c::ln_cpuid_ram_size() as usize }
    }

    /// Return chip model identifier as a string slice (e.g. "GD32F303", "ESP32C3", etc.).
    pub fn chip_name() -> &'static str {
        unsafe {
            let ptr = rn_cpuid_c::ln_cpuid_id_as_string();
            if ptr.is_null() {
                ""
            } else {
                CStr::from_ptr(ptr).to_str().unwrap_or("???")
            }
        }
    }

    /// Return CPU clock frequency in Hz.
    #[inline(always)]
    pub fn clock_speed_hz() -> u32 {
        unsafe { rn_cpuid_c::ln_cpuid_clock_speed() as u32 }
    }

    /// Return 32-bit unique hardware serial number.
    #[inline(always)]
    pub fn serial_id() -> u32 {
        unsafe { rn_cpuid_c::ln_cpuid_serial_id() }
    }
}
