//! The **RP2350** half of the ADC map: the wider analog-mux fields of `CS`, the
//! pin → channel table and the DMA request number.
//!
//! Numbers are the RP2350 datasheet §12.4.3 / §12.4.7 Table 1118 and §12.6.4.1
//! Table 1143 (DREQs). Selected by `lib.rs`; everything the two chips share lives
//! in `registers.rs`.
#![allow(dead_code)]

// --- CS.AINSEL / CS.RROBIN (Table 1118) --------------------------------------
//
// The RP2350 has eight external inputs and a ninth channel for the temperature
// sensor, so AINSEL is 4 bits wide and RROBIN 9 (RP2040: 3 and 5). AINSEL is
// "corrected for the package option", i.e. the hardware maps the values to the
// bonded-out inputs for you.

pub const ADC_CS_AINSEL_LSB: u32 = 12;
pub const ADC_CS_AINSEL_MASK: u32 = 0xf << ADC_CS_AINSEL_LSB;
pub const ADC_CS_RROBIN_LSB: u32 = 16;
pub const ADC_CS_RROBIN_MASK: u32 = 0x1ff << ADC_CS_RROBIN_LSB;

/// AINSEL of the internal temperature sensor (§12.4.3.1), on QFN-80.
pub const ADC_TS_CHANNEL: u32 = 8;

/// External inputs of the widest package (QFN-80): eight channels.
pub const ADC_INPUT_COUNT: u32 = 8;

/// ADC channel of a GPIO number (§12.4.3.1).
///
/// The mapping follows the package: on QFN-60 (and QFN-56) the four inputs are
/// GPIO26 … GPIO29, on QFN-80 the eight inputs are GPIO40 … GPIO47. Both are in
/// the same table because the pin a board hands over tells which one it is: a pad
/// that is not bonded out has no ADC input, and `Pin::from_number` refuses it
/// before this point.
pub const fn pin_channel(pin: u32) -> Option<u32> {
    if pin >= 26 && pin <= 29 {
        Some(pin - 26)
    } else if pin >= 40 && pin <= 47 {
        Some(pin - 40)
    } else {
        None
    }
}

// --- DMA data request (Table 1143) -------------------------------------------
//
// PIO2 and the four extra PWM wraparounds sit in front of the serial peripherals
// on this chip, so every DREQ from SPI0 on is higher than on the RP2040
// (SPI0_TX is 16 there, 24 here — see rs_dma_rp2xxx).

/// `DREQ_ADC`: the ADC FIFO's data request, one per sample in the FIFO.
pub const DREQ_ADC: u32 = 48;
