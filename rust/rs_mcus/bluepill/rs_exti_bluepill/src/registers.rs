#![allow(dead_code)]
#[repr(C)]
pub struct ExtiRegisters {
    pub inten: u32,
    pub even: u32,
    pub rten: u32,
    pub ften: u32,
    pub swiev: u32,
    pub pd: u32,
}

impl ExtiRegisters {
    #[inline(always)]
    pub fn ptr() -> *mut Self {
        rs_bluepill::EXTI_BASE as *mut Self
    }
}
