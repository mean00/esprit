//! Shared microsecond timebase C ABI for RP2040 / RP2350.

unsafe extern "C" {
    fn time_us_64() -> u64;
}

/// C++ `uint64_t lnGetUs64()`, replacing `rp_rcu.cpp` in `LN_ENABLE_RUST` builds.
#[unsafe(export_name = "_Z9lnGetUs64v")]
pub extern "C" fn ln_get_us64() -> u64 {
    // SAFETY: pico-sdk `hardware_timer/timer.c` provides `time_us_64()`.
    unsafe { time_us_64() }
}

/// C++ `uint32_t lnGetUs()`, replacing `rp_rcu.cpp` in `LN_ENABLE_RUST` builds.
#[unsafe(export_name = "_Z7lnGetUsv")]
pub extern "C" fn ln_get_us() -> u32 {
    ln_get_us64() as u32
}
