//! RCU values that only exist on the **CH32V3** side of the family.
//!
//! Bits 9..12 of `AHBENR` are this family's own (the C header spells the same
//! constants `LN_RCU_AHB_*_CH32V3x`); the ARM bluepill parts leave them reserved,
//! which is why `registers_stm32.rs` answers `None` here instead of a bit number.

/// `AHBENR` bit of the high-speed USB controller.
pub const AHB_USBHS: Option<u32> = Some(1 << 11);
/// `AHBENR` bit of the USB full-speed OTG controller.
pub const AHB_USBFS_OTG: Option<u32> = Some(1 << 12);
/// `AHBENR` bit of the true random number generator.
pub const AHB_RNG: Option<u32> = Some(1 << 9);
/// `AHBENR` bit of the SDIO controller.
pub const AHB_SDIO: Option<u32> = Some(1 << 10);
