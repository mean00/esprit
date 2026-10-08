#![no_std]
#![allow(dead_code)]

pub mod ffi;
pub use ffi::*;

use rs_esprit::gpio::{Exti as ExtiTrait, EdgeTrigger};

pub struct ExtiWrapped;

impl ExtiTrait for ExtiWrapped {
    fn attach_interrupt(
        &mut self,
        pin: u32,
        edge: EdgeTrigger,
        callback: Option<unsafe extern "C" fn(pin: i32, cookie: *mut core::ffi::c_void)>,
        cookie: *mut core::ffi::c_void,
    ) {
        let e = match edge {
            EdgeTrigger::None => lnEdge::LN_EDGE_NONE,
            EdgeTrigger::Rising => lnEdge::LN_EDGE_RISING,
            EdgeTrigger::Falling => lnEdge::LN_EDGE_FALLING,
            EdgeTrigger::Both => lnEdge::LN_EDGE_BOTH,
        };
        unsafe {
            lnExtiAttachInterrupt_c(pin as lnPin, e, callback, cookie);
        }
    }

    fn detach_interrupt(&mut self, pin: u32) {
        unsafe {
            lnExtiDetachInterrupt_c(pin as lnPin);
        }
    }

    fn enable_interrupt(&mut self, pin: u32) {
        unsafe {
            lnExtiEnableInterrupt_c(pin as lnPin);
        }
    }

    fn disable_interrupt(&mut self, pin: u32) {
        unsafe {
            lnExtiDisableInterrupt_c(pin as lnPin);
        }
    }
}
