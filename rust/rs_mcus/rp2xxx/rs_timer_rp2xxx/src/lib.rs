//! RP2040 / RP2350 timers: the `rs_mcu_traits::timer` contracts, not written yet.
//!
//! esprit has no RP2xxx implementation of these peripherals, and the C++ bridge
//! (`lnTimer_c.cpp`) is deliberately *not* built for RP2040/RP2350 — see the
//! `if(NOT (USE_RP2040 OR USE_RP2350))` guard in `esprit/rust/CMakeLists.txt`.
//!
//! The crate used to export panicking `ln_timer_*` / `ln_hw_stopwatch_*` /
//! `ln_delay_timer_*` C-ABI stubs so the old `rust_esprit::{Timer, DelayTimer}`
//! wrappers had something to link against. Those wrappers are gone (M7b) and no
//! C++ on this platform references the symbols (`swindle/src/platform/ln` is not
//! built for RP2xxx), so the stubs were deleted instead of kept as traps.
//!
//! What remains is the contract implementation below: a caller gets
//! [`Error::Unsupported`](rs_mcu_traits::Error::Unsupported) and the gap is
//! visible in `rs_mcus/rp2xxx/README.md`.

#![no_std]

// ---------------------------------------------------------------------------
//  The MCU-agnostic timer contracts
// ---------------------------------------------------------------------------
//
// Neither is written yet. `Timer` needs a PWM slice (M6, `rs_multi_pulse_rp2xxx`)
// and `DelayTimer` needs the TIMER0 alarm plus its NVIC line; until then both
// answer `Error::Unsupported` instead of panicking, so a caller gets a value it
// can handle and the gap is visible in `rs_mcus/rp2xxx/README.md`.

/// The RP2xxx `Timer` implementation: not written yet.
pub struct Timer;

impl rs_mcu_traits::timer::Timer for Timer {
    fn start_periodic(&mut self, _hz: u32) -> rs_mcu_traits::Result<u32> {
        Err(rs_mcu_traits::Error::Unsupported)
    }

    fn start_single_shot(
        &mut self,
        _duration_us: u32,
        _active_high: bool,
    ) -> rs_mcu_traits::Result<()> {
        Err(rs_mcu_traits::Error::Unsupported)
    }

    fn stop(&mut self) {}

    fn set_handler(&mut self, _handler: Option<&'static dyn rs_mcu_traits::timer::TimerHandler>) {}
}

/// The RP2xxx `DelayTimer` implementation: not written yet (TIMER0 alarm).
pub struct DelayTimer;

impl rs_mcu_traits::timer::DelayTimer for DelayTimer {
    fn arm_us(&mut self, _us: u32) -> rs_mcu_traits::Result<()> {
        Err(rs_mcu_traits::Error::Unsupported)
    }

    fn set_handler(&mut self, _handler: Option<&'static dyn rs_mcu_traits::timer::DelayHandler>) {}

    fn enable_interrupt(&mut self) {}

    fn disable_interrupt(&mut self) {}
}

