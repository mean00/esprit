//! RP2040 runtime constants and microsecond timebase C ABI.

pub const HAS_FPU: bool = false;

pub use rs_rp2xxx::runtime::{ln_get_us, ln_get_us64};
