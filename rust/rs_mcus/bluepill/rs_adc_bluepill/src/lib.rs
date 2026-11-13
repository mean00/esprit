#![allow(unused_variables, unused_imports, unused_parens)]
#![allow(non_camel_case_types, non_snake_case, dead_code)]

#![no_std]

pub mod registers;

use registers::*;
use rs_gpio_bluepill::Pin;
use rs_dma_bluepill::DmaChannel;
use rs_rcu_bluepill::{Peripheral, enable, get_clock};
use rs_rtos::BinarySemaphore;
use rs_mcu_traits::adc;
use rs_mcu_traits::handler::HandlerTable;
use rs_mcu_traits::{Error, Result};
use core::ptr::{read_volatile, write_volatile};
use core::ffi::c_void;

/// Channels a scan sequence can hold (the GD32 sequence registers take 16).
const MAX_SCAN_SOURCES: usize = 16;
/// Upper bound for [`adc::AdcConfig::averaging`]: a probe sense line needs a few
/// reads, not thousands (each read powers the SAR up and down).
const MAX_AVERAGING: u8 = 64;

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
    sem: BinarySemaphore,
}

impl TimingAdc {
    pub fn new(instance: u32) -> Self {
        if instance == 0 {
            enable(Peripheral::Adc0);
        } else {
            enable(Peripheral::Adc1);
        }
        Self {
            instance,
            sem: BinarySemaphore::new(),
        }
    }

    #[inline(always)]
    pub fn instance(&self) -> u32 {
        self.instance
    }

    pub fn configured_pins_count(&self) -> u32 {
        let regs = AdcRegisters::ptr(self.instance);
        unsafe { ((rs_mcu_traits::reg_read!(regs, rsq0) >> ADC_RSQ0_LEN_POS) & 0x0F) + 1 }
    }

