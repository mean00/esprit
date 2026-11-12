//! RP2040 numbers for the GPIO interrupts (datasheet §2.19.6, Table 300).
//!
//! `INTR0` is at IO_BANK0 + 0x0f0, four words (8 pads each) cover the 30 pads,
//! and the shared line is NVIC interrupt 13 (DMA_IRQ_0 = 11, DMA_IRQ_1 = 12,
//! IO_IRQ_BANK0 = 13 — RP2040 datasheet §2.3.2, Table 24).

/// Offset of `INTR0` inside IO_BANK0.
pub const INTR_OFFSET: u32 = 0x0f0;
/// Words per interrupt array: 4 → 32 slots, of which 30 are real pads.
pub const IRQ_WORDS: usize = 4;
/// Pads with an interrupt (pico-sdk `NUM_BANK0_GPIOS`).
pub const GPIO_COUNT: usize = 30;
/// NVIC interrupt of the shared GPIO line.
pub const NVIC_IO_IRQ_BANK0: u32 = 13;
