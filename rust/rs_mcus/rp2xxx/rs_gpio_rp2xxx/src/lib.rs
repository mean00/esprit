#![no_std]

//! Pure-Rust GPIO for the RP2040 and the RP2350: the `rs_esprit::gpio::GpioPin`
//! super trait implemented straight on SIO / IO_BANK0 / PADS_BANK0
//! (`registers.rs`, numbers from `rp2040.md`; the RP2350 has the same layout for
//! every field used here). The RP2xxx sibling of
//! `esprit/rust/rs_mcus/bluepill/rs_gpio_bluepill`: pin type, mode mapping, open-drain
//! sequencing and registers in one crate.
//!
//! [`Pin`] is *the* pin type of the RP2xxx Rust stack — `rust_esprit` re-exports it
//! as its `Pin`, and the RP2xxx C bindings (`rn_gpio_rp2040_c`) use it as their
//! `lnPin`, so a pin has one single type from the probe board table down to the
//! register write.
//!
//! No C ABI is exported on purpose: on RP2xxx esprit's C++ GPIO
//! (`esprit/mcus/rp_common/src/ln_rp_gpio.cpp`) is still linked and owns the
//! `lnDigitalWrite` / `lnPinMode` / `lnOpenDrainClose` symbols, so aliasing them
//! here would be a duplicate definition. The mode mapping and the open-drain
//! sequence below mirror that C++ file bit for bit, so a mixed firmware (C++ SWD
//! pins, Rust reset line) stays consistent.

pub mod registers;

use registers::*;
use rs_esprit::gpio::{EdgeTrigger, GpioInterruptHandler, GpioMode, GpioPin};

/// The bit of `pin` inside the SIO masks.
fn bit(pin: u32) -> u32 {
    1 << pin
}

fn sio_write(offset: u32, value: u32) {
    // SAFETY: the offsets are the documented SIO registers (registers.rs).
    unsafe { core::ptr::write_volatile((SIO_BASE + offset) as *mut u32, value) }
}

fn sio_read(offset: u32) -> u32 {
    // SAFETY: same as `sio_write`, read side.
    unsafe { core::ptr::read_volatile((SIO_BASE + offset) as *const u32) }
}

/// Drive the pin high (`true`) or low (`false`) through the SIO output latch.
fn drive(pin: u32, value: bool) {
    sio_write(
        if value {
            SIO_GPIO_OUT_SET
        } else {
            SIO_GPIO_OUT_CLR
        },
        bit(pin),
    );
}

/// Read the pad level (needs the pad's input enable).
fn sense(pin: u32) -> bool {
    sio_read(SIO_GPIO_IN) & bit(pin) != 0
}

/// Invert the SIO output latch.
fn flip(pin: u32) {
    sio_write(SIO_GPIO_OUT_XOR, bit(pin));
}

/// Write a pin's CTRL register: which function drives it and who owns the output
/// enable (`GPIO_CTRL_OE_*`).
fn write_ctrl(pin: u32, funcsel: u32, oe: u32) {
    let address = IO_BANK0_BASE + pin * GPIO_BLOCK_STRIDE + GPIO_CTRL_OFFSET;
    let value = (funcsel & GPIO_CTRL_FUNCSEL_MASK) << GPIO_CTRL_FUNCSEL_LSB
        | (oe & GPIO_CTRL_OE_MASK) << GPIO_CTRL_OE_LSB;
    // SAFETY: one 8-byte block per pin, CTRL at +0x004 (datasheet 2.19.4).
    unsafe { core::ptr::write_volatile(address as *mut u32, value) }
}

/// Write a pin's pad register (drive strength, pulls, input/output enable).
///
/// The whole register is written on purpose: that also clears the RP2350 ISO bit
/// (bit 8) of a pad a bootloader left isolated, which is what esprit's C++ does as
/// well (`lnPads->PADS[pin] &= 0xff` before writing).
fn write_pad(pin: u32, pad: u32) {
    let address = PADS_BANK0_BASE + PAD_FIRST_OFFSET + pin * PAD_STRIDE;
    // SAFETY: one 4-byte pad register per pin, GPIO0 at +0x04 (datasheet 2.19.6).
    unsafe { core::ptr::write_volatile(address as *mut u32, pad) }
}


