#![allow(unused_variables, unused_imports, unused_parens)]
#![allow(non_camel_case_types, non_snake_case, dead_code)]

#![no_std]

pub mod registers;

use registers::*;
use rs_gpio_bluepill::Pin;
use rs_dma_bluepill::DmaChannel;
use rs_rcu_bluepill::{Peripheral, enable};
use core::ptr::{read_volatile, write_volatile};
use core::ffi::c_void;

pub fn pin_to_adc_channel(pin: u32) -> u32 {
    let p = pin;
    if p < 8 {
        p // PA0-PA7 -> CH0-CH7
    } else if p >= 16 && p <= 17 {
        p - 16 + 8 // PB0-PB1 -> CH8-CH9
    } else if p >= 32 && p <= 37 {
        p - 32 + 10 // PC0-PC5 -> CH10-CH15
    } else {
        0 // Default fallback
    }
}

/// Idiomatic struct representing a DMA-driven multi-channel ADC sequencer.
pub struct TimingAdc {
    instance: u32,
}

impl TimingAdc {
    pub fn new(instance: u32) -> Self {
        if instance == 0 {
            enable(Peripheral::Adc0);
        } else {
            enable(Peripheral::Adc1);
        }
        Self { instance }
    }

    #[inline(always)]
    pub fn instance(&self) -> u32 {
        self.instance
    }

    pub fn configured_pins_count(&self) -> u32 {
        let regs = AdcRegisters::ptr(self.instance);
        unsafe { ((read_volatile(&mut (*regs).rsq0) >> ADC_RSQ0_LEN_POS) & 0x0F) + 1 }
    }

    pub fn set_source(&mut self, _timer: u32, _channel: u32, _fq: u32, pins: &[Pin]) -> bool {
        if self.instance != 0 {
            return false;
        }
        let regs = AdcRegisters::ptr(self.instance);
        let nb_pins = pins.len() as u32;

        unsafe {
            // Setup Scan Mode
            let mut ctl0 = read_volatile(&mut (*regs).ctl0);
            ctl0 |= ADC_CTL0_SM;
            write_volatile(&mut (*regs).ctl0, ctl0);

            // Configure sequencer length
            let rsq0 = (nb_pins.saturating_sub(1) & 0x0F) << ADC_RSQ0_LEN_POS;
            write_volatile(&mut (*regs).rsq0, rsq0);

            // Map pins to RSQ2 (first 5 channels)
            let mut rsq2 = 0;
            for (i, &p) in pins.iter().enumerate().take(5) {
                let ch = pin_to_adc_channel(p as u32);
                rsq2 |= ch << (5 * i);
            }
            write_volatile(&mut (*regs).rsq2, rsq2);

            // Setup External Trigger
            let mut ctl1 = read_volatile(&mut (*regs).ctl1);
            ctl1 &= !ADC_CTL1_ETSRC_MASK;
            ctl1 |= ADC_CTL1_ETSRC_SWSTART;
            ctl1 |= ADC_CTL1_ETERC;
            write_volatile(&mut (*regs).ctl1, ctl1);

            // Enable ADC
            ctl1 |= ADC_CTL1_ADCON;
            write_volatile(&mut (*regs).ctl1, ctl1);
        }

        true
    }

    pub fn multi_read(&mut self, nb_sample_per_channel: u32, output: &mut [u16]) -> bool {
        let regs = AdcRegisters::ptr(self.instance);
        let nb_pins = self.configured_pins_count();
        let total_samples = nb_sample_per_channel * nb_pins;
        if (output.len() as u32) < total_samples {
            return false;
        }

        let mut dma = DmaChannel::new(ADC0_DMA_ENGINE, ADC0_DMA_CHANNEL_IDX);

        unsafe {
            let mut ctl1 = read_volatile(&mut (*regs).ctl1);
            ctl1 |= ADC_CTL1_DMA;
            write_volatile(&mut (*regs).ctl1, ctl1);

            dma.begin_tx_transfer(
                &(*regs).rdata as *const _ as u32,
                output.as_mut_ptr() as u32,
                total_samples,
                true, // 16-bit PSIZE
                true, // 16-bit MSIZE
            );

            ctl1 |= ADC_CTL1_SWRCST;
            write_volatile(&mut (*regs).ctl1, ctl1);

            while !dma.is_transfer_complete() {}

            dma.end_transfer();

            ctl1 &= !ADC_CTL1_DMA;
            write_volatile(&mut (*regs).ctl1, ctl1);
        }

        true
    }

    pub fn async_read(
        &mut self,
        nb_sample_per_channel: u32,
        output: &mut [u16],
        cb: Option<unsafe extern "C" fn(*mut c_void)>,
        ctx: *mut c_void,
    ) -> bool {
        self.multi_read(nb_sample_per_channel, output);
        if let Some(callback) = cb {
            unsafe { callback(ctx); }
        }
        true
    }
}

/// Idiomatic struct representing a single-pin simple ADC converter.
pub struct SimpleAdc {
    instance: u32,
    pin: u32,
}

impl SimpleAdc {
    pub fn new(instance: u32, pin: u32) -> Self {
        if instance == 0 {
            enable(Peripheral::Adc0);
        } else {
            enable(Peripheral::Adc1);
        }
        Self { instance, pin }
    }

    #[inline(always)]
    pub fn instance(&self) -> u32 {
        self.instance
    }

    #[inline(always)]
    pub fn pin(&self) -> u32 {
        self.pin
    }

    pub fn set_smpt(&mut self, _smpt: u32) {}

    pub fn read(&self) -> i32 {
        let regs = AdcRegisters::ptr(self.instance);
        unsafe {
            let ch = pin_to_adc_channel(self.pin);

            let mut ctl1 = read_volatile(&mut (*regs).ctl1);
            ctl1 &= !ADC_CTL1_ETSRC_MASK;
            ctl1 |= ADC_CTL1_ETSRC_SWSTART;
            ctl1 |= ADC_CTL1_ETERC;
            ctl1 |= ADC_CTL1_ADCON;
            write_volatile(&mut (*regs).ctl1, ctl1);

            let mut nop_ctr = 0;
            while nop_ctr < 1000 {
                core::arch::asm!("nop");
                nop_ctr += 1;
            }

            ctl1 = read_volatile(&mut (*regs).ctl1);
            ctl1 |= ADC_CTL1_RSTCLB;
            write_volatile(&mut (*regs).ctl1, ctl1);
            while (read_volatile(&mut (*regs).ctl1) & ADC_CTL1_RSTCLB) != 0 {
                core::arch::asm!("nop");
            }

            ctl1 = read_volatile(&mut (*regs).ctl1);
            ctl1 |= ADC_CTL1_CLB;
            write_volatile(&mut (*regs).ctl1, ctl1);
            while (read_volatile(&mut (*regs).ctl1) & ADC_CTL1_CLB) != 0 {
                core::arch::asm!("nop");
            }

            write_volatile(&mut (*regs).rsq0, 0);
            write_volatile(&mut (*regs).rsq2, ch);

            ctl1 = read_volatile(&mut (*regs).ctl1);
            ctl1 |= ADC_CTL1_SWRCST;
            write_volatile(&mut (*regs).ctl1, ctl1);

            while (read_volatile(&mut (*regs).stat) & ADC_STAT_EOC) == 0 {
                core::arch::asm!("nop");
            }

            let data = read_volatile(&mut (*regs).rdata) & ADC_RDATA_DATA_MASK;
            data as i32
        }
    }
}

pub mod shim;
pub use shim::*;
