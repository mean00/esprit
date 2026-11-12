//! RCU values that only exist on the **CH32V3** side of the family, and the `None`
//! these ARM parts answer instead.
//!
//! Bits 9..12 of `AHBENR` are the CH32V3's own (the C header spells the same
//! constants `LN_RCU_AHB_*_CH32V3x`); GD32F30x / STM32F1 leave them reserved, so
//! asking for one of those peripherals here is a no-op rather than a write to a
//! bit that means something else.

/// `AHBENR` bit of the high-speed USB controller: absent on this family.
pub const AHB_USBHS: Option<u32> = None;
/// `AHBENR` bit of the USB full-speed OTG controller: absent on this family (the
/// plain USB full-speed device is `RCU_AHB_USBFS`).
pub const AHB_USBFS_OTG: Option<u32> = None;
/// `AHBENR` bit of the true random number generator: absent.
pub const AHB_RNG: Option<u32> = None;
/// `AHBENR` bit of the SDIO controller: absent.
pub const AHB_SDIO: Option<u32> = None;
