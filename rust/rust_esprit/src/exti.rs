#![allow(dead_code)]

use crate::gpio::{self, lnPin};
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

// ---------- low-level FFI wrappers ----------
//
// NOTE: We define our own FFI extern blocks here instead of using rn_exti_c
// because lnExti.h only forward-declares `enum lnPin : int;` without the actual
// enum values. Bindgen sees this opaque forward declaration and generates a
// dummy enum with only `__bindgen_cannot_repr_c_on_empty_enum = 0`, which
// corrupts all pin values. By using u32 directly (matching the C ABI of the
// forward-declared enum on ARM), we avoid this bindgen bug entirely.
//
// The trampoline transmutes the u32 to gpio::lnPin which has the proper variants.

type ExtiCallback = unsafe extern "C" fn(pin: u32, cookie: *mut cty::c_void);

unsafe extern "C" {
    #[link_name = "\u{1}_Z21lnExtiAttachInterrupt5lnPin6lnEdgePFvS_PvES1_"]
    fn lnExtiAttachInterrupt(
        pin: u32,
        edge: rn_exti_c::lnEdge,
        cb: Option<ExtiCallback>,
        cookie: *const cty::c_void,
    );
}
unsafe extern "C" {
    #[link_name = "\u{1}_Z21lnExtiDetachInterrupt5lnPin"]
    fn lnExtiDetachInterrupt(pin: u32);
}
unsafe extern "C" {
    #[link_name = "\u{1}_Z21lnExtiEnableInterrupt5lnPin"]
    fn lnExtiEnableInterrupt(pin: u32);
}
unsafe extern "C" {
    #[link_name = "\u{1}_Z22lnExtiDisableInterrupt5lnPin"]
    fn lnExtiDisableInterrupt(pin: u32);
}

// ---------- trampoline ----------

extern "C" fn generic_trampoline<T: PinCallback>(pin: u32, cookie: *mut cty::c_void) {
    let handler = unsafe { &mut *(cookie as *mut T) };
    // The C++ side passes lnPin as an int (it's a C enum with underlying type int).
    // Transmute the u32 to our canonical gpio::lnPin enum.
    let canonical: lnPin = unsafe { core::mem::transmute(pin) };
    handler.on_interrupt(canonical);
}

// ---------- public API ----------

/// Attach an external interrupt to a pin.
///
/// `handler` is borrowed until `detach_interrupt` is called.
/// The C callback fires with the cookie pointing to `handler`.
pub fn attach_interrupt<T: PinCallback>(pin: lnPin, edge: Edge, handler: &T) {
    unsafe {
        lnExtiAttachInterrupt(
            pin as u32,
            edge.into(),
            Some(generic_trampoline::<T>),
            handler as *const T as *const cty::c_void,
        );
    }
}

/// Detach an external interrupt from a pin.
pub fn detach_interrupt(pin: lnPin) {
    unsafe {
        lnExtiDetachInterrupt(pin as u32);
    }
}

/// Enable the external interrupt for a pin (after attaching).
pub fn enable_interrupt(pin: lnPin) {
    unsafe {
        lnExtiEnableInterrupt(pin as u32);
    }
}

/// Disable the external interrupt for a pin.
pub fn disable_interrupt(pin: lnPin) {
    unsafe {
        lnExtiDisableInterrupt(pin as u32);
    }
}

