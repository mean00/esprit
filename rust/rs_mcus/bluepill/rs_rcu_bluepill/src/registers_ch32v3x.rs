//! RCU values that only exist on the **CH32V3** side of the family.
//!
//! Bits 9..12 of `AHBENR` are this family's own (the C header spells the same
//! constants `LN_RCU_AHB_*_CH32V3x`); the ARM bluepill parts leave them reserved,
//! which is why `registers_stm32.rs` answers `None` here instead of a bit number.

#![allow(dead_code)]

/// `AHBENR` bit of the high-speed USB controller.
pub const AHB_USBHS: Option<u32> = Some(1 << 11);
/// `AHBENR` bit of the USB full-speed OTG controller.
pub const AHB_USBFS_OTG: Option<u32> = Some(1 << 12);
/// `AHBENR` bit of the true random number generator.
pub const AHB_RNG: Option<u32> = Some(1 << 9);
/// `AHBENR` bit of the SDIO controller.
pub const AHB_SDIO: Option<u32> = Some(1 << 10);

// --- EXTEN_CTR bits ---
pub const CH32_EXTEN_CTR_USBD_PULLUP_ENABLE: u32 = 1 << 1;
pub const CH32_EXTEN_CTR_ETH_10M_ENABLE: u32 = 1 << 2;

// --- CFG1 USBHS PHY bits ---
pub const CH32V3_USBHS_CFG1_KEEP_MASK: u32 = 0x00FF_FFFF;
pub const CH32V3_USBHS_PREFIX_DIV2: u32 = 1 << 24;
pub const CH32V3_USBHS_PLL_SRC_HSE: u32 = 1 << 27;
pub const CH32V3_USBHS_CKREF_SEL_4MHZ: u32 = 1 << 28;
pub const CH32V3_USBHS_USB_PHY_ENABLED: u32 = 1 << 30;
pub const CH32V3_USBHS_CKREF_SRC_USB_PHY: u32 = 1 << 31;

// --- CFG0 CH32V3x clock fields ---
pub const CH32_CFG0_AHB_MASK: u32 = 0xF << 4;
pub const CH32_CFG0_APB1_MASK: u32 = 7 << 8;
pub const CH32_CFG0_APB2_MASK: u32 = 7 << 11;
pub const CH32_CFG0_PLL_EXTERNAL: u32 = 1 << 16;
pub const CH32_CFG0_PLL_MASK: u32 = 0xF << 18;
