#![allow(dead_code)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
//use crate::rn_exti_c as exti;
use crate::rn_gpio::rnpin2lnpin;
use crate::rn_gpio::{lnPin, rnPin};

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum rnEdge {
    LN_EDGE_NONE = 0,
    LN_EDGE_RISING = 1,
    LN_EDGE_FALLING = 2,
    LN_EDGE_BOTH = 3,
}
pub type lnExtiCallback =
    ::core::option::Option<unsafe extern "C" fn(pin: rnPin, cookie: *const cty::c_void)>;
unsafe extern "C" {
    #[link_name = "\u{1}_Z21lnExtiAttachInterrupt5lnPin6lnEdgePFvS_PvES1_"]
    pub fn lnExtiAttachInterrupt(
        pin: lnPin,
        edge: rnEdge,
        cb: lnExtiCallback,
        cookie: *const cty::c_void,
    );
}
unsafe extern "C" {
    #[link_name = "\u{1}_Z21lnExtiDetachInterrupt5lnPin"]
    pub fn lnExtiDetachInterrupt(pin: lnPin);
}
unsafe extern "C" {
    #[link_name = "\u{1}_Z21lnExtiEnableInterrupt5lnPin"]
    pub fn lnExtiEnableInterrupt(pin: lnPin);
}
unsafe extern "C" {
    #[link_name = "\u{1}_Z22lnExtiDisableInterrupt5lnPin"]
    pub fn lnExtiDisableInterrupt(pin: lnPin);
}

//
//
//
pub fn attach_interrupt(pin: rnPin, edge: rnEdge, cb: lnExtiCallback, cookie: *mut cty::c_void) {
    unsafe {
        lnExtiAttachInterrupt(rnpin2lnpin(pin), edge, cb, cookie);
    }
}
//
//
//
pub fn detach_interrupt(pin: rnPin) {
    unsafe {
        lnExtiDetachInterrupt(rnpin2lnpin(pin));
    }
}
//
//
//
pub fn enable_interrupt(pin: rnPin) {
    unsafe {
        lnExtiEnableInterrupt(rnpin2lnpin(pin));
    }
}
//
//
//
pub fn disable_interrupt(pin: rnPin) {
    unsafe {
        lnExtiDisableInterrupt(rnpin2lnpin(pin));
    }
}
/*
 * this is a more idomatic way...
 */
pub trait PinCallback {
    fn on_interrupt(&mut self, pin: rnPin);
}
//#[unsafe(no_mangle)]
extern "C" fn generic_trampoline<T: PinCallback>(pin: rnPin, cookie: *const cty::c_void) {
    let handler = unsafe { &mut *(cookie as *mut T) };
    handler.on_interrupt(pin);
}
pub fn attach_interrupt_typed<T: PinCallback>(pin: rnPin, edge: rnEdge, handler: &T) {
    unsafe {
        lnExtiAttachInterrupt(
            rnpin2lnpin(pin),
            edge,
            Some(generic_trampoline::<T>),
            handler as *const T as *const cty::c_void,
        );
    }
}
// EOF