/// One RP2040 / RP2350 GPIO, in the encoding of the C `lnPin` enum
/// (`esprit/mcus/arm_rp2040/include/lnGPIO_pins.h`): a GPIO number *is* the pin,
/// plus the two sentinels that enum carries.
#[repr(i32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Pin {
    GPIO0 = 0,
    GPIO1 = 1,
    GPIO2 = 2,
    GPIO3 = 3,
    GPIO4 = 4,
    GPIO5 = 5,
    GPIO6 = 6,
    GPIO7 = 7,
    GPIO8 = 8,
    GPIO9 = 9,
    GPIO10 = 10,
    GPIO11 = 11,
    GPIO12 = 12,
    GPIO13 = 13,
    GPIO14 = 14,
    GPIO15 = 15,
    GPIO16 = 16,
    GPIO17 = 17,
    GPIO18 = 18,
    GPIO19 = 19,
    GPIO20 = 20,
    GPIO21 = 21,
    GPIO22 = 22,
    GPIO23 = 23,
    GPIO24 = 24,
    GPIO25 = 25,
    GPIO26 = 26,
    GPIO27 = 27,
    GPIO28 = 28,
    GPIO29 = 29,
    /// The C enum's 30th entry (`PPINS(99)`), not a real pad.
    GPIO99 = 99,
    /// The C enum's "no pin" sentinel.
    NoPin = -1,
}

impl From<u32> for Pin {
    /// Mirror of the C `(lnPin)number` cast, used where a pin arrives as a plain
    /// number (the probe board table). Only the declared values are valid.
    fn from(value: u32) -> Self {
        // SAFETY: `Pin` is a `repr(i32)` enum; callers pass a board-table entry
        // that was range-checked at build time.
        unsafe { core::mem::transmute::<u32, Pin>(value) }
    }
}

/// Pad bits for a pin that drives: 12 mA, fast slew, Schmitt on, input readable.
/// (The C++ sets 12 mA + fast slew and leaves the input disabled; the input is
/// enabled here so `read()` on an output pin reports the real level.)
const PAD_OUTPUT: u32 = (PAD_DRIVE_12MA << PAD_DRIVE_LSB) | PAD_SLEWFAST | PAD_SCHMITT | PAD_IE;
/// Pad bits for a pin whose driver must be off (input, or open drain released).
const PAD_HIGH_Z: u32 = PAD_OD | PAD_SLEWFAST | PAD_SCHMITT;

/// Emulated open drain, the same sequence as esprit's C++ `lnOpenDrainClose`:
/// the latch is pulled to 0 and the *driver* is what opens or closes, so the pin
/// is either driven low (`close`) or released to its pull-up.

