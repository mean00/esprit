//! Common Rust ADC demo using the MCU-agnostic ADC super traits.
//!
//! `rs_mcu_traits::adc::ScanAdc` (re-exported as `rust_esprit::HalScanAdc`) is the
//! N-sources × M-samples contract, DMA driven; the port itself is the family driver
//! (`rust_esprit::bluepill::adc::TimingAdc`, i.e. `rs_adc_bluepill`).
//!
//! Follows Esprit Rust Rule 1 (No Magic Numbers), Rule 2 (Two-Tier Architecture),
//! Rule 4 (Mandatory Examples), and Rule 6 (Idiomatic Rust).

#![no_std]

use rust_esprit::{Pin, ScanAdc, ScanAdcDriver, ScanConfig, logger, logger_init};

logger_init!();

#[cfg(any(feature = "rp2040"))]
const ADC_PIN: Pin = Pin::GPIO26;
#[cfg(not(any(feature = "rp2040")))]
const ADC_PIN: Pin = Pin::PA0;

const ADC_INSTANCE: u32 = 0;
const SAMPLES_COUNT: usize = 16;
const SAMPLE_RATE_HZ: u32 = 10_000;

#[unsafe(no_mangle)]
extern "C" fn user_init() {
    logger!("Starting ADC test...\n");

    let mut adc = ScanAdcDriver::new(ADC_INSTANCE);
    // The scan is configured with pin numbers, in ascending channel order.
    let sources = [ADC_PIN as u32];
    logger!("Configuring ADC timer trigger & pins...\n");
    let src_ok = ScanAdc::configure(
        &mut adc,
        &ScanConfig {
            sources: &sources,
            fq_hz: SAMPLE_RATE_HZ,
        },
    )
    .is_ok();
    logger!("configure ok: {}\n", if src_ok { 1 } else { 0 });

    let mut buffer = [0u16; SAMPLES_COUNT];
    logger!("Performing synchronous multimple read...\n");
    let filled = ScanAdc::read(&mut adc, &mut buffer).unwrap_or(0);
    logger!(
        "read ok: {}, first sample: {}\n",
        if filled > 0 { 1 } else { 0 },
        buffer[0]
    );

    logger!("--end--\n");
}

