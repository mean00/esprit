//! GD32F3x / STM32F1 Cortex-M MCU registers, base offsets, and bit constants.

#![allow(dead_code)]

use core::mem::offset_of;

/// ARM Cortex-M System Control Block (subset used by CPUID, FPU, VTOR, AIRCR).
#[repr(C)]
pub struct ScbRegisters {
    pub cpuid: u32,               // 0x00
    pub icsr: u32,                // 0x04
    pub vtor: u32,                // 0x08
    pub aircr: u32,               // 0x0C
    pub scr: u32,                 // 0x10
    pub ccr: u32,                 // 0x14
    pub shp: [u8; 12],            // 0x18..0x23 System handler priority
    pub shcsr: u32,               // 0x24
    pub cfsr: u32,                // 0x28
    pub hfsr: u32,                // 0x2C
    pub dfsr: u32,                // 0x30
    pub mmfar: u32,               // 0x34
    pub bfar: u32,                // 0x38
    pub afsr: u32,                // 0x3C
    _reserved_40_84: [u32; 18],   // 0x40..0x84
    pub cpacr: u32,               // 0x88
}

const _: () = assert!(offset_of!(ScbRegisters, cpuid) == 0x00);
const _: () = assert!(offset_of!(ScbRegisters, vtor) == 0x08);
const _: () = assert!(offset_of!(ScbRegisters, aircr) == 0x0C);
const _: () = assert!(offset_of!(ScbRegisters, shp) == 0x18);
const _: () = assert!(offset_of!(ScbRegisters, shcsr) == 0x24);
const _: () = assert!(offset_of!(ScbRegisters, cpacr) == 0x88);

impl ScbRegisters {
    #[inline(always)]
    pub fn ptr() -> *mut Self {
        rs_bluepill::ARM_SCB_BASE as *mut Self
    }
}

/// ARM Cortex-M4 FPU Media and VFP Feature Registers.
#[repr(C)]
pub struct FpuMvfrRegisters {
    pub mvfr0: u32, // 0x00
    pub mvfr1: u32, // 0x04
}

const _: () = assert!(offset_of!(FpuMvfrRegisters, mvfr1) == 0x04);

impl FpuMvfrRegisters {
    #[inline(always)]
    pub fn ptr() -> *const Self {
        rs_bluepill::ARM_FPU_MVFR_BASE as *const Self
    }
}

/// ARM Cortex-M SysTick timer registers (`0xE000_E010`).
#[repr(C)]
pub struct SysTickRegisters {
    pub csr: u32,   // 0x00 Control & status
    pub rvr: u32,   // 0x04 Reload value
    pub cvr: u32,   // 0x08 Current value
    pub calib: u32, // 0x0C Calibration
}

const _: () = assert!(offset_of!(SysTickRegisters, rvr) == 0x04);
const _: () = assert!(offset_of!(SysTickRegisters, cvr) == 0x08);
const _: () = assert!(offset_of!(SysTickRegisters, calib) == 0x0C);

impl SysTickRegisters {
    #[inline(always)]
    pub fn ptr() -> *mut Self {
        rs_bluepill::ARM_SYSTICK_BASE as *mut Self
    }
}

// --- CPUID / FPU / CPACR bits ---
pub const SCB_CPUID_PARTNO_SHIFT: u32 = 4;
pub const SCB_CPUID_PARTNO_MASK: u32 = 0xFFF;
pub const SCB_CPUID_PARTNO_CM4: u32 = 0xC24;
pub const MVFR0_SIMD_SHIFT: u32 = 16;
pub const MVFR0_SIMD_MASK: u32 = 0xFF;
pub const MVFR0_HW_MUL_DIV: u32 = 0x11;
pub const SCB_CPACR_FPU_FULL_ACCESS: u32 = (3 << 20) | (3 << 22);

/// ARM Cortex-M NVIC registers (`0xE000_E100`).
#[repr(C)]
pub struct NvicPack {
    pub data: [u32; 8],
    _reserved: [u32; 24],
}

#[repr(C)]
pub struct NvicRegisters {
    pub iser: NvicPack,         // 0x000 Set enable
    pub icer: NvicPack,         // 0x080 Clear enable
    pub ispr: NvicPack,         // 0x100 Set pending
    pub icpr: NvicPack,         // 0x180 Clear pending
    pub iabr: NvicPack,         // 0x200 Active
    _reserved1: [u32; 32],      // 0x280..0x2FC (56 - 24 = 32 words)
    pub ip: [u8; 240],          // 0x300..0x3EF Priority
    _reserved2: [u32; 644],     // 0x3F0..0xDFC
    pub stir: u32,              // 0xE00 Software trigger
}

const _: () = assert!(offset_of!(NvicRegisters, icer) == 0x080);
const _: () = assert!(offset_of!(NvicRegisters, ip) == 0x300);
const _: () = assert!(offset_of!(NvicRegisters, stir) == 0xE00);

impl NvicRegisters {
    #[inline(always)]
    pub fn ptr() -> *mut Self {
        rs_bluepill::ARM_NVIC_BASE as *mut Self
    }
}

// --- SCB AIRCR / CCR / SHCSR / NVIC priority constants ---
pub const SCB_AIRCR_KEY: u32 = 0x05FA << 16;
pub const SCB_AIRCR_VECTRESET: u32 = 1 << 0;
pub const SCB_AIRCR_SYSRESETREQ: u32 = 1 << 2;
pub const SCB_AIRCR_PRIGROUP_4BITS: u32 = 3 << 8;
pub const SCB_AIRCR_ENDIANNESS_MASK: u32 = 7;

pub const SCB_CCR_DIV_0_TRP: u32 = 1 << 4;
pub const SCB_CCR_STKALIGN: u32 = 1 << 9;
pub const SCB_SHCSR_FAULTS_ENA: u32 = 7 << 16;

pub const CONFIG_PRIO_BITS: i32 = 4;
pub const CONFIG_LIBRARY_LOWEST_INTERRUPT_PRIORITY: i32 = 0xF;
pub const CONFIG_LIBRARY_MAX_SYSCALL_INTERRUPT_PRIORITY: i32 = 5;
pub const CONFIG_MAX_SYSCALL_INTERRUPT_PRIORITY: u8 =
    (CONFIG_LIBRARY_MAX_SYSCALL_INTERRUPT_PRIORITY << (8 - CONFIG_PRIO_BITS)) as u8;

pub const LN_IRQ_WWDG: i32 = 0;
pub const LN_IRQ_ADC0_1: i32 = 18;
pub const LN_IRQ_ARM_LAST: i32 = 61;
