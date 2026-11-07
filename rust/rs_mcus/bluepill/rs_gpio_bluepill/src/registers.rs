#![allow(dead_code)]
#[repr(C)]
pub struct GpioRegisters {
    pub ctl0: u32,
    pub ctl1: u32,
    pub istat: u32,
    pub octl: u32,
    pub bop: u32,
    pub bc: u32,
    pub lock: u32,
}

pub const CTL_MD_INPUT: u32 = 0;
pub const CTL_INPUT_ANALOG: u32 = 0;
pub const CTL_INPUT_FLOATING: u32 = 1;
pub const CTL_INPUT_PULLUP_PULLDOWN: u32 = 2;

pub const CTL_OUTPUT_PP: u32 = 0;
pub const CTL_OUTPUT_OD: u32 = 1;
pub const CTL_OUTPUT_ALTERNATE_PP: u32 = 2;
pub const CTL_OUTPUT_ALTERNATE_OD: u32 = 3;

pub const SPEED_50MHZ: u32 = 3;
pub const SPEED_10MHZ: u32 = 1;
pub const SPEED_2MHZ: u32 = 2;

#[inline(always)]
pub fn gpio_set(mode: u32, ctl: u32) -> u32 {
    (ctl << 2) | mode
}
