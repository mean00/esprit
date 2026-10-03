#![allow(dead_code)]
#[repr(C)]
pub struct AfioRegisters {
    pub ec: u32,
    pub pcf0: u32,
    pub extiss: [u32; 4],
    pub dummy: u32,
    pub pcf1: u32,
}

impl AfioRegisters {
    #[inline(always)]
    pub fn ptr() -> *mut Self {
        rs_bluepill::AFIO_BASE as *mut Self
    }
}
