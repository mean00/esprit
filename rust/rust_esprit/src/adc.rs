#![allow(dead_code)]

use crate::lnPin;
use crate::rn_timing_adc_c;

pub use rn_timing_adc_c::ln_timing_adc_c;

/// A timing‑driven multi‑channel ADC reader.
///
/// The ADC is triggered by a hardware timer and reads multiple channels
/// via DMA.
///
/// Created via `AdcTiming::new(instance)`.
/// The underlying C object is destroyed on drop.
pub struct AdcTiming {
    raw: *mut rn_timing_adc_c::ln_timing_adc_c,
}

impl AdcTiming {
    /// Create a new timing‑ADC instance for hardware `instance`.
    ///
    /// Use `new` to create the driver, then [set_source](Self::set_source)
    /// to configure the timer trigger and pin list, then [multi_read](Self::multi_read)
    /// to take samples.
    pub fn new(instance: i32) -> Self {
        let raw = unsafe { rn_timing_adc_c::ln_timing_adc_create(instance) };
        assert!(!raw.is_null(), "ln_timing_adc_create returned NULL");
        Self { raw }
    }

    /// Returns a pointer to the raw C object.  Advanced use only.
    #[inline]
    pub fn raw(&self) -> *mut rn_timing_adc_c::ln_timing_adc_c {
        self.raw
    }

    /// Configure the timer source and ADC channels.
    ///
    /// * `timer`   – timer hardware instance
    /// * `channel` – timer channel
    /// * `fq`      – sampling frequency in Hz
    /// * `pins`    – slice of `lnPin` values (e.g. `&[lnPin::PA0, lnPin::PA1]`),
    ///               one per channel to read
    pub fn set_source(&mut self, timer: i32, channel: i32, fq: i32, pins: &[lnPin]) -> bool {
        unsafe {
            // The C function expects `*const rn_timing_adc_c::lnPin` (a bindgen‑generated
            // empty enum), but our `lnPin` from `gpio` is the real platform enum.
            // Transmute the slice pointer — both are repr(i32) enums.
            rn_timing_adc_c::ln_timing_adc_set_source(
                self.raw,
                timer,
                channel,
                fq,
                pins.len() as cty::c_int,
                pins.as_ptr() as *const rn_timing_adc_c::lnPin,
            )
        }
    }

    /// Read `nb_sample_per_channel` samples from *each* configured channel.
    ///
    /// The `output` buffer must be at least `channels × nb_sample_per_channel`
    /// elements long.  Data is interleaved: samples for channel 0, then channel 1, etc.
    pub fn multi_read(&mut self, nb_sample_per_channel: i32, output: &mut [u16]) -> bool {
        assert!(
            output.len() as i32 >= nb_sample_per_channel,
            "output buffer too small"
        );
        unsafe {
            rn_timing_adc_c::ln_timing_adc_multi_read(
                self.raw,
                nb_sample_per_channel,
                output.as_mut_ptr(),
            )
        }
    }
}

impl Drop for AdcTiming {
    fn drop(&mut self) {
        unsafe {
            rn_timing_adc_c::ln_timing_adc_delete(self.raw);
        }
    }
}
