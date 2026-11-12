#![no_std]
#![allow(dead_code)]

pub mod ffi;
pub use ffi::*;

use rs_mcu_traits::Result as HalResult;
use rs_mcu_traits::exti::{self, Edge, ExtiHandler};
use rs_mcu_traits::handler::HandlerTable;

/// One handler slot per pin: the C callback of this family carries the pin
/// number, so the trampoline can find the right handler without a cookie.
const EXTI_PIN_COUNT: usize = 112;

static HANDLERS: HandlerTable<dyn ExtiHandler, EXTI_PIN_COUNT> = HandlerTable::new();

/// The wrapped target's external-interrupt controller.
pub struct ExtiWrapped;

impl ExtiWrapped {
    /// The controller handle (all state lives in the platform's EXTI block and in
    /// the crate's handler table).
    pub const fn new() -> Self {
        Self
    }
}

impl Default for ExtiWrapped {
    fn default() -> Self {
        Self::new()
    }
}

/// Adapter from the family's C-style callback to the typed handler of the trait.
///
/// `pin` is the `lnPin` value the EXTI interrupt reports, i.e. the same number the
/// handler was installed with.
unsafe extern "C" fn exti_trampoline(pin: i32, _cookie: *mut core::ffi::c_void) {
    // SAFETY: the handler was installed before its line was enabled, and the
    // interrupt is the only reader.
    if let Some(handler) = unsafe { HANDLERS.isr_get(pin as usize) } {
        handler.on_exti(pin as u32);
    }
}

/// Map the trait's [`Edge`] to this family's.
fn raw_edge(edge: Edge) -> lnEdge {
    match edge {
        Edge::Rising => lnEdge::LN_EDGE_RISING,
        Edge::Falling => lnEdge::LN_EDGE_FALLING,
        Edge::Both => lnEdge::LN_EDGE_BOTH,
    }
}

impl exti::Exti for ExtiWrapped {
    fn attach(
        &mut self,
        pin: u32,
        edge: Edge,
        handler: Option<&'static dyn ExtiHandler>,
    ) -> HalResult<()> {
        match handler {
            Some(handler) => {
                HANDLERS.set(pin as usize, Some(handler));
                unsafe {
                    lnExtiAttachInterrupt_c(
                        pin as lnPin,
                        raw_edge(edge),
                        Some(exti_trampoline),
                        core::ptr::null_mut(),
                    );
                }
            }
            None => {
                HANDLERS.set(pin as usize, None);
                unsafe {
                    lnExtiDetachInterrupt_c(pin as lnPin);
                }
            }
        }
        Ok(())
    }

    fn enable(&mut self, pin: u32) -> HalResult<()> {
        unsafe {
            lnExtiEnableInterrupt_c(pin as lnPin);
        }
        Ok(())
    }

    fn disable(&mut self, pin: u32) -> HalResult<()> {
        unsafe {
            lnExtiDisableInterrupt_c(pin as lnPin);
        }
        Ok(())
    }

    fn detach(&mut self, pin: u32) {
        HANDLERS.set(pin as usize, None);
        unsafe {
            lnExtiDetachInterrupt_c(pin as lnPin);
        }
    }
}
