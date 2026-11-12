//! ADC: one conversion ([`Adc`]) and a paced multi-source scan ([`ScanAdc`]).
//!
//! The two are separate traits because they are separate capabilities: a family
//! may have a working single-shot ADC and no scan engine at all. A scan is
//! *paced by the ADC itself* (its own clock divider), so the contract never
//! mentions a trigger timer — a parameter only one family can honour.

use crate::error::Result;

/// Sample time and averaging of a conversion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AdcConfig {
    /// Acquisition window in nanoseconds. `0` keeps the family default; a value
    /// the silicon cannot reach is clamped and reported through the
    /// implementation's documentation (never an error, the reading stays valid).
    pub sample_time_ns: u32,
    /// Conversions averaged into one `read`. `0` and `1` both mean "one
    /// conversion".
    pub averaging: u8,
}

impl Default for AdcConfig {
    fn default() -> Self {
        Self {
            sample_time_ns: 0,
            averaging: 1,
        }
    }
}

/// What an ADC interrupt calls.
pub trait AdcHandler: Sync {
    /// One conversion completed (single-shot or scan).
    fn on_conversion(&self, _sample: u16) {}
    /// A scan wrote all its samples; the slice is the caller's buffer, in the
    /// order the sources were configured.
    fn on_scan_complete(&self, _samples: &[u16]) {}
}

/// A single-channel, one-conversion-at-a-time ADC.
pub trait Adc {
    /// Apply [`AdcConfig`]; `Err(InvalidConfig)` when the averaging factor is
    /// impossible for the family.
    fn configure(&mut self, config: &AdcConfig) -> Result<()>;

    /// Convert once and return the raw, right-aligned sample. `Err(Timeout)` when
    /// the converter never reported ready (a broken clock tree, not a slow input).
    fn read(&mut self) -> Result<u16>;

    /// Install the handler of **this** instance.
    fn set_handler(&mut self, handler: Option<&'static dyn AdcHandler>);
}

/// What a scan samples: the source ids (family-defined: an ADC channel, a pin
/// number, a temperature sensor) and how fast.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScanConfig<'a> {
    /// One entry per sample of a scan pass, **in the order the samples must
    /// appear in the output buffer**. A family that can only walk its sources in
    /// ascending order returns [`Error::InvalidConfig`](crate::Error::InvalidConfig)
    /// for an unsorted list instead of silently reordering the data.
    pub sources: &'a [u32],
    /// Conversions per second per source; `0` means "as fast as the silicon goes".
    pub fq_hz: u32,
}

/// A paced multi-source ADC scan, optionally DMA-driven.
pub trait ScanAdc {
    /// Program the sources and the rate.
    fn configure(&mut self, config: &ScanConfig<'_>) -> Result<()>;

    /// Samples one scan pass produces (`sources.len()` once configured).
    fn samples_per_scan(&self) -> usize;

    /// Run one full read: blocking, `output` is filled with whole passes (its
    /// length divided by [`samples_per_scan`](Self::samples_per_scan) passes) and
    /// the returned count is how many samples were stored.
    fn read(&mut self, output: &mut [u16]) -> Result<usize>;

    /// Start one full read and return immediately; when the last sample has
    /// landed, [`AdcHandler::on_scan_complete`] runs **in the completion
    /// interrupt**.
    ///
    /// `output` belongs to the transfer until then: it must not be read, written
    /// or dropped in between, and the handler must not block.
    fn start_async(&mut self, output: &mut [u16]) -> Result<()>;

    /// `true` while [`start_async`](Self::start_async) has not completed.
    fn is_busy(&self) -> bool;

    /// Install the handler of **this** instance.
    fn set_handler(&mut self, handler: Option<&'static dyn AdcHandler>);
}

/// What an asynchronous read writes into.
///
/// Implemented by the type the caller hands to the completion, so the buffer and
/// the context stay one value — no second, aliasing parameter.
pub trait AdcBuffer {
    /// A mutable slice of at least `nb` samples for the DMA to fill.
    fn adc_buffer(&mut self, nb: usize) -> &mut [u16];
}
