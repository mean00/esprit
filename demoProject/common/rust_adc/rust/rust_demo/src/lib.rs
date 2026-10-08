//! Common Rust ADC demo using high-level `rust_esprit` API.
//!
//! Follows Esprit Rust Rule 1 (No Magic Numbers), Rule 2 (Two-Tier Architecture),
//! Rule 4 (Mandatory Examples), and Rule 6 (Idiomatic Rust).

#![no_std]

use rust_esprit::{logger, logger_init};
use rust_esprit::{AdcTiming, Pin};

logger_init!();

#[cfg(any(feature = "rp2040"))]
const ADC_PIN: Pin = Pin::GPIO26;
#[cfg(not(any(feature = "rp2040")))]
const ADC_PIN: Pin = Pin::PA0;

const ADC_INSTANCE: i32 = 0;
const SAMPLES_COUNT: usize = 16;

#[unsafe(no_mangle)]
extern "C" fn user_init() {
    logger!("Starting ADC test...\n");

    let mut adc: AdcTiming<Pin> = AdcTiming::new(ADC_INSTANCE);
    let pins = [ADC_PIN];
    logger!("Configuring ADC timer trigger & pins...\n");
    let src_ok = adc.set_source(0, 0, 10_000, &pins);
    logger!("set_source ok: {}\n", if src_ok { 1 } else { 0 });

    let mut buffer = [0u16; SAMPLES_COUNT];
    logger!("Performing synchronous multi_read...\n");
    let read_ok = adc.multi_read(SAMPLES_COUNT as u32, &mut buffer);
    logger!("multi_read ok: {}, first sample: {}\n", if read_ok { 1 } else { 0 }, buffer[0]);

    logger!("--end--\n");
}
