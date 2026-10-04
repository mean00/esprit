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
            0 => Peripheral::Timer0,
            1 => Peripheral::Timer1,
            2 => Peripheral::Timer2,
            3 => Peripheral::Timer3,
            4 => Peripheral::Timer4,
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

    /// Disable the timer counter (CEN).
    #[inline]
    pub fn disable(&mut self) {
        unsafe {
            let mut ctl0 = read_volatile(&mut (*self.regs).ctl0);
            ctl0 &= !TIMER_CTL0_CEN;
            write_volatile(&mut (*self.regs).ctl0, ctl0);
        }
    }

    /// Reset counter value to zero.
    #[inline]
    pub fn reset_counter(&mut self) {
        unsafe {
            write_volatile(&mut (*self.regs).cnt, 0);
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
            0 => Peripheral::Timer0,
            1 => Peripheral::Timer1,
            2 => Peripheral::Timer2,
            3 => Peripheral::Timer3,
            4 => Peripheral::Timer4,
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

    /// Set the compare mode (PWM0, PWM1, ForceLow) with preload on a given channel (0..3).
    pub fn set_channel_mode(&mut self, channel: u32, mode: ChannelMode) {
        let mode_val = match mode {
            ChannelMode::Pwm0 => TIMER_CHCTL_MODE_PWM0,
            ChannelMode::Pwm1 => 0x7,
            ChannelMode::ForceLow => TIMER_CHCTL_MODE_FORCE_LOW,
        };
        let cfg = (mode_val << 4) | TIMER_CHCTL_PRELOAD_EN;

        unsafe {
            match channel {
                0 => {
                    let mut reg = read_volatile(&mut (*self.regs).chctlc0);
                    reg &= !0x00FF;
                    reg |= cfg;
                    write_volatile(&mut (*self.regs).chctlc0, reg);
                }
                1 => {
                    let mut reg = read_volatile(&mut (*self.regs).chctlc0);
                    reg &= !0xFF00;
                    reg |= cfg << 8;
                    write_volatile(&mut (*self.regs).chctlc0, reg);
                }
                2 => {
                    let mut reg = read_volatile(&mut (*self.regs).chctlc1);
                    reg &= !0x00FF;
                    reg |= cfg;
                    write_volatile(&mut (*self.regs).chctlc1, reg);
                }
                3 => {
                    let mut reg = read_volatile(&mut (*self.regs).chctlc1);
                    reg &= !0xFF00;
                    reg |= cfg << 8;
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
                0 => write_volatile(&mut (*self.regs).chcv0, value),
                1 => write_volatile(&mut (*self.regs).chcv1, value),
                2 => write_volatile(&mut (*self.regs).chcv2, value),
                3 => write_volatile(&mut (*self.regs).chcv3, value),
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
            core::ptr::addr_of_mut!((*self.regs).chcv0) as usize as u32 + (channel * 4)
        }
    }
}
