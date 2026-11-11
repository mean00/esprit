//! PWM: a repetitive waveform on a pin, and the single-pulse special case
//! (what esprit calls "multi pulse": a fixed-width pulse for a bit-banged SWD or
//! SDI clock).

use crate::error::Result;

/// Waveform of one channel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PwmConfig {
    /// Period, in Hz.
    pub frequency_hz: u32,
    /// Duty cycle in tenths of a percent: `0` = always low, `1000` = always high.
    pub duty_permille: u16,
}

impl Default for PwmConfig {
    fn default() -> Self {
        Self {
            frequency_hz: 1_000_000,
            duty_permille: 500,
        }
    }
}

/// What a PWM interrupt calls.
pub trait PwmHandler: Sync {
    /// The single pulse armed with [`Pwm::pulse_us`] finished.
    fn on_pulse_complete(&self);
}

/// One PWM channel of a family timer/slice.
pub trait Pwm {
    /// Program the waveform and start it.
    fn configure(&mut self, channel: u8, config: &PwmConfig) -> Result<()>;

    /// Start a configured channel.
    fn start(&mut self, channel: u8) -> Result<()>;

    /// Stop a channel, leaving the pin in its idle level.
    fn stop(&mut self, channel: u8) -> Result<()>;

    /// Emit one pulse of `us` microseconds (`active_high` picks the polarity) and
    /// call [`PwmHandler::on_pulse_complete`] when it ends.
    fn pulse_us(&mut self, us: u32, active_high: bool) -> Result<()>;

    /// Install the handler of **this** instance.
    fn set_handler(&mut self, handler: Option<&'static dyn PwmHandler>);
}
