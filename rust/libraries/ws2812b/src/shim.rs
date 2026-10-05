#![allow(dead_code)]

//! Legacy C ABI compatibility shim for WS2812B driver.
//!
//! Exposes `extern "C"` functions and C handles for backward compatibility
//! with C and C++ codebases without polluting the idiomatic Rust API.

use crate::{Pin, Ws2812b};

/// Concrete instance handle for C interoperability (supports up to 64 LEDs).
pub struct Ws2812bCInstance {
    pub inner: Ws2812b<64>,
}

#[unsafe(no_mangle)]
pub extern "C" fn ws2812b_create(pin: u32) -> *mut Ws2812bCInstance {
    let _pin = Pin::from(pin);
    // For no_std without global allocator, caller can manage instance storage
    core::ptr::null_mut()
}

#[unsafe(no_mangle)]
pub extern "C" fn ws2812b_begin(handle: *mut Ws2812bCInstance) -> bool {
    if handle.is_null() {
        return false;
    }
    unsafe { (*handle).inner.begin().is_ok() }
}

#[unsafe(no_mangle)]
pub extern "C" fn ws2812b_set_global_brightness(handle: *mut Ws2812bCInstance, brightness: u8) {
    if !handle.is_null() {
        unsafe { (*handle).inner.set_global_brightness(brightness) }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ws2812b_set_color(handle: *mut Ws2812bCInstance, r: u8, g: u8, b: u8) {
    if !handle.is_null() {
        unsafe { (*handle).inner.set_color(r, g, b) }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ws2812b_set_led_color(
    handle: *mut Ws2812bCInstance,
    led: u32,
    r: u8,
    g: u8,
    b: u8,
) {
    if !handle.is_null() {
        unsafe { (*handle).inner.set_led_color(led as usize, r, g, b) }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ws2812b_update(handle: *mut Ws2812bCInstance) {
    if !handle.is_null() {
        unsafe { (*handle).inner.update() }
    }
}
