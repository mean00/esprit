#![no_std]

//! Pure-Rust GPIO for the RP2040 and the RP2350: the `rs_mcu_traits::gpio::Gpio`
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
use rs_mcu_traits::gpio::{self, PinMode};
use rs_mcu_traits::Result;

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

/// Pad bits for a pin that drives: 12 mA, fast slew, Schmitt on, input readable.
/// (The C++ sets 12 mA + fast slew and leaves the input disabled; the input is
/// enabled here so `read()` on an output pin reports the real level.)
const PAD_OUTPUT: u32 = (PAD_DRIVE_12MA << PAD_DRIVE_LSB) | PAD_SLEWFAST | PAD_SCHMITT | PAD_IE;
/// Pad bits for a pin whose driver must be off (input, or open drain released).
const PAD_HIGH_Z: u32 = PAD_OD | PAD_SLEWFAST | PAD_SCHMITT;

/// Configure the pad for `mode` (the hardware half of [`gpio::Gpio::set_mode`]).
///
///   * `Output`            → FUNCSEL = SIO, output enable forced on, 12 mA
///   * `OutputOpenDrain`   → FUNCSEL = SIO, latch 0, pad output disabled (Hi-Z);
///     [`gpio::Gpio::open_drain`]`(true)` then enables the driver (line low)
///     and `(false)` disables it again (the external pull-up wins)
///   * `Input` / `InputPullUp` / `InputPullDown` → SIO, input enabled, driver
///     disabled, pull as asked
///   * `Analog`            → FUNCSEL = NULL, input disabled (the RP2040 has no
///     DAC): the pad must not drive the analog input it shares
///   * `AlternatePushPull` / `AlternateOpenDrain` → FUNCSEL = NULL, output
///     enable follows the peripheral. The RP2xxx has no open-drain alternate
///     function, and which peripheral is on the pad is a *family* decision, so
///     the driver muxes it explicitly with [`Pin::set_function`] (F1 SPI, F2
///     UART, F4 PWM…).
fn apply_mode(pin: u32, mode: PinMode) {
    match mode {
        PinMode::Input => {
            write_pad(pin, PAD_HIGH_Z | PAD_IE);
            write_ctrl(pin, FUNCSEL_SIO, GPIO_CTRL_OE_DISABLE);
        }
        PinMode::InputPullUp => {
            write_pad(pin, PAD_HIGH_Z | PAD_IE | PAD_PUE);
            write_ctrl(pin, FUNCSEL_SIO, GPIO_CTRL_OE_DISABLE);
        }
        PinMode::InputPullDown => {
            write_pad(pin, PAD_HIGH_Z | PAD_IE | PAD_PDE);
            write_ctrl(pin, FUNCSEL_SIO, GPIO_CTRL_OE_DISABLE);
        }
        PinMode::Output => {
            write_pad(pin, PAD_OUTPUT);
            write_ctrl(pin, FUNCSEL_SIO, GPIO_CTRL_OE_ENABLE);
        }
        // Start released; `open_drain` closes or opens the drain.
        PinMode::OutputOpenDrain => open_drain_close(pin, false),
        PinMode::Analog => {
            write_pad(pin, PAD_OD | PAD_SLEWFAST | PAD_SCHMITT);
            write_ctrl(pin, FUNCSEL_NULL, GPIO_CTRL_OE_DISABLE);
        }
        PinMode::AlternatePushPull | PinMode::AlternateOpenDrain => {
            write_ctrl(pin, FUNCSEL_NULL, GPIO_CTRL_OE_NORMAL)
        }
    }
}

