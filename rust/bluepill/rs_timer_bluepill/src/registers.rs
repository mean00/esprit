#![allow(dead_code)]
use rs_bluepill::{TIM1_BASE, TIM2_BASE, TIM3_BASE, TIM4_BASE, TIM5_BASE, TIM6_BASE, TIM7_BASE, TIM8_BASE};

pub const TIMER_INSTANCE_0: u32 = 0;
pub const TIMER_INSTANCE_1: u32 = 1;
pub const TIMER_INSTANCE_2: u32 = 2;
pub const TIMER_INSTANCE_3: u32 = 3;
pub const TIMER_INSTANCE_4: u32 = 4;
pub const TIMER_INSTANCE_5: u32 = 5;
pub const TIMER_INSTANCE_6: u32 = 6;
pub const TIMER_INSTANCE_7: u32 = 7;
pub const TIMER_MAX_INSTANCES: usize = 5;
pub const STOPWATCH_MAX_INSTANCES: usize = 8;
pub const STOPWATCH_COUNTER_MAX: u32 = 0xFFFF;

pub const TIMER_CHANNEL_0: u32 = 0;
pub const TIMER_CHANNEL_1: u32 = 1;
pub const TIMER_CHANNEL_2: u32 = 2;
pub const TIMER_CHANNEL_3: u32 = 3;

pub const TIMER_REG_OFFSET_BYTES: u32 = 4;
pub const TIMER_PRESCALER_1MS_72MHZ: u32 = 71999;
pub const TIMER_HANDLE_CHANNEL_SHIFT: u32 = 8;
pub const TIMER_HANDLE_MASK: u32 = 0xFF;
pub const PWM_HALF_DUTY_DIVISOR: u32 = 2;

// --- Timer Control & Config Bits (GD32F3 / STM32F1) ---
pub const TIMER_CTL0_CEN: u32 = 1 << 0;
pub const TIMER_CTL0_UDIS: u32 = 1 << 1;
pub const TIMER_CTL0_URS: u32 = 1 << 2;
pub const TIMER_CTL0_OPM: u32 = 1 << 3;
pub const TIMER_CTL0_DIR: u32 = 1 << 4;
pub const TIMER_CTL0_CAM: u32 = 3 << 5;
pub const TIMER_CTL0_ARPE: u32 = 1 << 7;
pub const TIMER_CTL0_CKDIV: u32 = 3 << 8;

pub const TIMER_CTL1_DMAS: u32 = 1 << 3;

pub const TIMER_DIEN_CH_DMA_BASE_BIT: u32 = 9; // CH0DEN = 1<<9, CH1DEN = 1<<10, etc.

pub const TIMER_CHCTL_MODE_FORCE_LOW: u32 = 0x4;
pub const TIMER_CHCTL_MODE_PWM0: u32 = 0x6;
pub const TIMER_CHCTL_MODE_PWM1: u32 = 0x7;
pub const TIMER_CHCTL_PRELOAD_EN: u32 = 1 << 3;
pub const TIMER_CHCTL_MODE_SHIFT: u32 = 4;

pub const TIMER_CHCTLC_CH_LOW_MASK: u32 = 0x00FF;
pub const TIMER_CHCTLC_CH_HIGH_MASK: u32 = 0xFF00;
pub const TIMER_CHCTLC_CH_HIGH_SHIFT: u32 = 8;

pub const TIMER_CHCTL2_CH0EN: u32 = 1 << 0;
pub const TIMER_CHCTL2_CHANNEL_SHIFT: u32 = 4;

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
            TIMER_INSTANCE_0 => TIM1_BASE as *mut Self,
            TIMER_INSTANCE_1 => TIM2_BASE as *mut Self,
            TIMER_INSTANCE_2 => TIM3_BASE as *mut Self,
            TIMER_INSTANCE_3 => TIM4_BASE as *mut Self,
            TIMER_INSTANCE_4 => TIM5_BASE as *mut Self,
            TIMER_INSTANCE_5 => TIM6_BASE as *mut Self,
            TIMER_INSTANCE_6 => TIM7_BASE as *mut Self,
            TIMER_INSTANCE_7 => TIM8_BASE as *mut Self,
            _ => TIM1_BASE as *mut Self, // Fallback
        }
    }
}
