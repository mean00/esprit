#![allow(dead_code)]
#![allow(dead_code)]

#[repr(C)]
pub struct AdcRegisters {
    pub stat: u32,
    pub ctl0: u32,
    pub ctl1: u32,
    pub sampt0: u32,
    pub sampt1: u32,
    pub iofr0: u32,
    pub iofr1: u32,
    pub iofr2: u32,
    pub iofr3: u32,
    pub wdt: u32,
    pub rsq0: u32,
    pub rsq1: u32,
    pub rsq2: u32,
    pub isq: u32,
    pub idata0: u32,
    pub idata1: u32,
    pub idata2: u32,
    pub idata3: u32,
    pub rdata: u32,
}

impl AdcRegisters {
    #[inline(always)]
    pub fn ptr(instance: u32) -> *mut Self {
        match instance {
            0 => 0x40012400 as *mut Self, // ADC0 (ADC1 in STM32 terminology)
            1 => 0x40012800 as *mut Self, // ADC1
            2 => 0x40013C00 as *mut Self, // ADC2
            _ => panic!("Invalid ADC instance"),
        }
    }
}
