#![allow(dead_code)]
use rs_bluepill::{SPI1_BASE, SPI2_BASE, SPI3_BASE};

#[repr(C)]
pub struct SpiRegisters {
    pub cr1: u32,
    pub cr2: u32,
    pub sr: u32,
    pub dr: u32,
    pub crcpr: u32,
    pub rxcrcr: u32,
    pub txcrcr: u32,
    pub i2scfgr: u32,
    pub i2spr: u32,
}

impl SpiRegisters {
    #[inline(always)]
    pub fn ptr(instance: u32) -> *mut Self {
        match instance {
            0 => SPI1_BASE as *mut Self,
            1 => SPI2_BASE as *mut Self,
            2 => SPI3_BASE as *mut Self,
            _ => SPI1_BASE as *mut Self,
        }
    }
}
