//! Safe Rust wrapper around the simple (single-shot) ADC backend.
//!
//! The backend is selected by the platform features (see
//! `esprit/rust/rs_mcus/rp2xxx/README.md`): `rs_adc_bluepill` on GD32/CH32,
//! `rs_adc_rp2xxx` on RP2040/RP2350, `rs_adc_wrapped` (C API) elsewhere.
//!
//! On the backends that have a Rust driver of their own (`bluepill`, `rp2040`)
//! this module simply re-exports it: there is no C ABI hop and no opaque handle.
//! That matters because the GD32/CH32 C shim encoded `(instance, pin)` *in* the
//! pointer (`instance | pin << 16`), so pin 0 — PA0 — produced a NULL handle and
//! tripped the null assertion. Only the C-API backends (`esp32`, `wrapped`) still
//! need the handle wrapper, because all they have is the C ABI.

use core::ffi::c_void;

cfg_if::cfg_if! {
    if #[cfg(feature = "rp2040")] {
        /// Simple ADC driver of the selected backend (RP2040/RP2350).
        pub use rs_adc_rp2xxx::SimpleAdc;
    } else if #[cfg(any(feature = "esp32", feature = "wrapped"))] {
        /// Simple ADC driver over the C API (no Rust backend available).
        pub struct SimpleAdc {
            handle: *mut c_void,
        }

        impl SimpleAdc {
            pub fn new(instance: u32, pin: u32) -> Self {
                let handle = unsafe { crate::rn_simple_adc_c::ln_simple_adc_create(instance, pin) };
                assert!(!handle.is_null());
                Self { handle }
            }

            pub fn set_smpt(&self, smpt: u32) {
                unsafe { crate::rn_simple_adc_c::ln_simple_adc_set_smpt(self.handle, smpt) }
            }

            pub fn read(&self) -> i32 {
                unsafe { crate::rn_simple_adc_c::ln_simple_adc_read(self.handle) }
            }
        }

        impl Drop for SimpleAdc {
            fn drop(&mut self) {
                unsafe { crate::rn_simple_adc_c::ln_simple_adc_destroy(self.handle) }
            }
        }

        impl rs_esprit::adc::SimpleAdc for SimpleAdc {
            fn read(&self) -> i32 {
                self.read()
            }

            fn set_handler(&mut self, _handler: Option<&'static dyn rs_esprit::adc::AdcHandler>) {}
        }
    } else {
        /// Simple ADC driver of the selected backend (GD32/CH32).
        pub use rs_adc_bluepill::SimpleAdc;
    }
}

