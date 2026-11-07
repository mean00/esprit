//! Register map and field layout of the RP2040 GPIO blocks: SIO (the GPIO
//! function of the processor subsystem), IO_BANK0 (per-pin control) and
//! PADS_BANK0 (per-pin electricals).
//!
//! Numbers are the ones in `rp2040.md` (§2.3.1.7 for the SIO list, §2.19.2
//! Table 279 for the function select, §2.19.4/5 for IO_BANK0 and §2.19.6 for
//! PADS_BANK0). The RP2350 has the same layout for every field used here; its
//! pads add an ISO bit (bit 8) that a full-register write clears, which is what
//! esprit's C++ does too (`lnPads->PADS[pin] &= 0xff` before writing).
//!
//! The map is complete on purpose — a driver only uses part of it.
#![allow(dead_code)]

// --- Bases (datasheet §2.2 "Address map") ---------------------------------

/// Single-cycle I/O block: the GPIO function of the CPU.
pub const SIO_BASE: u32 = 0xd000_0000;
/// Per-GPIO function select and overrides.
pub const IO_BANK0_BASE: u32 = 0x4001_4000;
/// Per-GPIO pad control (drive strength, pulls, input/output enable).
pub const PADS_BANK0_BASE: u32 = 0x4001_c000;

// --- SIO registers (offsets from SIO_BASE) --------------------------------
// The SET/CLR/XOR registers are write-1-to-act and cannot be read back: they are
// the only ones this driver writes, so a read-modify-write race is impossible.

pub const SIO_GPIO_IN: u32 = 0x004;
pub const SIO_GPIO_OUT: u32 = 0x010;
pub const SIO_GPIO_OUT_SET: u32 = 0x014;
pub const SIO_GPIO_OUT_CLR: u32 = 0x018;
pub const SIO_GPIO_OUT_XOR: u32 = 0x01c;
pub const SIO_GPIO_OE: u32 = 0x020;
pub const SIO_GPIO_OE_SET: u32 = 0x024;
pub const SIO_GPIO_OE_CLR: u32 = 0x028;
pub const SIO_GPIO_OE_XOR: u32 = 0x02c;

// --- IO_BANK0 (per pin) ---------------------------------------------------

/// Every pin has an 8 byte block: STATUS then CTRL.
pub const GPIO_BLOCK_STRIDE: u32 = 8;
/// Offset of CTRL inside a pin's block (STATUS is at 0x000).
pub const GPIO_CTRL_OFFSET: u32 = 0x004;

/// FUNCSEL field: the function this pin is driven by. `0x1f` is NULL.
pub const GPIO_CTRL_FUNCSEL_LSB: u32 = 0;
pub const GPIO_CTRL_FUNCSEL_MASK: u32 = 0x1f;
/// OEOVER field: who owns the output enable.
pub const GPIO_CTRL_OE_LSB: u32 = 12;
pub const GPIO_CTRL_OE_MASK: u32 = 0x3;

/// Output enable follows the peripheral (what a muxed pin wants).
pub const GPIO_CTRL_OE_NORMAL: u32 = 0;
/// Output enable is the inverse of the peripheral's.
pub const GPIO_CTRL_OE_INVERTED: u32 = 1;
/// Output enable forced off: the pad driver is high impedance.
pub const GPIO_CTRL_OE_DISABLE: u32 = 2;
/// Output enable forced on.
pub const GPIO_CTRL_OE_ENABLE: u32 = 3;

/// FUNCSEL value meaning "not muxed to anything".
pub const FUNCSEL_NULL: u32 = 0x1f;
/// SIO = function 5, on every pin (datasheet Table 279).
pub const FUNCSEL_SIO: u32 = 5;
/// SPI0 = F1, UART0 = F2, I2C0 = F3, PWM0 = F4: the F number names the
/// peripheral *instance*, so it is the same on every pin that can reach it.
/// (Same three esprit's C++ `lnPinMode` hardcodes, i.e. instance 0 only.)
pub const FUNCSEL_SPI0: u32 = 1;
pub const FUNCSEL_UART0: u32 = 2;
pub const FUNCSEL_PWM0: u32 = 4;

// --- PADS_BANK0 (per pin) -------------------------------------------------

/// VOLTAGE_SELECT is at 0x00, the pads follow one register each.
pub const PAD_FIRST_OFFSET: u32 = 0x04;
pub const PAD_STRIDE: u32 = 4;

/// Fast slew rate (`1` = fast, `0` = slow).
pub const PAD_SLEWFAST: u32 = 1 << 0;
/// Schmitt trigger enable (the pad reset state has it on).
pub const PAD_SCHMITT: u32 = 1 << 1;
/// Pull down enable.
pub const PAD_PDE: u32 = 1 << 2;
/// Pull up enable.
pub const PAD_PUE: u32 = 1 << 3;
/// Drive strength, two bits.
pub const PAD_DRIVE_LSB: u32 = 4;
pub const PAD_DRIVE_2MA: u32 = 0;
pub const PAD_DRIVE_4MA: u32 = 1;
pub const PAD_DRIVE_8MA: u32 = 2;
pub const PAD_DRIVE_12MA: u32 = 3;
/// Input enable: the pad's value reaches the input synchroniser.
pub const PAD_IE: u32 = 1 << 6;
/// Output disable: has priority over every output enable, so the pad floats.
pub const PAD_OD: u32 = 1 << 7;
