#![allow(dead_code)]

//! GPIO abstraction: pin identifiers ([`Pin`]), owned pins ([`GpioPin`])
//! and pin modes ([`GpioMode`]).
//!
//! The raw C names (`lnPin`, `lnGpioMode`, `lnPinMode_c`, …) are re‑exported
//! under `rust_esprit::raw` for advanced users.

#[cfg(feature = "rp2040")]
mod import_gpio {
    pub(crate) use crate::rn_gpio_rp2040_c;
    pub(crate) use crate::rn_gpio_rp2040_c as gpio;
    pub use crate::rn_gpio_rp2040_c::lnPinMode_c;
}

#[cfg(feature = "esp32")]
mod import_gpio {
    pub(crate) use crate::rn_gpio_esp32_c;
    pub(crate) use crate::rn_gpio_esp32_c as gpio;
    pub use crate::rn_gpio_esp32_c::lnPinMode_c;
}

#[cfg(not(any(feature = "rp2040", feature = "esp32")))]
mod import_gpio {
    pub(crate) use crate::rn_gpio_bp_c;
    pub(crate) use crate::rn_gpio_bp_c as gpio;
    pub use crate::rn_gpio_bp_c::lnPinMode_c;
}

// Internal FFI facade: other modules reach the raw GPIO functions through
// `crate::gpio`.  The same items are re-exported publicly as `raw::…`.
pub(crate) use import_gpio::gpio::lnDigitalRead;
pub(crate) use import_gpio::gpio::lnDigitalToggle;
pub(crate) use import_gpio::gpio::lnDigitalWrite;
pub(crate) use import_gpio::gpio::lnGetGpioDirectionRegister;
pub(crate) use import_gpio::gpio::lnGetGpioOffRegister;
pub(crate) use import_gpio::gpio::lnGetGpioOnRegister;
pub(crate) use import_gpio::gpio::lnGetGpioToggleRegister;
pub(crate) use import_gpio::gpio::lnGetGpioValueRegister;
pub(crate) use import_gpio::gpio::lnOpenDrainClose;
pub(crate) use import_gpio::gpio::lnPin;
pub(crate) use import_gpio::gpio::lnPinMode_c;
pub(crate) use import_gpio::gpio::lnReadPort;

// ---------------------------------------------------------------------------
//  Pin identifiers
// ---------------------------------------------------------------------------

/// Canonical pin identifier for the current target.
///
/// This is the same type as `rust_esprit::raw::lnPin`; the available variants
/// depend on the target — GD32 uses `Pin::PA0` … `Pin::PG15`, RP2040/RP2350
/// and ESP32 use `Pin::GPIO0` … .
///
/// # Example
/// ```ignore
/// let led = Pin::PB6;
/// ```
#[doc(alias = "lnPin")]
pub use import_gpio::gpio::lnPin as Pin;

/// Legacy name for [`Pin`].
#[deprecated(note = "use `Pin` instead")]
#[doc(alias = "Pin")]
pub type pin = Pin;

/// Build a [`Pin`] from a raw numeric value (advanced).
impl From<u32> for Pin {
    fn from(val: u32) -> Self {
        // SAFETY: `lnPin` is a C enum; this mirrors the cast historically
        // performed by `pin_to_lnpin`.
        unsafe { core::mem::transmute(val) }
    }
}

// ---------------------------------------------------------------------------
//  Pin modes
// ---------------------------------------------------------------------------

/// GPIO pin mode.
///
/// Idiomatic replacement for the raw C `lnGpioMode` enum.  The variant order
/// and values match the C enum exactly.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[repr(u32)]
pub enum GpioMode {
    Floating = 0,
    InputFloating = 1,
    InputPullUp = 2,
    InputPullDown = 3,
    Output = 4,
    OutputOpenDrain = 5,
    AlternatePushPull = 6,
    AlternateOpenDrain = 7,
    Pwm = 8,
    Adc = 9,
    Dac = 10,
    Uart = 11,
    Spi = 12,
    UartAlt = 13,
}

impl From<GpioMode> for crate::raw::lnGpioMode {
    fn from(mode: GpioMode) -> Self {
        use crate::raw::lnGpioMode::{
            lnADC_MODE, lnALTERNATE_OD, lnALTERNATE_PP, lnDAC_MODE, lnFLOATING,
            lnINPUT_FLOATING, lnINPUT_PULLDOWN, lnINPUT_PULLUP, lnOUTPUT,
            lnOUTPUT_OPEN_DRAIN, lnPWM, lnSPI_MODE, lnUART, lnUART_Alt,
        };
        match mode {
            GpioMode::Floating => lnFLOATING,
            GpioMode::InputFloating => lnINPUT_FLOATING,
            GpioMode::InputPullUp => lnINPUT_PULLUP,
            GpioMode::InputPullDown => lnINPUT_PULLDOWN,
            GpioMode::Output => lnOUTPUT,
            GpioMode::OutputOpenDrain => lnOUTPUT_OPEN_DRAIN,
            GpioMode::AlternatePushPull => lnALTERNATE_PP,
            GpioMode::AlternateOpenDrain => lnALTERNATE_OD,
            GpioMode::Pwm => lnPWM,
            GpioMode::Adc => lnADC_MODE,
            GpioMode::Dac => lnDAC_MODE,
            GpioMode::Uart => lnUART,
            GpioMode::Spi => lnSPI_MODE,
            GpioMode::UartAlt => lnUART_Alt,
        }
    }
}

