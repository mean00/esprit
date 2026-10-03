#![allow(dead_code)]
#[repr(C)]
pub struct RcuRegisters {
    pub ctl: u32,      // 00 CR
    pub cfg0: u32,     // 04 CFGR
    pub int: u32,      // 08 CIR
    pub apb2rst: u32,  // 0c APB2RSTR
    pub apb1rst: u32,  // 10 APB1RSTR
    pub ahben: u32,    // 14 AHBENR
    pub apb2en: u32,   // 18 APB2ENR
    pub apb1en: u32,   // 1c APB1ENR
    pub bdctl: u32,    // 20 BDCR
    pub rstclk: u32,   // 24 CSR
    pub ahbrst: u32,   // 28 AHBRSTR
    pub cfg1: u32,     // 2c CFGR2
    pub dsv: u32,      // 30 N/A
}

impl RcuRegisters {
    #[inline(always)]
    pub fn ptr() -> *mut Self {
        rs_bluepill::RCU_BASE as *mut Self
    }
}
