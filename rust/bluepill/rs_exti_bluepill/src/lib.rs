#![allow(unused_variables, unused_imports, unused_parens)]
#![allow(non_camel_case_types, non_snake_case, dead_code)]

#![no_std]

mod registers;
use registers::ExtiRegisters;
use rs_gpio_bluepill::Pin;
use core::ptr::{read_volatile, write_volatile};

// --- Idiomatic Rust Core ---

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Edge {
    None = 0,
    Rising = 1,
    Falling = 2,
    Both = 3,
}

pub type Callback = ::core::option::Option<unsafe extern "C" fn(pin: Pin, cookie: *mut core::ffi::c_void)>;

pub fn attach_interrupt(pin: Pin, edge: Edge, _cb: Callback, _cookie: *mut core::ffi::c_void) {
    let port = (pin as u32) >> 4;
    let source = (pin as u32) & 0xF;
    
    // 1. Select source via AFIO
    let afio = rs_afio_bluepill::registers::AfioRegisters::ptr();
    unsafe {
        let mut mask = read_volatile(&mut (*afio).extiss[(source >> 2) as usize]);
        let shift = 4 * (source & 3);
        mask &= !(0xF << shift);
        mask |= port << shift;
        write_volatile(&mut (*afio).extiss[(source >> 2) as usize], mask);
    }
    
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

// --- Legacy Bridge (Adapter Pattern) ---
pub type lnEdge = Edge;
pub type lnExtiCallback = Callback;

#[inline(always)]
#[allow(non_snake_case)]
pub fn lnExtiAttachInterrupt_c(
    pin: rs_gpio_bluepill::lnPin,
    edge: lnEdge,
    cb: lnExtiCallback,
    cookie: *mut core::ffi::c_void,
) {
    attach_interrupt(pin, edge, cb, cookie);
}

#[inline(always)]
#[allow(non_snake_case)]
pub fn lnExtiDetachInterrupt_c(pin: rs_gpio_bluepill::lnPin) {
    detach_interrupt(pin);
}

#[inline(always)]
#[allow(non_snake_case)]
pub fn lnExtiEnableInterrupt_c(pin: rs_gpio_bluepill::lnPin) {
    enable_interrupt(pin);
}

#[inline(always)]
#[allow(non_snake_case)]
pub fn lnExtiDisableInterrupt_c(pin: rs_gpio_bluepill::lnPin) {
    disable_interrupt(pin);
}
