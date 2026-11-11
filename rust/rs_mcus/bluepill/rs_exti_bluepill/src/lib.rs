#![allow(unused_variables, unused_imports, unused_parens)]
#![allow(non_camel_case_types, non_snake_case, dead_code)]
#![no_std]

mod registers;
use core::ptr::{read_volatile, write_volatile};
use registers::ExtiRegisters;
use rs_gpio_bluepill::Pin;
use rs_mcu_traits::Result;
use rs_mcu_traits::exti::{self, Edge as ExtiEdge, ExtiHandler};
use rs_mcu_traits::handler::HandlerTable;

/// One handler slot per pin (the line number is the pin within its port).
const EXTI_PIN_COUNT: usize = 112;

static EXTI_HANDLERS: HandlerTable<dyn ExtiHandler, EXTI_PIN_COUNT> = HandlerTable::new();

/// Adapter from this family's C-style callback to the typed handler of the trait.
unsafe extern "C" fn exti_trampoline(pin: Pin, _cookie: *mut core::ffi::c_void) {
    // SAFETY: the handler was installed before its line was enabled, and the
    // interrupt is the only reader.
    if let Some(handler) = unsafe { EXTI_HANDLERS.isr_get(pin as usize) } {
        handler.on_exti(pin as u32);
    }
}

// --- Idiomatic Rust Core ---

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Edge {
    None = 0,
    Rising = 1,
    Falling = 2,
    Both = 3,
}

pub type Callback =
    ::core::option::Option<unsafe extern "C" fn(pin: Pin, cookie: *mut core::ffi::c_void)>;

pub fn attach_interrupt(pin: Pin, edge: Edge, _cb: Callback, _cookie: *mut core::ffi::c_void) {
    let port = (pin as u32) >> 4;
    let source = (pin as u32) & 0xF;

    // 1. Select source via AFIO
    rs_afio_bluepill::select_exti_source(port, source);

    // 2. Program Edge via EXTI
    const EDGE_RISING_BIT: u32 = 1;
    const EDGE_FALLING_BIT: u32 = 2;

    let exti = ExtiRegisters::ptr();
    unsafe {
        if (edge as u32) & EDGE_RISING_BIT != 0 {
            let val = read_volatile(&mut (*exti).rten) | (1 << source);
            write_volatile(&mut (*exti).rten, val);
        } else {
            let val = read_volatile(&mut (*exti).rten) & !(1 << source);
            write_volatile(&mut (*exti).rten, val);
        }

        if (edge as u32) & EDGE_FALLING_BIT != 0 {
            let val = read_volatile(&mut (*exti).ften) | (1 << source);
            write_volatile(&mut (*exti).ften, val);
        } else {
            let val = read_volatile(&mut (*exti).ften) & !(1 << source);
            write_volatile(&mut (*exti).ften, val);
        }
    }
}

pub fn detach_interrupt(pin: Pin) {
    let source = (pin as u32) & 0xF;
    let exti = ExtiRegisters::ptr();
    unsafe {
        let inten = read_volatile(&mut (*exti).inten) & !(1 << source);
        write_volatile(&mut (*exti).inten, inten);

        let rten = read_volatile(&mut (*exti).rten) & !(1 << source);
        write_volatile(&mut (*exti).rten, rten);

        let ften = read_volatile(&mut (*exti).ften) & !(1 << source);
        write_volatile(&mut (*exti).ften, ften);
    }
}

pub fn enable_interrupt(pin: Pin) {
    let source = (pin as u32) & 0xF;
    let exti = ExtiRegisters::ptr();
    unsafe {
        let inten = read_volatile(&mut (*exti).inten) | (1 << source);
        write_volatile(&mut (*exti).inten, inten);

        core::arch::asm!("nop", "nop", "nop");
        write_volatile(&mut (*exti).pd, 1 << source);
    }
}

pub fn disable_interrupt(pin: Pin) {
    let source = (pin as u32) & 0xF;
    let exti = ExtiRegisters::ptr();
    unsafe {
        let inten = read_volatile(&mut (*exti).inten) & !(1 << source);
        write_volatile(&mut (*exti).inten, inten);

        core::arch::asm!("nop", "nop", "nop");
        write_volatile(&mut (*exti).pd, 1 << source);
    }
}

/// Idiomatic struct interface for External Interrupts (EXTI).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Exti;

impl Exti {
    /// The controller handle (the state lives in the EXTI registers and in the
    /// crate's handler table, so this is just a handle).
    #[inline(always)]
    pub const fn new() -> Self {
        Self
    }

    #[inline(always)]
    pub fn attach_interrupt(pin: Pin, edge: Edge, cb: Callback, cookie: *mut core::ffi::c_void) {
        attach_interrupt(pin, edge, cb, cookie);
    }

    #[inline(always)]
    pub fn detach_interrupt(pin: Pin) {
        detach_interrupt(pin);
    }

    #[inline(always)]
    pub fn enable_interrupt(pin: Pin) {
        enable_interrupt(pin);
    }

    #[inline(always)]
    pub fn disable_interrupt(pin: Pin) {
        disable_interrupt(pin);
    }
}

impl Default for Exti {
    fn default() -> Self {
        Self::new()
    }
}

impl exti::Exti for Exti {
    fn attach(
        &mut self,
        pin: u32,
        edge: ExtiEdge,
        handler: Option<&'static dyn ExtiHandler>,
    ) -> Result<()> {
        let p = Pin::from(pin);
        let e = match edge {
            ExtiEdge::Rising => Edge::Rising,
            ExtiEdge::Falling => Edge::Falling,
            ExtiEdge::Both => Edge::Both,
        };
        match handler {
            Some(handler) => {
                EXTI_HANDLERS.set(pin as usize, Some(handler));
                // The family callback carries the pin as an `lnPin`, so the
                // trampoline needs no cookie to find the handler again.
                let callback: Callback = Some(exti_trampoline);
                attach_interrupt(p, e, callback, core::ptr::null_mut());
            }
            None => {
                EXTI_HANDLERS.set(pin as usize, None);
                detach_interrupt(p);
            }
        }
        Ok(())
    }

    fn enable(&mut self, pin: u32) -> Result<()> {
        enable_interrupt(Pin::from(pin));
        Ok(())
    }

    fn disable(&mut self, pin: u32) -> Result<()> {
        disable_interrupt(Pin::from(pin));
        Ok(())
    }

    fn detach(&mut self, pin: u32) {
        EXTI_HANDLERS.set(pin as usize, None);
        detach_interrupt(Pin::from(pin));
    }
}

pub mod shim;
pub use shim::*;
