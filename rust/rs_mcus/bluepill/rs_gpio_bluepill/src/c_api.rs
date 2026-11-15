//! The C entries of the GPIO driver.
//!
//! `esprit/mcus/common_bluepill/lnGPIO.cpp` is not compiled in a Rust build
//! (`drop_C.md` P5); the C++ side reaches these through `esprit/include/lnGPIO.h`,
//! whose declarations become `extern "C"` when `LN_RUST_BLUEPILL` is defined
//! (`esprit/CMakeLists.txt`). The enum values cross unchanged: `enum lnPin` and
//! `enum lnGpioMode` are `int` on the C++ side and [`Pin`] / [`Mode`] are
//! `#[repr(u32)]` with the same discriminants (`lnGPIO.h`, `rs_gpio_bluepill`).
//!
//! Semantics are the C++ file's, one for one; `lnGetGpioStatusRegister()` is the only
//! addition: `lnFastIO`'s constructor needs the port's *input status* register, which
//! the C++ reached through its private `gpio[]` table (see `lnFastGpio.h`).

use crate::*;

#[inline]
fn pin_of(number: i32) -> Option<Pin> {
    Pin::from_number(number as u32)
}

/// `enum lnGpioMode` has 14 entries, `Floating = 0` .. `UartAlt = 13`.
#[inline]
fn mode_of(value: i32) -> Option<Mode> {
    if (0..=13).contains(&value) {
        // SAFETY: 0..=13 are all valid discriminants of `Mode`.
        Some(unsafe { core::mem::transmute::<u32, Mode>(value as u32) })
    } else {
        None
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lnPinMode(pin: i32, mode: i32, speed_in_mhz: u32) {
    if let (Some(p), Some(m)) = (pin_of(pin), mode_of(mode)) {
        set_mode(p, m, speed_in_mhz);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lnDigitalWrite(pin: i32, value: bool) {
    if let Some(p) = pin_of(pin) {
        write(p, value);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lnDigitalRead(pin: i32) -> bool {
    pin_of(pin).map(read).unwrap_or(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn lnDigitalToggle(pin: i32) {
    if let Some(p) = pin_of(pin) {
        toggle(p);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lnOpenDrainClose(pin: i32, close: bool) {
    if let Some(p) = pin_of(pin) {
        open_drain_close(p, close);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lnReadPort(port: u32) -> u32 {
    read_port(port)
}

/// Bring-up register (`BOP`): what `lnFastIO::on()`/`off()` write.
#[unsafe(no_mangle)]
pub extern "C" fn lnGetGpioToggleRegister(port: u32) -> *mut u32 {
    unsafe { core::ptr::addr_of_mut!((*get_port_ptr(port)).bop) }
}

/// Configuration register of the pin's low nibble (`CTL0`).
#[unsafe(no_mangle)]
pub extern "C" fn lnGetGpioDirectionRegister(port: u32) -> *mut u32 {
    unsafe { core::ptr::addr_of_mut!((*get_port_ptr(port)).ctl0) }
}

/// Output level register (`OCTL`) - the C++ naming: "bit value for the LOW bits".
#[unsafe(no_mangle)]
pub extern "C" fn lnGetGpioValueRegister(port: u32) -> *mut u32 {
    unsafe { core::ptr::addr_of_mut!((*get_port_ptr(port)).octl) }
}

/// Input status register (`ISTAT`) - not in the C++ header, added for the inline
/// `lnFastIO` constructor that used to read the private `gpio[]` table.
#[unsafe(no_mangle)]
pub extern "C" fn lnGetGpioStatusRegister(port: u32) -> *mut u32 {
    unsafe { core::ptr::addr_of_mut!((*get_port_ptr(port)).istat) }
}

/// Unused by the C++ that is left, kept because `lnGPIO.h` declares it.
#[unsafe(no_mangle)]
pub extern "C" fn lnGetGpioOnRegister(port: u32) -> *mut u32 {
    unsafe { core::ptr::addr_of_mut!((*get_port_ptr(port)).bop) }
}

/// Unused by the C++ that is left, kept because `lnGPIO.h` declares it.
#[unsafe(no_mangle)]
pub extern "C" fn lnGetGpioOffRegister(port: u32) -> *mut u32 {
    unsafe { core::ptr::addr_of_mut!((*get_port_ptr(port)).bc) }
}
