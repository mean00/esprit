//! Canonical `lnPin` type shared across all FFI binding modules.
//!
//! Bindgen generates a platform-specific `lnPin` enum in each GPIO binding
//! module (`rn_gpio_bp_c`, `rn_gpio_esp32_c`, `rn_gpio_rp2040_c`).  Other
//! FFI modules (timer, exti, timing_adc) only see a forward declaration and
//! would generate a stub `lnPin` that is a *different Rust type*.
//!
//! This module re‑exports the one true `lnPin` for the current target so
//! that all consumers use the same type.  On RP2040/RP2350 that type is not a
//! binding at all: it is `rs_gpio_rp2xxx::Pin`, which is also `rust_esprit::Pin`
//! and the type the `GpioPin` implementation is written for
//! (`rn_gpio_rp2040_c` re-exports it), so a pin has exactly one type in the whole
//! stack.

#[cfg(not(any(feature = "rp2040", feature = "esp32")))]
pub use rs_gpio_bluepill::lnPin;

#[cfg(feature = "esp32")]
pub use crate::rn_gpio_esp32_c::lnPin;

#[cfg(feature = "rp2040")]
pub use crate::rn_gpio_rp2040_c::lnPin;