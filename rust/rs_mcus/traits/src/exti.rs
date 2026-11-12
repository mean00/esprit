//! External interrupts: arm an edge on a pin, get a typed handler.
//!
//! This is where a pin **interrupt** lives — [`crate::gpio::Gpio`] is only the
//! pad (mode, level, direction). The controller is addressed by the family pin
//! number, so the same code works on a GD32 (EXTI line = pin number) and on an
//! RP2xxx (IO_BANK0 per-pin interrupt enable), even though the two have nothing in
//! common underneath. A family that has no separate EXTI block (an RP2xxx pin
//! interrupt *is* the GPIO interrupt) implements it by forwarding to
//! [`crate::gpio::Gpio`]'s registers.

use crate::error::Result;

/// Which level transition raises a pin interrupt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    Rising,
    Falling,
    Both,
}

/// What an external interrupt calls.
pub trait ExtiHandler: Sync {
    /// The edge on `pin` completed.
    fn on_exti(&self, pin: u32);
}

/// External-interrupt controller.
pub trait Exti {
    /// Route `pin`'s `edge` to `handler` and enable it.
    ///
    /// One handler per pin: the family keeps one slot per pin (30 pads → a
    /// [`crate::handler::HandlerTable`] of 30), so arming a second pin with the
    /// same handler is fine and arming the same pin twice replaces it.
    fn attach(
        &mut self,
        pin: u32,
        edge: Edge,
        handler: Option<&'static dyn ExtiHandler>,
    ) -> Result<()>;

    /// Enable the interrupt of an already attached pin.
    fn enable(&mut self, pin: u32) -> Result<()>;

    /// Disable the interrupt of `pin`, keeping its handler armed.
    fn disable(&mut self, pin: u32) -> Result<()>;

    /// Disable and forget the handler of `pin`.
    fn detach(&mut self, pin: u32);
}