impl Pin {
    /// The pad with GPIO number `number` (`0` … `29`), or `None` for anything
    /// else — including the two sentinels [`Pin::GPIO99`] and [`Pin::NoPin`],
    /// which name no pad.
    ///
    /// This is the checked replacement for the `(lnPin)number` cast: a
    /// `#[repr(i32)]` enum built from an out-of-range integer is undefined
    /// behaviour, and a board table or a C caller can always hand one over. It is
    /// a `const fn`, so a board table can turn its pin *number* into a [`Pin`] at
    /// compile time and let a bad entry fail the build:
    ///
    /// ```ignore
    /// const NRST: Pin = match Pin::from_number(BOARD.dev[DEV_NRST] as u32) {
    ///     Some(pin) => pin,
    ///     None => panic!("BOARD.dev[DEV_NRST] is not a real RP2xxx pad"),
    /// };
    /// ```
    pub const fn from_number(number: u32) -> Option<Self> {
        match number {
            0 => Some(Self::GPIO0),
            1 => Some(Self::GPIO1),
            2 => Some(Self::GPIO2),
            3 => Some(Self::GPIO3),
            4 => Some(Self::GPIO4),
            5 => Some(Self::GPIO5),
            6 => Some(Self::GPIO6),
            7 => Some(Self::GPIO7),
            8 => Some(Self::GPIO8),
            9 => Some(Self::GPIO9),
            10 => Some(Self::GPIO10),
            11 => Some(Self::GPIO11),
            12 => Some(Self::GPIO12),
            13 => Some(Self::GPIO13),
            14 => Some(Self::GPIO14),
            15 => Some(Self::GPIO15),
            16 => Some(Self::GPIO16),
            17 => Some(Self::GPIO17),
            18 => Some(Self::GPIO18),
            19 => Some(Self::GPIO19),
            20 => Some(Self::GPIO20),
            21 => Some(Self::GPIO21),
            22 => Some(Self::GPIO22),
            23 => Some(Self::GPIO23),
            24 => Some(Self::GPIO24),
            25 => Some(Self::GPIO25),
            26 => Some(Self::GPIO26),
            27 => Some(Self::GPIO27),
            28 => Some(Self::GPIO28),
            29 => Some(Self::GPIO29),
            _ => None,
        }
    }

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

/// One handler slot per pad (the RP2xxx IO_BANK0 interrupt is per pin).
impl gpio::Gpio for Pin {
    fn pin_number(&self) -> u32 {
        self.number()
    }

    fn write(&mut self, high: bool) {
        drive(self.number(), high);
    }

    fn read(&self) -> bool {
        sense(self.number())
    }

    fn toggle(&mut self) {
        flip(self.number());
    }

    fn set_mode(&mut self, mode: PinMode) -> Result<()> {
        apply_mode(self.number(), mode);
        Ok(())
    }

    /// RP2xxx pads have no per-pin clock speed: the electrical knob is the drive
    /// strength, already set to 12 mA by [`gpio::Gpio::set_mode`]. The speed is
    /// therefore ignored.
    fn set_mode_speed(&mut self, mode: PinMode, _speed_mhz: u32) -> Result<()> {
        apply_mode(self.number(), mode);
        Ok(())
    }

    fn open_drain(&mut self, drive_low: bool) -> Result<()> {
        open_drain_close(self.number(), drive_low);
        Ok(())
    }
}

/// Emulated open drain, the same sequence as esprit's C++ `lnOpenDrainClose`:
/// the latch is pulled to 0 and the *driver* is what opens or closes, so the pin
/// is either driven low (`close`) or released to its pull-up.
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Every real pad survives the round trip `number` -> `Pin` -> `number`.
    #[test]
    fn from_number_accepts_every_real_pad() {
        for number in 0u32..=29 {
            let pin = Pin::from_number(number).expect("GPIO0..GPIO29 are real pads");
            assert_eq!(pin.number(), number);
        }
    }

    /// Everything else is refused instead of becoming an invalid enum value:
    /// 30..=98, the `GPIO99` sentinel, and the huge `u32` a negative board-table
    /// entry turns into (`-1` is the C `NoPin` sentinel).
    #[test]
    fn from_number_rejects_anything_that_is_not_a_pad() {
        for number in [30u32, 98, Pin::GPIO99 as u32, u32::MAX] {
            assert!(Pin::from_number(number).is_none(), "{number} is not a pad");
        }
        assert!(Pin::from_number(Pin::NoPin as i32 as u32).is_none());
    }
}

