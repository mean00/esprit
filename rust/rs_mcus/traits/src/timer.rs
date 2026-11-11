//! Timers: a periodic/single-shot timebase ([`Timer`]) and a one-shot delay
//! ([`DelayTimer`]).
//!
//! Both are driven by a family timer channel but they answer different questions:
//! `Timer` produces a waveform on a pin (the SWD clock of a bit-banged probe), a
//! `DelayTimer` fires once after `us` and is used to schedule work.

use crate::error::Result;

/// What a [`Timer`] calls on every period.
pub trait TimerHandler: Sync {
    /// One period elapsed.
    fn on_tick(&self);
}

/// A timer channel producing a periodic signal or one pulse.
pub trait Timer {
    /// Run at `hz` and return the frequency the hardware actually reaches
    /// (dividers round, and the family reports what it programmed).
    fn start_periodic(&mut self, hz: u32) -> Result<u32>;

    /// One pulse of `duration_us`; `active_high` picks the polarity.
    fn start_single_shot(&mut self, duration_us: u32, active_high: bool) -> Result<()>;

    /// Stop the channel and its interrupt.
    fn stop(&mut self);

    /// Install the handler of **this** channel.
    fn set_handler(&mut self, handler: Option<&'static dyn TimerHandler>);
}

/// What a [`DelayTimer`] calls when the delay elapsed.
pub trait DelayHandler: Sync {
    /// The one-shot delay armed with [`DelayTimer::arm_us`] expired.
    fn on_elapsed(&self);
}

/// A one-shot delay source: arm `us` microseconds, get one interrupt.
pub trait DelayTimer {
    /// Arm the delay.
    fn arm_us(&mut self, us: u32) -> Result<()>;

    /// Install the handler of **this** delay.
    fn set_handler(&mut self, handler: Option<&'static dyn DelayHandler>);

    /// Unmask the interrupt without re-arming.
    fn enable_interrupt(&mut self);

    /// Mask the interrupt.
    fn disable_interrupt(&mut self);
}
