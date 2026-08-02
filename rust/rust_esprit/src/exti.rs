#![allow(dead_code)]

use crate::gpio::lnPin;
use crate::rn_exti_c;

/// Edge trigger configuration for external interrupts.
#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Edge {
    None = 0,
    Rising = 1,
    Falling = 2,
    Both = 3,
}

/// Convert from the platform's `Edge` to the C-bindgen `lnEdge`.
impl From<Edge> for rn_exti_c::lnEdge {
    fn from(e: Edge) -> Self {
        match e {
            Edge::None => rn_exti_c::lnEdge::LN_EDGE_NONE,
            Edge::Rising => rn_exti_c::lnEdge::LN_EDGE_RISING,
            Edge::Falling => rn_exti_c::lnEdge::LN_EDGE_FALLING,
            Edge::Both => rn_exti_c::lnEdge::LN_EDGE_BOTH,
        }
    }
}

/// A callback that gets invoked on an external interrupt.
///
/// For stateless handlers, implement `PinCallback` on a unit struct.
/// For handlers that need mutable state, implement on whichever
/// type you like—the trampoline will cast your cookie back to `&mut T`.
pub trait PinCallback {
    fn on_interrupt(&mut self, pin: lnPin);
}

// ---------- trampoline ----------

/// C-ABI trampoline invoked by the C shim.  `cookie` points to the `&T`
/// handler registered by [`attach_interrupt`].
extern "C" fn generic_trampoline<T: PinCallback>(pin: lnPin, cookie: *mut cty::c_void) {
    let handler = unsafe { &mut *(cookie as *mut T) };
    handler.on_interrupt(pin);
}

// ---------- public API ----------

/// Attach an external interrupt to a pin.
///
/// `handler` is borrowed until `detach_interrupt` is called.
/// The C callback fires with the cookie pointing to `handler`.
pub fn attach_interrupt<T: PinCallback>(pin: lnPin, edge: Edge, handler: &T) {
    unsafe {
        rn_exti_c::lnExtiAttachInterrupt_c(
            pin,
            edge.into(),
            Some(generic_trampoline::<T>),
            handler as *const T as *mut cty::c_void,
        );
    }
}

/// Detach an external interrupt from a pin.
pub fn detach_interrupt(pin: lnPin) {
    unsafe {
        rn_exti_c::lnExtiDetachInterrupt_c(pin);
    }
}

/// Enable the external interrupt for a pin (after attaching).
pub fn enable_interrupt(pin: lnPin) {
    unsafe {
        rn_exti_c::lnExtiEnableInterrupt_c(pin);
    }
}

/// Disable the external interrupt for a pin.
pub fn disable_interrupt(pin: lnPin) {
    unsafe {
        rn_exti_c::lnExtiDisableInterrupt_c(pin);
    }
}

