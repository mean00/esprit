//! The **RP2040** half of the ADC map: the analog-mux fields of `CS` and the pin
//! → channel table.
//!
//! Numbers are the RP2040 datasheet §4.9.2.1 / §4.9.4 Table 568 and §2.5.3.1
//! Table 119 (DREQs). Selected by `lib.rs`; everything the two chips share lives
//! in `registers.rs`.
#![allow(dead_code)]

// --- CS.AINSEL / CS.RROBIN (Table 568) ---------------------------------------
//
// The RP2040 offers four external inputs plus the temperature sensor, so the mux
// select is 3 bits and the round-robin mask 5 bits. The RP2350 widened both (see
// `registers_rp2350.rs`), which is why they cannot live in the shared file.

pub const ADC_CS_AINSEL_LSB: u32 = 12;
pub const ADC_CS_AINSEL_MASK: u32 = 0x7 << ADC_CS_AINSEL_LSB;
pub const ADC_CS_RROBIN_LSB: u32 = 16;
pub const ADC_CS_RROBIN_MASK: u32 = 0x1f << ADC_CS_RROBIN_LSB;

/// AINSEL of the internal temperature sensor (§4.9.2.1).
pub const ADC_TS_CHANNEL: u32 = 4;

/// External inputs, i.e. how many channels a round-robin mask can hold.
pub const ADC_INPUT_COUNT: u32 = 4;

/// ADC channel of a GPIO number, in the order the hardware mux numbers them:
/// GPIO26 → 0 … GPIO29 → 3 (§4.9.2.1). Everything else is a digital-only pad.
pub const fn pin_channel(pin: u32) -> Option<u32> {
    if pin >= 26 && pin <= 29 {
        Some(pin - 26)
    } else {
        None
    }
}

// --- DMA data request (Table 119) --------------------------------------------

/// `DREQ_ADC`: the ADC FIFO's data request, one per sample in the FIFO.
pub const DREQ_ADC: u32 = 36;
