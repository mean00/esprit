//! RP2350 runtime constants and microsecond timebase C ABI.

pub const HAS_FPU: bool = true;

pub use rs_rp2xxx::runtime::{ln_get_us, ln_get_us64};