/// Configure the pad for `mode` (the hardware half of [`GpioPin::set_mode`]).
///
///   * `Output`            → FUNCSEL = SIO, output enable forced on, 12 mA
///   * `OutputOpenDrain`   → FUNCSEL = SIO, latch 0, pad output disabled (Hi-Z);
///     [`GpioPin::open_drain_close`]`(true)` then enables the driver (line low)
///     and `(false)` disables it again (the external pull-up wins)
///   * `InputFloating` / `InputPullUp` / `InputPullDown` → SIO, input enabled,
///     driver disabled, pull as asked
///   * `Floating`          → everything off (pad high impedance)
///   * `Adc` / `Dac`       → FUNCSEL = NULL, input disabled (the RP2040 has no
///     DAC; `Dac` is mapped like `Adc` so it cannot silently drive a pad)
///   * `Uart` / `Spi` / `Pwm` → peripheral instance 0 (F2 / F1 / F4), which is
///     what esprit's C++ `lnPinMode` hardcodes
fn apply_mode(pin: u32, mode: GpioMode) {
    match mode {
        GpioMode::Floating => {
            write_pad(pin, PAD_HIGH_Z);
            write_ctrl(pin, FUNCSEL_SIO, GPIO_CTRL_OE_DISABLE);
        }
        GpioMode::InputFloating => {
            write_pad(pin, PAD_HIGH_Z | PAD_IE);
            write_ctrl(pin, FUNCSEL_SIO, GPIO_CTRL_OE_DISABLE);
        }
        GpioMode::InputPullUp => {
            write_pad(pin, PAD_HIGH_Z | PAD_IE | PAD_PUE);
            write_ctrl(pin, FUNCSEL_SIO, GPIO_CTRL_OE_DISABLE);
        }
        GpioMode::InputPullDown => {
            write_pad(pin, PAD_HIGH_Z | PAD_IE | PAD_PDE);
            write_ctrl(pin, FUNCSEL_SIO, GPIO_CTRL_OE_DISABLE);
        }
        GpioMode::Output => {
            write_pad(pin, PAD_OUTPUT);
            write_ctrl(pin, FUNCSEL_SIO, GPIO_CTRL_OE_ENABLE);
        }
        // Start released; `open_drain_close` closes or opens the drain.
        GpioMode::OutputOpenDrain => open_drain_close(pin, false),
        GpioMode::Adc | GpioMode::Dac => {
            write_pad(pin, PAD_OD | PAD_SLEWFAST | PAD_SCHMITT);
            write_ctrl(pin, FUNCSEL_NULL, GPIO_CTRL_OE_DISABLE);
        }
        GpioMode::Spi => {
            write_pad(pin, PAD_OUTPUT);
            write_ctrl(pin, FUNCSEL_SPI0, GPIO_CTRL_OE_NORMAL);
        }
        GpioMode::Uart | GpioMode::UartAlt => {
            write_pad(pin, PAD_OUTPUT);
            write_ctrl(pin, FUNCSEL_UART0, GPIO_CTRL_OE_NORMAL);
        }
        GpioMode::Pwm => {
            write_pad(pin, PAD_OUTPUT);
            write_ctrl(pin, FUNCSEL_PWM0, GPIO_CTRL_OE_NORMAL);
        }
        // RP2xxx has no open-drain alternate function (a peripheral's pad driver is
        // push-pull): leave the pin unmuxed, mux it explicitly with `set_function`.
        GpioMode::AlternatePushPull | GpioMode::AlternateOpenDrain => {
            write_ctrl(pin, FUNCSEL_NULL, GPIO_CTRL_OE_NORMAL)
        }
    }
}

impl Pin {
    /// This pin's number, `0` … `29`.
    pub fn number(self) -> u32 {
        self as u32
    }

    /// Route the pin to an arbitrary FUNCSEL (an alternate function [`GpioMode`]
    /// cannot name). The output enable follows the peripheral.
    pub fn set_function(&mut self, funcsel: u32) {
        write_ctrl(self.number(), funcsel, GPIO_CTRL_OE_NORMAL);
    }
}

impl GpioPin for Pin {
    fn write(&mut self, value: bool) {
        drive(self.number(), value);
    }

    fn read(&self) -> bool {
        sense(self.number())
    }

    fn toggle(&mut self) {
        flip(self.number());
    }

    fn set_mode(&mut self, mode: GpioMode) {
        apply_mode(self.number(), mode);
    }

    /// RP2xxx pads have no per-pin clock speed: the electrical knob is the drive
    /// strength, already set to 12 mA by [`GpioPin::set_mode`]. The speed is
    /// therefore ignored, exactly like esprit's C++ `lnPinMode` ignores it.
    fn set_mode_speed(&mut self, mode: GpioMode, _speed_mhz: u32) {
        apply_mode(self.number(), mode);
    }

    fn open_drain_close(&mut self, close: bool) {
        open_drain_close(self.number(), close);
    }

    /// GPIO interrupts are not routed yet: that needs IO_BANK0 `CPU0_INTE` plus a
    /// handler table, and no caller of this driver needs them today.
    fn enable_interrupt(&mut self, _trigger: EdgeTrigger) -> bool {
        false
    }

    fn set_handler(&mut self, _handler: Option<&'static dyn GpioInterruptHandler>) {}
}

fn open_drain_close(pin: u32, close: bool) {
    drive(pin, false); // the drain can only pull down
    if close {
        write_pad(pin, PAD_OUTPUT); // driver on, line low
        write_ctrl(pin, FUNCSEL_SIO, GPIO_CTRL_OE_ENABLE);
    } else {
        write_pad(pin, PAD_HIGH_Z); // driver off, external pull-up wins
        write_ctrl(pin, FUNCSEL_SIO, GPIO_CTRL_OE_DISABLE);
    }
}
