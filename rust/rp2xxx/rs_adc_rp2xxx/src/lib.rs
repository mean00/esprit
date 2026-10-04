//! RP2040 / RP2350 ADC backend for `rust_esprit`.
//!
//! There are two very different situations behind this single crate, hence two
//! different behaviours.
//!
//! ## Simple (single-shot) ADC — delegates to the C++ driver
//!
//! esprit *does* provide an RP2xxx implementation for this one: the C++
//! `lnSimpleADC` class (`mcus/arm_rp2040/include/lnADC.h`, implemented in
//! `src/ln_rp_adc.cpp`; the same driver swindle uses in
//! `swindle/src/platform/rp2040/bmp_adc_rp2040.cpp`).
//!
//! The `ln_simple_adc_*` entry points below therefore call into it, through the
//! `extern "C"` bridge `rust_esprit/c_interface/ln_rp_simple_adc_c.cpp`, which
//! `esprit/rust/CMakeLists.txt` builds for RP2xxx only.
//!
//! ## Timing (DMA) ADC — unsupported, panics
//!
//! esprit has **no** RP2xxx implementation of `lnTimingAdc`: that class only
//! exists for the Bluepill targets (`mcus/common_bluepill/lnADC_timing.cpp`),
//! and its C++ bridge `lnTiming_adc_c.cpp` is deliberately *not* built for
//! RP2040/RP2350 (see the `if(NOT (USE_RP2040 OR USE_RP2350))` guard in
//! `esprit/rust/CMakeLists.txt`).
//!
//! `ln_timing_adc_*` is therefore still `unimplemented!(...)`: reaching it
//! panics with a clear message instead of producing an obscure undefined
//! reference at link time.
//!
//! In both cases the exported names, signatures and handle types match the
//! other backends (`rs_adc_bluepill` on GD32/CH32, `c_api::rn_timing_adc_c` /
//! `c_api::rn_simple_adc_c` on ESP32) so that `rust_esprit` can select a backend
//! per platform — see `esprit/rust/rp2xxx/README.md`.


#![no_std]
#![allow(unused_variables, non_camel_case_types, non_snake_case, dead_code)]

use core::ffi::c_void;

/// Opaque timing-ADC handle — same shape as the C++ `ln_timing_adc_c` and as
/// `rs_adc_bluepill::ln_timing_adc_c`.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct ln_timing_adc_c {
    pub dummy: *mut c_void,
}

/// Completion callback for [`ln_timing_adc_async_read`].
pub type ln_timing_adc_async_callback_t =
    ::core::option::Option<unsafe extern "C" fn(arg1: *mut c_void)>;

/// Create a timing ADC for the given ADC peripheral instance.
#[unsafe(no_mangle)]
pub extern "C" fn ln_timing_adc_create(instance: i32) -> *mut ln_timing_adc_c {
    unimplemented!("ln_timing_adc_create is not supported on RP2040 / RP2350")
}

/// Destroy a timing ADC created by [`ln_timing_adc_create`].
#[unsafe(no_mangle)]
pub extern "C" fn ln_timing_adc_delete(in_: *mut ln_timing_adc_c) -> bool {
    unimplemented!("ln_timing_adc_delete is not supported on RP2040 / RP2350")
}

/// Configure the ADC source: timer, channel, sampling frequency and pins.
///
/// Generic over the pin element type: the RP2xxx `lnPin` enum is generated
/// inside `rust_esprit`, so a sibling crate cannot name it (circular
/// dependency).  The pointer is never dereferenced.
pub fn ln_timing_adc_set_source<T>(
    instance: *mut ln_timing_adc_c,
    timer: u32,
    channel: u32,
    fq: u32,
    nbPins: u32,
    pin: *const T,
) -> bool {
    unimplemented!("ln_timing_adc_set_source is not supported on RP2040 / RP2350")
}

/// Synchronous multi-sample read into `output`.
#[unsafe(no_mangle)]
pub extern "C" fn ln_timing_adc_multi_read(
    instance: *mut ln_timing_adc_c,
    nbSamplePerChannel: u32,
    output: *mut u16,
) -> bool {
    unimplemented!("ln_timing_adc_multi_read is not supported on RP2040 / RP2350")
}

/// Asynchronous multi-sample read; `cb` is invoked when the DMA transfer ends.
#[unsafe(no_mangle)]
pub extern "C" fn ln_timing_adc_async_read(
    instance: *mut ln_timing_adc_c,
    nbSamplePerChannel: u32,
    output: *mut u16,
    cb: ln_timing_adc_async_callback_t,
    ctx: *mut c_void,
) -> bool {
    unimplemented!("ln_timing_adc_async_read is not supported on RP2040 / RP2350")
}

// ---------------------------------------------------------------------------
//  Simple (single-shot) ADC — delegates to the C++ `lnSimpleADC` driver
//  (mcus/arm_rp2xxx/src/ln_rp_adc.cpp) through the RP2xxx-only C bridge.
// ---------------------------------------------------------------------------

unsafe extern "C" {
    /// `new lnSimpleADC(instance, pin)` — see
    /// `rust_esprit/c_interface/ln_rp_simple_adc_c.cpp`.
    fn ln_rp_simple_adc_create(instance: i32, pin: u32) -> *mut c_void;
    /// `delete (lnSimpleADC *)adc`.
    fn ln_rp_simple_adc_destroy(adc: *mut c_void);
    /// `((lnSimpleADC *)adc)->simpleRead(averaging)`.
    fn ln_rp_simple_adc_read(adc: *mut c_void, averaging: u32) -> u32;
}

/// Number of conversions averaged by [`ln_simple_adc_read`].
///
/// `SimpleAdc::read()` takes no averaging parameter, so one conversion per call
/// keeps it consistent with the Bluepill Rust backend.
const SIMPLE_ADC_AVERAGING: u32 = 1;

/// Create a simple ADC on the given instance/pin (drives the C++ `lnSimpleADC`).
///
/// `pin` is an `lnPin` value: RP2xxx only has ADC inputs on GPIO26..GPIO29 and
/// the driver asserts on anything else.
#[unsafe(no_mangle)]
pub extern "C" fn ln_simple_adc_create(instance: u32, pin: u32) -> *mut c_void {
    unsafe { ln_rp_simple_adc_create(instance as i32, pin) }
}

/// Set the sample time for subsequent reads.
///
/// The RP2xxx `lnSimpleADC` has no sample-time knob (unlike the GD32 ADC
/// `SMPT` field), so this is a no-op — exactly as in the Bluepill Rust backend.
#[unsafe(no_mangle)]
pub extern "C" fn ln_simple_adc_set_smpt(_adc: *mut c_void, _smpt: u32) {}

/// Perform a conversion, through the C++ driver.
///
/// Returns 0 for a null handle (`lnSimpleADC::simpleRead()` would fault).
#[unsafe(no_mangle)]
pub extern "C" fn ln_simple_adc_read(adc: *mut c_void) -> i32 {
    unsafe { ln_rp_simple_adc_read(adc, SIMPLE_ADC_AVERAGING) as i32 }
}

/// Destroy a simple ADC created by [`ln_simple_adc_create`].
#[unsafe(no_mangle)]
pub extern "C" fn ln_simple_adc_destroy(adc: *mut c_void) {
    unsafe { ln_rp_simple_adc_destroy(adc) }
}
