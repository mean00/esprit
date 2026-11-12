#![no_std]
#![allow(dead_code)]

pub mod ffi;
pub use ffi::*;

use rs_mcu_traits::adc::{self, AdcHandler};
use rs_mcu_traits::handler::HandlerTable;

/// One handler slot: the wrapped targets expose a single timing ADC.
const ADC_INSTANCES: usize = 2;

static HANDLERS: HandlerTable<dyn AdcHandler, ADC_INSTANCES> = HandlerTable::new();

/// How many sources a scan can ask for (the C sequence registers hold 16).
const MAX_SOURCES: usize = 16;
/// Upper bound for [`adc::AdcConfig::averaging`].
const MAX_AVERAGING: u8 = 64;

pub struct SimpleAdcWrapped {
    raw: *mut cty::c_void,
    /// Conversions averaged into one [`adc::Adc::read`].
    averaging: u8,
}

impl SimpleAdcWrapped {
    pub fn new(instance: u32, pin: u32) -> Self {
        let raw = unsafe { ln_simple_adc_create(instance, pin) };
        assert!(!raw.is_null(), "ln_simple_adc_create returned NULL");
        Self { raw, averaging: 1 }
    }

    pub fn set_smpt(&mut self, smpt: u32) {
        unsafe {
            ln_simple_adc_set_smpt(self.raw, smpt);
        }
    }
}

impl Drop for SimpleAdcWrapped {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe {
                ln_simple_adc_destroy(self.raw);
            }
        }
    }
}

impl adc::Adc for SimpleAdcWrapped {
    /// `sample_time_ns` is handed to the C driver as its `SMPT` code (the wrapped
    /// targets define it in cycles, not nanoseconds); `averaging` is applied by
    /// reading that many times.
    fn configure(&mut self, config: &adc::AdcConfig) -> rs_mcu_traits::Result<()> {
        if config.averaging > MAX_AVERAGING {
            return Err(rs_mcu_traits::Error::InvalidConfig);
        }
        if config.sample_time_ns != 0 {
            self.set_smpt(config.sample_time_ns);
        }
        self.averaging = config.averaging.max(1);
        Ok(())
    }

    fn read(&mut self) -> rs_mcu_traits::Result<u16> {
        let count = self.averaging.max(1) as u32;
        let mut sum = 0u32;
        for _ in 0..count {
            sum += unsafe { ln_simple_adc_read(self.raw) } as u32;
        }
        Ok((sum / count) as u16)
    }

    fn set_handler(&mut self, _handler: Option<&'static dyn AdcHandler>) {}
}

pub struct TimingAdcWrapped {
    raw: *mut ln_timing_adc_c,
}

impl TimingAdcWrapped {
    pub fn new(instance: i32) -> Self {
        let raw = unsafe { ln_timing_adc_create(instance as cty::c_int) };
        assert!(!raw.is_null(), "ln_timing_adc_create returned NULL");
        Self { raw }
    }

    pub fn raw(&self) -> *mut ln_timing_adc_c {
        self.raw
    }
}

impl Drop for TimingAdcWrapped {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe {
                ln_timing_adc_delete(self.raw);
            }
        }
    }
}

impl adc::ScanAdc for TimingAdcWrapped {
    /// `sources` carries pin numbers, which is what the C driver's sequence takes.
    fn configure(&mut self, config: &adc::ScanConfig<'_>) -> rs_mcu_traits::Result<()> {
        if config.sources.is_empty() || config.sources.len() > MAX_SOURCES {
            return Err(rs_mcu_traits::Error::InvalidConfig);
        }
        let mut pins_buf = [0 as lnPin; MAX_SOURCES];
        for (slot, number) in pins_buf.iter_mut().zip(config.sources) {
            *slot = *number as lnPin;
        }
        let count = config.sources.len();
        let ok = unsafe {
            ln_timing_adc_set_source(
                self.raw,
                0,
                0,
                config.fq_hz as cty::c_uint,
                count as cty::c_uint,
                pins_buf.as_ptr(),
            )
        };
        if ok {
            Ok(())
        } else {
            Err(rs_mcu_traits::Error::InvalidConfig)
        }
    }

    fn samples_per_scan(&self) -> usize {
        // The C driver exposes no accessor: the last configured sequence length is
        // what `multi_read` multiplies, so the caller's buffer length is the only
        // thing this backend can use. One source is the safe default.
        1
    }

    fn read(&mut self, output: &mut [u16]) -> rs_mcu_traits::Result<usize> {
        let per_scan = self.samples_per_scan().max(1);
        let count = output.len() - output.len() % per_scan;
        if count == 0 {
            return Err(rs_mcu_traits::Error::BufferTooSmall);
        }
        let per_channel = (count / per_scan) as u32;
        let ok = unsafe {
            ln_timing_adc_multi_read(
                self.raw,
                per_channel as cty::c_uint,
                output.as_mut_ptr(),
            )
        };
        if ok {
            Ok(count)
        } else {
            Err(rs_mcu_traits::Error::Busy)
        }
    }

    /// The wrapped C ABI has its own asynchronous form, but it takes a C callback
    /// and a cookie; this implementation runs the blocking read and then the
    /// handler, which keeps the observable contract of the trait.
    fn start_async(&mut self, output: &mut [u16]) -> rs_mcu_traits::Result<()> {
        let count = self.read(output)?;
        if let Some(handler) = HANDLERS.get(0) {
            handler.on_scan_complete(&output[..count]);
        }
        Ok(())
    }

    fn is_busy(&self) -> bool {
        false
    }

    fn set_handler(&mut self, handler: Option<&'static dyn AdcHandler>) {
        HANDLERS.set(0, handler);
    }
}
