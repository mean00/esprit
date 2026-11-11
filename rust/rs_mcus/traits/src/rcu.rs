//! RCU: reset and clock of the peripherals.
//!
//! The peripheral identity is an **associated type**, so this file names no
//! chip: `rs_rcu_bluepill::Peripheral` lists GD32 gates, `rs_rcu_rp2xxx::Peripheral`
//! lists RP2xxx ones, and neither leaks into the contract (the old shared enum
//! carried `UsbHsCh32v3x` next to `Spi0`).

/// Peripheral gate/clock control of a family.
pub trait Rcu {
    /// The family's own peripheral list.
    type Peripheral: Copy;

    /// Take the peripheral out of reset and enable its clock.
    fn enable(peripheral: Self::Peripheral);

    /// Put the peripheral back in reset (it loses its configuration).
    fn disable(peripheral: Self::Peripheral);

    /// Assert then deassert the reset, leaving the peripheral configured but idle.
    fn reset(peripheral: Self::Peripheral);

    /// Frequency of the clock the peripheral runs from, in Hz. A peripheral that
    /// has no separate gate reports the bus clock.
    fn clock_hz(peripheral: Self::Peripheral) -> u32;
}
