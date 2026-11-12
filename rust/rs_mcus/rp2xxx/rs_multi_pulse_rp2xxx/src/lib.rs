//! RP2040 / RP2350 PWM: the `rs_mcu_traits::pwm::Pwm` contract, not written yet.
//!
//! esprit has no RP2xxx implementation of this peripheral, and the C++ bridge
//! (`lnMultiPulse_c.cpp`) is deliberately *not* built for RP2040/RP2350 — see the
//! `if(NOT (USE_RP2040 OR USE_RP2350))` guard in `esprit/rust/CMakeLists.txt`.
//!
//! The crate used to export panicking `ln_multi_pulse_*` C-ABI stubs for the old
//! `rust_esprit::MultiPulse` wrapper. That wrapper is gone (M7b) and nothing
//! references the symbols on this platform, so the stubs were deleted.
//!
//! What remains is the contract implementation below: a caller gets
//! [`Error::Unsupported`](rs_mcu_traits::Error::Unsupported) and the gap is
//! visible in `rs_mcus/rp2xxx/README.md`.

#![no_std]

// ---------------------------------------------------------------------------
//  The MCU-agnostic PWM contract
// ---------------------------------------------------------------------------

/// The RP2xxx `Pwm` implementation.
///
/// A PWM slice could drive it (`PWM_BASE`, `CH_TOP`/`CC`, FUNCSEL 4), but the
/// driver is not written: every method answers
/// [`Error::Unsupported`](rs_mcu_traits::Error::Unsupported) so a caller gets a
/// value it can handle instead of a panic, and the gap is tracked in
/// `rs_mcus/rp2xxx/README.md`.
pub struct Pwm;

impl rs_mcu_traits::pwm::Pwm for Pwm {
    fn configure(
        &mut self,
        _channel: u8,
        _config: &rs_mcu_traits::pwm::PwmConfig,
    ) -> rs_mcu_traits::Result<()> {
        Err(rs_mcu_traits::Error::Unsupported)
    }

    fn start(&mut self, _channel: u8) -> rs_mcu_traits::Result<()> {
        Err(rs_mcu_traits::Error::Unsupported)
    }

    fn stop(&mut self, _channel: u8) -> rs_mcu_traits::Result<()> {
        Ok(())
    }

    fn pulse_us(&mut self, _us: u32, _active_high: bool) -> rs_mcu_traits::Result<()> {
        Err(rs_mcu_traits::Error::Unsupported)
    }

    fn set_handler(&mut self, _handler: Option<&'static dyn rs_mcu_traits::pwm::PwmHandler>) {}
}

