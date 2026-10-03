#![allow(dead_code)]
use rs_bluepill::{I2C1_BASE, I2C2_BASE};

#[repr(C)]
pub struct I2cRegisters {
    pub cr1: u32,
    pub cr2: u32,
    pub oar1: u32,
    pub oar2: u32,
    pub dr: u32,
    pub sr1: u32,
    pub sr2: u32,
    pub ccr: u32,
    pub trise: u32,
}

impl I2cRegisters {
    #[inline(always)]
    pub fn ptr(instance: u32) -> *mut Self {
        match instance {
            0 => I2C1_BASE as *mut Self,
            1 => I2C2_BASE as *mut Self,
            _ => I2C1_BASE as *mut Self,
        }
    }
}
