#![allow(unused_variables, unused_imports, unused_parens)]
#![allow(non_camel_case_types, non_snake_case, dead_code)]
#![no_std]

mod registers;
use core::cell::UnsafeCell;
use core::ptr::{read_volatile, write_volatile};
use registers::ExtiRegisters;

rs_mcu_traits::define_mmio_accessors!(ExtiRegisters::ptr());
use rs_gpio_bluepill::Pin;
use rs_mcu_traits::Result;
use rs_mcu_traits::exti::{self, Edge as ExtiEdge, ExtiHandler};
use rs_mcu_traits::handler::HandlerTable;

/// One handler slot per pin (the line number is the pin within its port).
const EXTI_PIN_COUNT: usize = 112;

static EXTI_HANDLERS: HandlerTable<dyn ExtiHandler, EXTI_PIN_COUNT> = HandlerTable::new();

struct ExtiSlot {
    pin: u32,
    cb: Callback,
    cookie: *mut core::ffi::c_void,
}

struct SlotCell(UnsafeCell<ExtiSlot>);
unsafe impl Sync for SlotCell {}

impl SlotCell {
    const fn new() -> Self {
        Self(UnsafeCell::new(ExtiSlot {
            pin: 0,
            cb: None,
            cookie: core::ptr::null_mut(),
        }))
    }

    #[inline(always)]
    fn get(&self) -> *mut ExtiSlot {
        self.0.get()
    }
}

static EXTI_SLOTS: [SlotCell; 16] = [const { SlotCell::new() }; 16];

/// Adapter from this family's C-style callback to the typed handler of the trait.
unsafe extern "C" fn exti_trampoline(pin: Pin, _cookie: *mut core::ffi::c_void) {
    // SAFETY: the handler was installed before its line was enabled, and the
    // interrupt is the only reader.
    if let Some(handler) = unsafe { EXTI_HANDLERS.isr_get(pin as usize) } {
        handler.on_exti(pin as u32);
    }
}

/// Dispatch pending EXTI line to installed callback or trait handler.
#[unsafe(no_mangle)]
pub extern "C" fn rs_exti_irq_handler(line: usize) {
    exti_irq_handler(line);
}

/// Dispatch pending EXTI line to installed callback or trait handler.
pub fn exti_irq_handler(line: usize) {
    if line < 16 {
        unsafe {
            let pd = read_reg!(pd);
            if (pd & (1 << line)) != 0 {
                write_reg!(pd, 1 << line);
                let slot = EXTI_SLOTS[line].get();
                let (cb, cookie, pin_num) = ((*slot).cb, (*slot).cookie, (*slot).pin);
                if let Some(cb) = cb {
                    if let Some(pin) = Pin::from_number(pin_num) {
                        cb(pin, cookie);
                    }
                } else if let Some(handler) = EXTI_HANDLERS.isr_get(pin_num as usize) {
                    handler.on_exti(pin_num);
                }
            }
        }
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

pub fn attach_interrupt(pin: Pin, edge: Edge, cb: Callback, cookie: *mut core::ffi::c_void) {
    let port = (pin as u32) >> 4;
    let source = (pin as u32) & 0xF;

    if (source as usize) < 16 {
        critical_section::with(|_| unsafe {
            let slot = EXTI_SLOTS[source as usize].get();
            (*slot).pin = pin as u32;
            (*slot).cb = cb;
            (*slot).cookie = cookie;
        });
    }

    // 1. Select source via AFIO
    rs_afio_bluepill::select_exti_source(port, source);

    // 2. Program Edge via EXTI
    const EDGE_RISING_BIT: u32 = 1;
    const EDGE_FALLING_BIT: u32 = 2;

    unsafe {
        if (edge as u32) & EDGE_RISING_BIT != 0 {
            modify_reg!(rten, |v| v | (1 << source));
        } else {
            modify_reg!(rten, |v| v & !(1 << source));
        }

        if (edge as u32) & EDGE_FALLING_BIT != 0 {
            modify_reg!(ften, |v| v | (1 << source));
        } else {
            modify_reg!(ften, |v| v & !(1 << source));
        }
    }
}

pub fn detach_interrupt(pin: Pin) {
    let source = (pin as u32) & 0xF;
    if (source as usize) < 16 {
        critical_section::with(|_| unsafe {
            let slot = EXTI_SLOTS[source as usize].get();
            (*slot).cb = None;
            (*slot).cookie = core::ptr::null_mut();
        });
    }
    unsafe {
        modify_reg!(inten, |v| v & !(1 << source));
        modify_reg!(rten, |v| v & !(1 << source));
        modify_reg!(ften, |v| v & !(1 << source));
    }
}

pub fn enable_interrupt(pin: Pin) {
    let source = (pin as u32) & 0xF;
    unsafe {
        modify_reg!(inten, |v| v | (1 << source));

        core::arch::asm!("nop", "nop", "nop");
        write_reg!(pd, 1 << source);
    }
}

pub fn disable_interrupt(pin: Pin) {
    let source = (pin as u32) & 0xF;
    unsafe {
        modify_reg!(inten, |v| v & !(1 << source));

        core::arch::asm!("nop", "nop", "nop");
        write_reg!(pd, 1 << source);
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
        let p = Pin::from_number(pin).ok_or(rs_mcu_traits::Error::InvalidConfig)?;
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
        let p = Pin::from_number(pin).ok_or(rs_mcu_traits::Error::InvalidConfig)?;
        enable_interrupt(p);
        Ok(())
    }

    fn disable(&mut self, pin: u32) -> Result<()> {
        let p = Pin::from_number(pin).ok_or(rs_mcu_traits::Error::InvalidConfig)?;
        disable_interrupt(p);
        Ok(())
    }

    fn detach(&mut self, pin: u32) {
        EXTI_HANDLERS.set(pin as usize, None);
        detach_interrupt(Pin::from(pin));
    }
}

pub mod shim;
pub use shim::*;
