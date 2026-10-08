//! Encapsulated hardware Timer peripheral for Bluepill / GD32F3 / CH32V3xx.
//!
//! Follows Esprit Rust Rule 1 (No Magic Numbers) and Rule 6 (Idiomatic Rust).

use crate::registers::*;
use core::ptr::{read_volatile, write_volatile};
use rs_rcu_bluepill::{enable, get_clock, Peripheral};

/// Output compare modes for timer channels.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ChannelMode {
    /// Active while counter is less than compare value (CNT < CCRx).
    Pwm0,
    /// Inactive while counter is less than compare value (CNT >= CCRx).
    Pwm1,
    /// Force channel output low/inactive.
    ForceLow,
}

/// Managed hardware Timer instance.
pub struct Timer {
    timer_idx: u32,
    regs: *mut TimerRegisters,
}

impl Timer {
    /// Initialise a hardware timer by index (0 = TIM1, 1 = TIM2, 2 = TIM3, 3 = TIM4, 4 = TIM5).
    pub fn new(timer_idx: u32) -> Result<Self, &'static str> {
        let periph = match timer_idx {
            TIMER_INSTANCE_0 => Peripheral::Timer0,
            TIMER_INSTANCE_1 => Peripheral::Timer1,
            TIMER_INSTANCE_2 => Peripheral::Timer2,
            TIMER_INSTANCE_3 => Peripheral::Timer3,
            TIMER_INSTANCE_4 => Peripheral::Timer4,
            _ => return Err("Invalid timer index"),
        };
        enable(periph);

        Ok(Self {
            timer_idx,
            regs: TimerRegisters::ptr(timer_idx),
        })
    }

    /// Enable the timer counter (CEN).
    #[inline]
    pub fn enable(&mut self) {
        unsafe {
            let mut ctl0 = read_volatile(&mut (*self.regs).ctl0);
            ctl0 |= TIMER_CTL0_CEN;
            write_volatile(&mut (*self.regs).ctl0, ctl0);
        }
    }

    /// Disable the timer counter (CEN) and clear counter.
    #[inline]
    pub fn disable(&mut self) {
        unsafe {
            let mut ctl0 = read_volatile(&mut (*self.regs).ctl0);
            ctl0 &= !TIMER_CTL0_CEN;
            write_volatile(&mut (*self.regs).ctl0, ctl0);
            write_volatile(&mut (*self.regs).cnt, 0);
        }
    }

    /// Reset counter value to zero.
    #[inline]
    pub fn reset_counter(&mut self) {
        unsafe {
            write_volatile(&mut (*self.regs).cnt, 0);
        }
    }

    /// Set counter value.
    #[inline]
    pub fn set_counter(&mut self, cnt: u32) {
        unsafe {
            write_volatile(&mut (*self.regs).cnt, cnt);
        }
    }


    /// Set counter prescaler value.
    #[inline]
    pub fn set_prescaler(&mut self, psc: u16) {
        unsafe {
            write_volatile(&mut (*self.regs).psc, psc as u32);
        }
    }

    /// Set auto-reload register (CAR / ARR).
    #[inline]
    pub fn set_auto_reload(&mut self, car: u32) {
        unsafe {
            write_volatile(&mut (*self.regs).car, car);
        }
    }

    /// Read auto-reload register.
    #[inline]
    pub fn auto_reload(&self) -> u32 {
        unsafe { read_volatile(&mut (*self.regs).car) }
    }

    /// Enable or disable auto-reload preload (ARPE).
    #[inline]
    pub fn set_auto_reload_preload(&mut self, enabled: bool) {
        unsafe {
            let mut ctl0 = read_volatile(&mut (*self.regs).ctl0);
            if enabled {
                ctl0 |= TIMER_CTL0_ARPE;
            } else {
                ctl0 &= !TIMER_CTL0_ARPE;
            }
            write_volatile(&mut (*self.regs).ctl0, ctl0);
        }
    }

    /// Configure timer frequency based on the MCU clock tree.
    ///
    /// Computes and sets prescaler=0 and auto-reload = (clock / freq) - 1.
    /// Returns the period (rollover count = CAR + 1).
    pub fn set_frequency(&mut self, frequency_hz: u32) -> Result<u32, &'static str> {
        let periph = match self.timer_idx {
            TIMER_INSTANCE_0 => Peripheral::Timer0,
            TIMER_INSTANCE_1 => Peripheral::Timer1,
            TIMER_INSTANCE_2 => Peripheral::Timer2,
            TIMER_INSTANCE_3 => Peripheral::Timer3,
            TIMER_INSTANCE_4 => Peripheral::Timer4,
            _ => return Err("Invalid timer"),
        };
        let clock = get_clock(periph);
        if clock == 0 {
            return Err("Timer clock reported 0 Hz");
        }

        let divider = (clock + (frequency_hz / 2)) / frequency_hz;
        let divider = if divider == 0 { 1 } else { divider };

        self.set_prescaler(0);
        self.set_auto_reload(divider - 1);
        self.reset_counter();

        Ok(divider)
    }

    /// Set the compare mode (PWM0, PWM1, ForceLow) on a given channel (0..3).
    pub fn set_channel_mode(&mut self, channel: u32, mode: ChannelMode) {
        let mode_val = match mode {
            ChannelMode::Pwm0 => TIMER_CHCTL_MODE_PWM0,
            ChannelMode::Pwm1 => TIMER_CHCTL_MODE_PWM1,
            ChannelMode::ForceLow => TIMER_CHCTL_MODE_FORCE_LOW,
        };
        let cfg = mode_val << TIMER_CHCTL_MODE_SHIFT;


        unsafe {
            match channel {
                TIMER_CHANNEL_0 => {
                    let mut reg = read_volatile(&mut (*self.regs).chctlc0);
                    reg &= !TIMER_CHCTLC_CH_LOW_MASK;
                    reg |= cfg;
                    write_volatile(&mut (*self.regs).chctlc0, reg);
                }
                TIMER_CHANNEL_1 => {
                    let mut reg = read_volatile(&mut (*self.regs).chctlc0);
                    reg &= !TIMER_CHCTLC_CH_HIGH_MASK;
                    reg |= cfg << TIMER_CHCTLC_CH_HIGH_SHIFT;
                    write_volatile(&mut (*self.regs).chctlc0, reg);
                }
                TIMER_CHANNEL_2 => {
                    let mut reg = read_volatile(&mut (*self.regs).chctlc1);
                    reg &= !TIMER_CHCTLC_CH_LOW_MASK;
                    reg |= cfg;
                    write_volatile(&mut (*self.regs).chctlc1, reg);
                }
                TIMER_CHANNEL_3 => {
                    let mut reg = read_volatile(&mut (*self.regs).chctlc1);
                    reg &= !TIMER_CHCTLC_CH_HIGH_MASK;
                    reg |= cfg << TIMER_CHCTLC_CH_HIGH_SHIFT;
                    write_volatile(&mut (*self.regs).chctlc1, reg);
                }
                _ => {}
            }
        }
    }

    /// Set compare value (CCRx / CHCVx) for a channel.
    pub fn set_channel_compare(&mut self, channel: u32, value: u32) {
        unsafe {
            match channel {
                TIMER_CHANNEL_0 => write_volatile(&mut (*self.regs).chcv0, value),
                TIMER_CHANNEL_1 => write_volatile(&mut (*self.regs).chcv1, value),
                TIMER_CHANNEL_2 => write_volatile(&mut (*self.regs).chcv2, value),
                TIMER_CHANNEL_3 => write_volatile(&mut (*self.regs).chcv3, value),
                _ => {}
            }
        }
    }

    /// Enable or disable output on a channel in CHCTL2 (CCER).
    pub fn set_channel_output(&mut self, channel: u32, enabled: bool) {
        let bit = TIMER_CHCTL2_CH0EN << (channel * TIMER_CHCTL2_CHANNEL_SHIFT);
        unsafe {
            let mut chctl2 = read_volatile(&mut (*self.regs).chctl2);
            if enabled {
                chctl2 |= bit;
            } else {
                chctl2 &= !bit;
            }
            write_volatile(&mut (*self.regs).chctl2, chctl2);
        }
    }

    /// Enable or disable DMA request on a channel compare event (DIEN).
    pub fn enable_channel_dma(&mut self, channel: u32, enabled: bool) {
        let bit = 1 << (TIMER_DIEN_CH_DMA_BASE_BIT + channel);
        unsafe {
            let mut dien = read_volatile(&mut (*self.regs).dien);
            if enabled {
                dien |= bit;
            } else {
                dien &= !bit;
            }
            write_volatile(&mut (*self.regs).dien, dien);
        }
    }

    /// Set DMA request source to capture/compare event (DMAS bit in CTL1).
    pub fn set_dma_on_compare_event(&mut self, enabled: bool) {
        unsafe {
            let mut ctl1 = read_volatile(&mut (*self.regs).ctl1);
            if enabled {
                ctl1 |= TIMER_CTL1_DMAS;
            } else {
                ctl1 &= !TIMER_CTL1_DMAS;
            }
            write_volatile(&mut (*self.regs).ctl1, ctl1);
        }
    }

    /// Return the peripheral memory address of the compare register (CHCVx).
    pub fn channel_compare_reg_addr(&self, channel: u32) -> u32 {
        unsafe {
            core::ptr::addr_of_mut!((*self.regs).chcv0) as usize as u32 + (channel * TIMER_REG_OFFSET_BYTES)
        }
    }
}

