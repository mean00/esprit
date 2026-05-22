#![allow(dead_code)]

#[cfg(feature = "rp2040")]
mod import_gpio {
    pub use crate::rn_fast_gpio_rp2040 as rn_fast_gpio;
    pub(crate) use crate::rn_gpio_rp2040_c;
    pub(crate) use crate::rn_gpio_rp2040_c as gpio;
    pub use crate::rn_gpio_rp2040_c::lnPinMode_c;
}

#[cfg(feature = "esp32")]
mod import_gpio {
    pub use crate::rn_fast_gpio_esp32c3 as rn_fast_gpio;
    pub(crate) use crate::rn_gpio_esp32_c;
    pub(crate) use crate::rn_gpio_esp32_c as gpio;
    pub use crate::rn_gpio_esp32_c::lnPinMode_c;
}

#[cfg(not(any(feature = "rp2040", feature = "esp32")))]
mod import_gpio {
    pub use crate::rn_fast_gpio_bp as rn_fast_gpio;
    pub(crate) use crate::rn_gpio_bp_c;
    pub(crate) use crate::rn_gpio_bp_c as gpio;
    pub use crate::rn_gpio_bp_c::lnPinMode_c;
}

pub use import_gpio::gpio::lnGpioMode as GpioMode;
pub use import_gpio::gpio::lnPin;

// Re-export all the low-level GPIO C functions so that other modules
// can use `crate::gpio` as a single FFI facade.
pub use import_gpio::gpio::lnDigitalRead;
pub use import_gpio::gpio::lnDigitalToggle;
pub use import_gpio::gpio::lnDigitalWrite;
pub use import_gpio::gpio::lnGetGpioDirectionRegister;
pub use import_gpio::gpio::lnGetGpioOffRegister;
pub use import_gpio::gpio::lnGetGpioOnRegister;
pub use import_gpio::gpio::lnGetGpioToggleRegister;
pub use import_gpio::gpio::lnGetGpioValueRegister;
pub use import_gpio::gpio::lnOpenDrainClose;
pub use import_gpio::gpio::lnPinMode;
pub use import_gpio::gpio::lnPinMode_c;
pub use import_gpio::gpio::lnReadPort;

/// Convert a platform-specific pin enum val to the canonical `lnPin`.
/// The platform pin types (e.g. `rnPin`) are C-compatible integer enums
/// that differ per MCU.  This helper transmutes them into the common
/// `lnPin` expected by the low-level GPIO C API.
#[inline]
pub fn pin_to_lnpin(pin: impl Into<i32>) -> lnPin {
    let val: i32 = pin.into();
    unsafe { core::mem::transmute(val) }
}

//  Convenience free functions — 2‑arg wrappers around the 3‑arg C API

/// Set pin mode (output, input, etc.) with default speed 0 MHz.
pub fn pin_mode(pin: lnPin, mode: GpioMode) {
    unsafe { lnPinMode_c(pin, mode, 0) }
}

/// Set pin mode with explicit I/O speed in MHz.
pub fn pin_mode_speed(pin: lnPin, mode: GpioMode, speed_mhz: u32) {
    unsafe { lnPinMode_c(pin, mode, speed_mhz) }
}

/// Drive the pin high (`true`) or low (`false`).
pub fn digital_write(pin: lnPin, val: bool) {
    unsafe { lnDigitalWrite(pin, val) }
}

/// Toggle the output state of the pin.
pub fn digital_toggle(pin: lnPin) {
    unsafe { lnDigitalToggle(pin) }
}

/// Read the cur input level of the pin.
pub fn digital_read(pin: lnPin) -> bool {
    unsafe { lnDigitalRead(pin) }
}

//  Typed GPIO pin (was in pin.rs)

/// Represents a microcontroller GPIO pin.
/// Created via `Pin::new(lnPin::PC13)` and then configured
/// with methods like `.set_high()`, `.toggle()`, etc.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Pin {
    pin: lnPin,
}

impl Pin {
    /// Create a new pin from the raw C enum val.
    /// # Example
    /// ```ignore
    /// let led = Pin::new(lnPin::PC13);
    /// led.set_mode(GpioMode::lnOUTPUT);
    /// led.set_high();
    /// ```
    pub fn new(pin: lnPin) -> Self {
        Self { pin }
    }

    /// Return the underlying C enum.
    pub fn raw(self) -> lnPin {
        self.pin
    }

    /// Return the port index for this pin (0 = PA, 1 = PB, ...).
    pub fn port(self) -> u32 {
        (self.pin as u32) >> 4
    }

    /// Return the bit num within the port (0..15).
    pub fn bit(self) -> u32 {
        (self.pin as u32) & 0xF
    }

    /// Set the pin mode.
    pub fn set_mode(&mut self, mode: GpioMode) {
        unsafe {
            lnPinMode_c(self.pin, mode, 0);
        }
    }

    /// Set the pin mode with a target speed in MHz.
    pub fn set_mode_speed(&mut self, mode: GpioMode, speed_mhz: u32) {
        unsafe {
            lnPinMode_c(self.pin, mode, speed_mhz);
        }
    }

    /// Drive the pin high (digital `1`).
    pub fn set_high(&mut self) {
        unsafe {
            lnDigitalWrite(self.pin, true);
        }
    }

    /// Drive the pin low (digital `0`).
    pub fn set_low(&mut self) {
        unsafe {
            lnDigitalWrite(self.pin, false);
        }
    }

    /// Write a boolean val to the pin (`true` = high, `false` = low).
    pub fn write(&mut self, val: bool) {
        unsafe {
            lnDigitalWrite(self.pin, val);
        }
    }

    /// Toggle the output state.
    pub fn toggle(&mut self) {
        unsafe {
            lnDigitalToggle(self.pin);
        }
    }

    /// Return `true` if the input reads as high.
    pub fn is_high(&self) -> bool {
        unsafe { lnDigitalRead(self.pin) }
    }

    /// Return `true` if the input reads as low.
    pub fn is_low(&self) -> bool {
        !self.is_high()
    }

    /// Read the entire GPIO port.
    pub fn read_port(&self) -> u32 {
        unsafe { lnReadPort(self.port()) }
    }

    /// Open-drain close/open helper.
    pub fn open_drain_close(&mut self, close: bool) {
        unsafe {
            lnOpenDrainClose(self.pin, close);
        }
    }

    // -- Fast I/O register access (expert) --

    /// Get a pointer to the toggle register for this pin's port.
    /// Writing `1 << bit()` toggles the pin with a single bus cycle.
    pub fn toggle_register(&self) -> *mut u32 {
        unsafe { lnGetGpioToggleRegister(self.port()) }
    }

    /// Get a pointer to the "on" (set) register for this pin's port.
    pub fn on_register(&self) -> *mut u32 {
        unsafe { lnGetGpioOnRegister(self.port()) }
    }

    /// Get a pointer to the "off" (clear) register for this pin's port.
    pub fn off_register(&self) -> *mut u32 {
        unsafe { lnGetGpioOffRegister(self.port()) }
    }

    /// Get a pointer to the direction register for this pin's port.
    pub fn direction_register(&self) -> *mut u32 {
        unsafe { lnGetGpioDirectionRegister(self.port()) }
    }

    /// Get a pointer to the input val register for this pin's port.
    pub fn val_register(&self) -> *mut u32 {
        unsafe { lnGetGpioValueRegister(self.port()) }
    }
}

