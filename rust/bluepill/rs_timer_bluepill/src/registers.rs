#![allow(dead_code)]
use rs_bluepill::{TIM1_BASE, TIM2_BASE, TIM3_BASE, TIM4_BASE, TIM5_BASE};

#[repr(C)]
pub struct TimerRegisters {
    pub ctl0: u32,
    pub ctl1: u32,
    pub smcfg: u32,
    pub dien: u32,
    pub intf: u32,
    pub swevg: u32,
    pub chctlc0: u32,
    pub chctlc1: u32,
    pub chctl2: u32,
    pub cnt: u32,
    pub psc: u32,
    pub car: u32,
    pub crep: u32,
    pub chcv0: u32,
    pub chcv1: u32,
    pub chcv2: u32,
    pub chcv3: u32,
    pub dmacfg: u32,
    pub dmatb: u32,
}

impl TimerRegisters {
    pub fn ptr(timer_idx: u32) -> *mut Self {
        match timer_idx {
            0 => TIM1_BASE as *mut Self,
            1 => TIM2_BASE as *mut Self,
            2 => TIM3_BASE as *mut Self,
            3 => TIM4_BASE as *mut Self,
            4 => TIM5_BASE as *mut Self,
            _ => TIM1_BASE as *mut Self, // Fallback
        }
    }
}