static mut TIMER_HANDLERS: [Option<&'static dyn rs_esprit::TimerHandler>; TIMER_MAX_INSTANCES] = [None, None, None, None, None];

impl rs_esprit::Timer for Timer {
    #[inline]
    fn set_frequency(&mut self, frequency_hz: u32) -> Result<u32, &'static str> {
        self.set_frequency(frequency_hz)
    }

    #[inline]
    fn enable(&mut self) {
        self.enable();
    }

    #[inline]
    fn disable(&mut self) {
        self.disable();
    }

    #[inline]
    fn set_handler(&mut self, handler: Option<&'static dyn rs_esprit::TimerHandler>) {
        unsafe {
            if (self.timer_idx as usize) < TIMER_MAX_INSTANCES {
                TIMER_HANDLERS[self.timer_idx as usize] = handler;
            }
        }
    }

    #[inline]
    fn single_shot(&mut self, duration_ms: u32, _up: bool) {
        unsafe {
            let mut ctl0 = read_volatile(&mut (*self.regs).ctl0);
            ctl0 &= !TIMER_CTL0_CEN;
            write_volatile(&mut (*self.regs).ctl0, ctl0);

            write_volatile(&mut (*self.regs).psc, TIMER_PRESCALER_1MS_72MHZ);
            write_volatile(&mut (*self.regs).car, duration_ms);

            ctl0 |= TIMER_CTL0_OPM;
            ctl0 |= TIMER_CTL0_CEN;
            write_volatile(&mut (*self.regs).ctl0, ctl0);
        }
    }
}