impl From<GpioMode> for u32 {
    fn from(mode: GpioMode) -> u32 {
        mode as u32
    }
}

// ---------------------------------------------------------------------------
//  Deprecated free functions (legacy names)
// ---------------------------------------------------------------------------

/// Set pin mode (output, input, etc.) with default speed 0 MHz.
#[deprecated(note = "use `GpioPin::set_mode` instead")]
pub fn pin_mode(pin: Pin, mode: GpioMode) {
    unsafe { lnPinMode_c(pin, mode.into(), 0) }
}

/// Set pin mode with explicit I/O speed in MHz.
#[deprecated(note = "use `GpioPin::set_mode_speed` instead")]
pub fn pin_mode_speed(pin: Pin, mode: GpioMode, speed_mhz: u32) {
    unsafe { lnPinMode_c(pin, mode.into(), speed_mhz) }
}

/// Drive the pin high (`true`) or low (`false`).
#[deprecated(note = "use `GpioPin::write` instead")]
pub fn digital_write(pin: Pin, val: bool) {
    unsafe { lnDigitalWrite(pin, val) }
}

/// Toggle the output state of the pin.
#[deprecated(note = "use `GpioPin::toggle` instead")]
pub fn digital_toggle(pin: Pin) {
    unsafe { lnDigitalToggle(pin) }
}

/// Read the current input level of the pin.
#[deprecated(note = "use `GpioPin::is_high` instead")]
pub fn digital_read(pin: Pin) -> bool {
    unsafe { lnDigitalRead(pin) }
}

/// Convert a raw integer pin index to the canonical [`Pin`].
///
/// The platform pin types (e.g. `rnPin`) are C-compatible integer enums that
/// differ per MCU; this helper casts them into the common `lnPin` expected by
/// the low-level GPIO C API.
#[deprecated(note = "use `Pin::from` instead")]
pub fn pin_to_lnpin(pin: impl Into<i32>) -> Pin {
    Pin::from(pin.into() as u32)
}

// ---------------------------------------------------------------------------
//  Owned GPIO pin
// ---------------------------------------------------------------------------

/// An owned, copyable microcontroller GPIO pin.
///
/// Created via [`GpioPin::new`] (or `From<Pin>`) and configured with methods
/// like `.set_mode()`, `.set_high()`, `.toggle()`, etc.
///
/// # Example
/// ```ignore
/// let mut led = GpioPin::new(Pin::PB6);
/// led.set_mode(GpioMode::Output);
/// led.set_high();
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct GpioPin {
    pin: Pin,
}

impl From<Pin> for GpioPin {
    fn from(pin: Pin) -> Self {
        Self { pin }
    }
}

impl From<GpioPin> for Pin {
    fn from(pin: GpioPin) -> Self {
        pin.pin
    }
}

impl GpioPin {
    /// Create a new pin from the canonical pin enum value.
    pub fn new(pin: Pin) -> Self {
        Self { pin }
    }

    /// Return the underlying pin identifier.
    pub fn pin(self) -> Pin {
        self.pin
    }

    /// Return the underlying pin identifier (same as [`GpioPin::pin`]).
    #[inline]
    pub fn raw(self) -> Pin {
        self.pin
    }

    /// Return the port index for this pin (0 = PA, 1 = PB, ...).
    pub fn port(self) -> u32 {
        (self.pin as u32) >> 4
    }

    /// Return the bit number within the port (0..15).
    pub fn bit(self) -> u32 {
        (self.pin as u32) & 0xF
    }

    /// Set the pin mode.
    pub fn set_mode(&mut self, mode: GpioMode) {
        unsafe { lnPinMode_c(self.pin, mode.into(), 0) }
    }

    /// Set the pin mode with a target speed in MHz.
    pub fn set_mode_speed(&mut self, mode: GpioMode, speed_mhz: u32) {
        unsafe { lnPinMode_c(self.pin, mode.into(), speed_mhz) }
    }

    /// Drive the pin high (digital `1`).
    pub fn set_high(&mut self) {
        unsafe { lnDigitalWrite(self.pin, true) }
    }

    /// Drive the pin low (digital `0`).
    pub fn set_low(&mut self) {
        unsafe { lnDigitalWrite(self.pin, false) }
    }

    /// Write a boolean value to the pin (`true` = high, `false` = low).
    pub fn write(&mut self, val: bool) {
        unsafe { lnDigitalWrite(self.pin, val) }
    }

    /// Toggle the output state.
    pub fn toggle(&mut self) {
        unsafe { lnDigitalToggle(self.pin) }
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
        unsafe { lnOpenDrainClose(self.pin, close) }
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

    /// Get a pointer to the input value register for this pin's port.
    pub fn val_register(&self) -> *mut u32 {
        unsafe { lnGetGpioValueRegister(self.port()) }
    }
}

