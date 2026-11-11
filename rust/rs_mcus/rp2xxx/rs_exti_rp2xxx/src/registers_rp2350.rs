//! RP2350 numbers for the GPIO interrupts (datasheet §9.11.2, Tables 690-697).
//!
//! `INTR0` is at IO_BANK0 + 0x230, six words cover the 48 pads, and the shared
//! line is NVIC interrupt 21 (RP2350 datasheet §3.2: PWM_IRQ_WRAP_0/1 = 8/9,
//! DMA_IRQ_0..3 = 10..13, USBCTRL_IRQ = 14, PIO0..PIO2 = 15..20, IO_IRQ_BANK0 =
//! 21, IO_IRQ_BANK0_NS = 22). Note that 13 is *not* the GPIO line here — it is
//! DMA_IRQ_3.

/// Offset of `INTR0` inside IO_BANK0.
pub const INTR_OFFSET: u32 = 0x230;
/// Words per interrupt array: 6 → 48 slots, one per pad.
pub const IRQ_WORDS: usize = 6;
/// Pads with an interrupt.
pub const GPIO_COUNT: usize = 48;
/// NVIC interrupt of the shared GPIO line.
pub const NVIC_IO_IRQ_BANK0: u32 = 21;
