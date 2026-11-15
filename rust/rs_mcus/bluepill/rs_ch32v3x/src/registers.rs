//! CH32V3x PFIC, SysTick, and CSR constants and register layouts.

#![allow(dead_code)]

use core::mem::offset_of;

/// WCH QingKe SysTick register block (`rs_bluepill::STK_BASE`, `0xE000_F000`).
#[repr(C)]
pub struct SysTickRegs {
    pub ctlr: u32, // 0x00
    pub sr: u32,   // 0x04
    pub cnt: u64,  // 0x08
    pub cmp: u64,  // 0x10
}

const _: () = assert!(offset_of!(SysTickRegs, sr) == 0x04);
const _: () = assert!(offset_of!(SysTickRegs, cnt) == 0x08);
const _: () = assert!(offset_of!(SysTickRegs, cmp) == 0x10);

impl SysTickRegs {
    #[inline(always)]
    pub fn ptr() -> *mut Self {
        rs_bluepill::STK_BASE as *mut Self
    }
}

pub const STK_CTLR_STE: u32 = 1 << 0;
pub const STK_SR_CNTIF: u32 = 1 << 0;

/// WCH PFIC interrupt controller register block (`rs_bluepill::PFIC_BASE`, `0xE000_E000`).
#[repr(C)]
pub struct PficRegs {
    pub isr: [u32; 4],        // 0x000
    _reserved0: [u32; 4],     // 0x010
    pub ipr: [u32; 4],        // 0x020
    _reserved1: [u32; 4],     // 0x030
    pub ithreshold: u32,      // 0x040
    _reserved2: u32,          // 0x044
    pub cfgr: u32,            // 0x048
    pub gisr: u32,            // 0x04C
    pub vtfidr: [u8; 4],      // 0x050
    _reserved3: [u32; 3],     // 0x054
    pub vtfaddr: [u32; 4],    // 0x060
    _reserved4: [u32; 36],    // 0x070..0x0FC
    pub ienr: [u32; 4],       // 0x100
    _reserved5: [u32; 28],    // 0x110..0x17C
    pub irer: [u32; 4],       // 0x180
    _reserved6: [u32; 28],    // 0x190..0x1FC
    pub ipsr: [u32; 4],       // 0x200
    _reserved7: [u32; 28],    // 0x210..0x27C
    pub iprr: [u32; 4],       // 0x280
    _reserved8: [u32; 28],    // 0x290..0x2FC
    pub iactr: [u32; 4],      // 0x300
    _reserved9: [u32; 60],    // 0x310..0x3FC
    pub iprioir: [u32; 64],   // 0x400..0x4FC
    _reserved10: [u32; 516],  // 0x500..0xD0C
    pub sctlr: u32,           // 0xD10
}

const _: () = assert!(offset_of!(PficRegs, gisr) == 0x04C);
const _: () = assert!(offset_of!(PficRegs, vtfaddr) == 0x060);
const _: () = assert!(offset_of!(PficRegs, ienr) == 0x100);
const _: () = assert!(offset_of!(PficRegs, irer) == 0x180);
const _: () = assert!(offset_of!(PficRegs, ipsr) == 0x200);
const _: () = assert!(offset_of!(PficRegs, iprioir) == 0x400);
const _: () = assert!(offset_of!(PficRegs, sctlr) == 0xD10);

impl PficRegs {
    #[inline(always)]
    pub fn ptr() -> *mut Self {
        rs_bluepill::PFIC_BASE as *mut Self
    }
}

pub const CH32_SYSCR_HWSTKEN: u32 = 1 << 0;
pub const CH32_SYSCR_INESTEN: u32 = 1 << 1;
pub const CH32_SYSCR_MPTCFG_2NESTED: u32 = 1 << 2;
pub const CH32_SYSCR_HWSTKOVEN: u32 = 1 << 4;

pub const LN_RISCV_FPU_MODE_1: u32 = 1 << 13;
pub const MTVEC_VECTORED_MODE: u32 = 3;

pub const PFIC_GISR_NESTSTA_SHIFT: u32 = 8;
pub const PFIC_VTFADDR_ENABLE: u32 = 1 << 0;
pub const PFIC_SCTLR_SYSRESET: u32 = 1 << 31;

pub const DEFAULT_PRIO: u32 = 5;
pub const SYSTICK_PRIO: i32 = 6;
pub const SW_IRQ_PRIO: i32 = 7;

pub const LN_IRQ_SYSTICK: usize = 78;
pub const LN_IRQ_SW: usize = 79;
pub const PFIC_SW_IRQ_NUM: usize = 14;