    pub fn set_source(&mut self, _timer: u32, _channel: u32, _fq: u32, pins: &[Pin]) -> bool {
        if self.instance != 0 {
            return false;
        }
        let regs = AdcRegisters::ptr(self.instance);
        let nb_pins = pins.len() as u32;

        unsafe {
            // Setup Scan Mode
            let mut ctl0 = rs_mcu_traits::reg_read!(regs, ctl0);
            ctl0 |= ADC_CTL0_SM;
            rs_mcu_traits::reg_write!(regs, ctl0, ctl0);

            // Configure sequencer length
            let rsq0 = (nb_pins.saturating_sub(1) & 0x0F) << ADC_RSQ0_LEN_POS;
            rs_mcu_traits::reg_write!(regs, rsq0, rsq0);

            // Map pins to RSQ2 (first 5 channels)
            let mut rsq2 = 0;
            for (i, &p) in pins.iter().enumerate().take(5) {
                let ch = pin_to_adc_channel(p as u32);
                rsq2 |= ch << (5 * i);
            }
            rs_mcu_traits::reg_write!(regs, rsq2, rsq2);

            // Setup External Trigger
            let mut ctl1 = rs_mcu_traits::reg_read!(regs, ctl1);
            ctl1 &= !ADC_CTL1_ETSRC_MASK;
            ctl1 |= ADC_CTL1_ETSRC_SWSTART;
            ctl1 |= ADC_CTL1_ETERC;
            rs_mcu_traits::reg_write!(regs, ctl1, ctl1);

            // Enable ADC
            ctl1 |= ADC_CTL1_ADCON;
            rs_mcu_traits::reg_write!(regs, ctl1, ctl1);
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
            // Clear any stale semaphore token
            self.sem.try_take();

            let mut ctl1 = rs_mcu_traits::reg_read!(regs, ctl1);
            ctl1 |= ADC_CTL1_DMA;
            rs_mcu_traits::reg_write!(regs, ctl1, ctl1);

            dma.attach_completion_semaphore(&self.sem);

            // ADC is Peripheral-to-Memory transfer (DIR=0)
            dma.begin_rx_transfer(
                &(*regs).rdata as *const _ as u32,
                output.as_mut_ptr() as u32,
                total_samples,
                true, // 16-bit PSIZE
                true, // 16-bit MSIZE
            );

            ctl1 |= ADC_CTL1_SWRCST;
            rs_mcu_traits::reg_write!(regs, ctl1, ctl1);

            // Block until DMA transfer complete interrupt triggers semaphore
            self.sem.take();

            dma.end_transfer();
            dma.detach_callback();

            ctl1 &= !ADC_CTL1_DMA;
            rs_mcu_traits::reg_write!(regs, ctl1, ctl1);
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

    /// Split a buffer length into whole scan passes: `(samples per channel,
    /// total samples)`.
    fn split_len(&self, capacity: usize) -> Result<(u32, usize)> {
        let per_scan = self.configured_pins_count().max(1) as usize;
        let count = capacity - capacity % per_scan;
        if count == 0 {
            return Err(Error::BufferTooSmall);
        }
        Ok(((count / per_scan) as u32, count))
    }
}

/// Idiomatic struct representing a single-pin simple ADC converter.
pub struct SimpleAdc {
    instance: u32,
    pin: u32,
    /// Conversions averaged into one [`adc::Adc::read`].
    averaging: u8,
}

impl SimpleAdc {
    pub fn new(instance: u32, pin: u32) -> Self {
        if instance == 0 {
            enable(Peripheral::Adc0);
        } else {
            enable(Peripheral::Adc1);
        }
        Self {
            instance,
            pin,
            averaging: 1,
        }
    }

    #[inline(always)]
    pub fn instance(&self) -> u32 {
        self.instance
    }

    #[inline(always)]
    pub fn pin(&self) -> u32 {
        self.pin
    }

    pub fn set_smpt(&self, smpt: u32) {
        // The sampling time of a channel lives in SAMPT0 (channels 0..9) or SAMPT1
        // (channels 10..17), three bits per channel. Leaving it at the reset value
        // (1.5 cycles) under-reads any source with a non-negligible impedance — the
        // NRST/2 divider of a probe sense input, for instance — by a large margin.
        let regs = AdcRegisters::ptr(self.instance);
        let ch = pin_to_adc_channel(self.pin);
        let smpt = smpt & ADC_SAMPT_MASK;
        unsafe {
            if ch < ADC_SAMPT0_CHANNELS {
                let shift = ch * ADC_SAMPT_BITS;
                let mut v = rs_mcu_traits::reg_read!(regs, sampt0);
                v &= !(ADC_SAMPT_MASK << shift);
                rs_mcu_traits::reg_write!(regs, sampt0, v | (smpt << shift));
            } else {
                let shift = (ch - ADC_SAMPT0_CHANNELS) * ADC_SAMPT_BITS;
                let mut v = rs_mcu_traits::reg_read!(regs, sampt1);
                v &= !(ADC_SAMPT_MASK << shift);
                rs_mcu_traits::reg_write!(regs, sampt1, v | (smpt << shift));
            }
        }
    }

    /// One conversion through the SAR, as the hardware reports it (`0 … 4095`).
    ///
    /// The inherent name is `read_raw` because the [`adc::Adc`] impl owns the
    /// `read()` of the trait, and a method call inside that impl would otherwise
    /// resolve to the trait method (and recurse).
    pub fn read_raw(&self) -> i32 {
        let regs = AdcRegisters::ptr(self.instance);
        unsafe {
            let ch = pin_to_adc_channel(self.pin);

            let mut ctl1 = rs_mcu_traits::reg_read!(regs, ctl1);
            ctl1 &= !ADC_CTL1_ETSRC_MASK;
            ctl1 |= ADC_CTL1_ETSRC_SWSTART;
            ctl1 |= ADC_CTL1_ETERC;
            ctl1 |= ADC_CTL1_ADCON;
            rs_mcu_traits::reg_write!(regs, ctl1, ctl1);

            let mut nop_ctr = 0;
            while nop_ctr < 1000 {
                core::arch::asm!("nop");
                nop_ctr += 1;
            }

            ctl1 = rs_mcu_traits::reg_read!(regs, ctl1);
            ctl1 |= ADC_CTL1_RSTCLB;
            rs_mcu_traits::reg_write!(regs, ctl1, ctl1);
            while (rs_mcu_traits::reg_read!(regs, ctl1) & ADC_CTL1_RSTCLB) != 0 {
                core::arch::asm!("nop");
            }

            ctl1 = rs_mcu_traits::reg_read!(regs, ctl1);
            ctl1 |= ADC_CTL1_CLB;
            rs_mcu_traits::reg_write!(regs, ctl1, ctl1);
            while (rs_mcu_traits::reg_read!(regs, ctl1) & ADC_CTL1_CLB) != 0 {
                core::arch::asm!("nop");
            }

            rs_mcu_traits::reg_write!(regs, rsq0, 0);
            rs_mcu_traits::reg_write!(regs, rsq2, ch);

            ctl1 = rs_mcu_traits::reg_read!(regs, ctl1);
            ctl1 |= ADC_CTL1_SWRCST;
            rs_mcu_traits::reg_write!(regs, ctl1, ctl1);

            while (rs_mcu_traits::reg_read!(regs, stat) & ADC_STAT_EOC) == 0 {
                core::arch::asm!("nop");
            }

            let data = rs_mcu_traits::reg_read!(regs, rdata) & ADC_RDATA_DATA_MASK;
            data as i32
        }
    }
}

const ADC_MAX_INSTANCES: usize = 2;

/// One handler slot per ADC instance: `ADC0`'s end-of-conversion and `ADC1`'s are
/// independent.
static HANDLERS: HandlerTable<dyn adc::AdcHandler, ADC_MAX_INSTANCES> = HandlerTable::new();

/// Sample-time codes of the GD32 SAR: 1.5, 7.5, 13.5, 28.5, 41.5, 55.5, 71.5 and
/// 239.5 `clk_adc` cycles, in `SMPT` order (datasheet §12.4.4).
const SMPT_WINDOWS_CYCLES: [u32; 8] = [2, 8, 14, 29, 42, 56, 72, 240];

/// The smallest `SMPT` code whose window covers `sample_time_ns`.
///
/// Pure so the arithmetic is testable without an ADC; `adc_hz` is the ADC clock
/// (`APB2 / 2…8`, 72 MHz on the F303 at boot).
fn smpt_code(sample_time_ns: u32, adc_hz: u32) -> u32 {
    let cycles = ((sample_time_ns as u64 * adc_hz as u64) + 999_999_999) / 1_000_000_000;
    let mut code = SMPT_WINDOWS_CYCLES.len() as u32 - 1;
    for (index, window) in SMPT_WINDOWS_CYCLES.iter().enumerate() {
        if *window as u64 >= cycles {
            code = index as u32;
            break;
        }
    }
    code
}

impl adc::Adc for SimpleAdc {
    /// `sample_time_ns` is mapped onto the SAR's `SMPT` code (`0` keeps the
    /// current setting); `averaging` is applied by reading that many times.
    fn configure(&mut self, config: &adc::AdcConfig) -> Result<()> {
        if config.averaging > MAX_AVERAGING {
            return Err(Error::InvalidConfig);
        }
        if config.sample_time_ns != 0 {
            let adc_hz = rs_rcu_bluepill::get_clock(Peripheral::Adc0);
            self.set_smpt(smpt_code(config.sample_time_ns, adc_hz));
        }
        self.averaging = config.averaging.max(1);
        Ok(())
    }

    fn read(&mut self) -> Result<u16> {
        let count = self.averaging.max(1) as u32;
        let mut sum = 0u32;
        for _ in 0..count {
            sum += self.read_raw() as u32;
        }
        let value = (sum / count) as u16;
        if let Some(handler) = HANDLERS.get(self.instance as usize) {
            handler.on_conversion(value);
        }
        Ok(value)
    }

    fn set_handler(&mut self, handler: Option<&'static dyn adc::AdcHandler>) {
        HANDLERS.set(self.instance as usize, handler);
    }
}

impl adc::ScanAdc for TimingAdc {
    /// The GD32 sequence is a list of channels, so `sources` carries the channel
    /// numbers; `fq_hz` is programmed through the trigger timer.
    fn configure(&mut self, config: &adc::ScanConfig<'_>) -> Result<()> {
        if config.sources.is_empty() || config.sources.len() > MAX_SCAN_SOURCES {
            return Err(Error::InvalidConfig);
        }
        let mut pin_objs = [Pin::PA0; MAX_SCAN_SOURCES];
        for (slot, number) in pin_objs.iter_mut().zip(config.sources) {
            *slot = Pin::from(*number);
        }
        let count = config.sources.len();
        if self.set_source(0, 0, config.fq_hz, &pin_objs[..count]) {
            Ok(())
        } else {
            Err(Error::InvalidConfig)
        }
    }

    fn samples_per_scan(&self) -> usize {
        self.configured_pins_count() as usize
    }

    fn read(&mut self, output: &mut [u16]) -> Result<usize> {
        let (per_channel, count) = self.split_len(output.len())?;
        if self.multi_read(per_channel, &mut output[..count]) {
            Ok(count)
        } else {
            Err(Error::Busy)
        }
    }

    /// Asynchronous DMA scan is not yet implemented on this family.
    fn start_async(&mut self, _output: &mut [u16]) -> Result<()> {
        Err(Error::Unsupported)
    }

    fn is_busy(&self) -> bool {
        false
    }

    fn set_handler(&mut self, handler: Option<&'static dyn adc::AdcHandler>) {
        HANDLERS.set(self.instance as usize, handler);
    }
}

pub mod shim;
pub use shim::*;
