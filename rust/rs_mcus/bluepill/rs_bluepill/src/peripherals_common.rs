//! Peripheral addresses that are **the same on every bluepill-family part** —
//! STM32F1, GD32F30x and CH32V3.
//!
//! Source of truth: `esprit/private_include/lnPeripheral_priv_stm32.h` and
//! `lnPeripheral_priv_ch32v3x.h`. Everything in this file appears in *both*, at
//! the same address; what only one family has lives in `peripherals_stm32.rs` /
//! `peripherals_ch32v3x.rs`.
//!
//! Naming note: the numbers are GD32/CH32 style (`TIMER1` is the STM32 `TIM2`,
//! `SPI1` is the STM32 `SPI2`), which is what the C side uses too.

// --- GPIO ---------------------------------------------------------------------

pub const GPIOA_BASE: u32 = 0x4001_0800;
pub const GPIOB_BASE: u32 = 0x4001_0C00;
pub const GPIOC_BASE: u32 = 0x4001_1000;
pub const GPIOD_BASE: u32 = 0x4001_1400;
pub const GPIOE_BASE: u32 = 0x4001_1800;
/// GPIOF / GPIOG exist on the bigger packages of both families (the C side has no
/// entry for them; the addresses are the STM32F1 high-density ones).
pub const GPIOF_BASE: u32 = 0x4001_1C00;
pub const GPIOG_BASE: u32 = 0x4001_2000;

// --- Clock & reset (RCU / RCC) -------------------------------------------------

pub const RCU_BASE: u32 = 0x4002_1000;

// --- Alternate function I/O and EXTI ------------------------------------------

pub const AFIO_BASE: u32 = 0x4001_0000;
pub const EXTI_BASE: u32 = 0x4001_0400;

// --- ADC (the first two; a third exists only on the ARM side) ------------------

pub const ADC1_BASE: u32 = 0x4001_2400;
pub const ADC2_BASE: u32 = 0x4001_2800;

/// Register block of ADC `instance` (`0` = ADC1, `1` = ADC2, …).
///
/// A lookup rather than a bare constant because the two families do not have the
/// same number of ADCs: the chip file decides how many. `None` means the instance
/// does not exist on this part.
pub use super::peripherals_chip::adc_base;

// --- Timers (TIMER0..TIMER6 on both; the numbering above that differs) --------

pub const TIM1_BASE: u32 = 0x4001_2C00; // TIMER0 (advanced)
pub const TIM2_BASE: u32 = 0x4000_0000; // TIMER1
pub const TIM3_BASE: u32 = 0x4000_0400; // TIMER2
pub const TIM4_BASE: u32 = 0x4000_0800; // TIMER3
pub const TIM5_BASE: u32 = 0x4000_0C00; // TIMER4
pub const TIM6_BASE: u32 = 0x4000_1000; // TIMER5 (basic)
pub const TIM7_BASE: u32 = 0x4000_1400; // TIMER6 (basic)

/// The 8th timer. The address is the same on both families — `TIM8` on the ARM
/// side, `TIMER7` on CH32V3 — so it is shared, under the STM32-flavoured name the
/// timer driver uses. The CH32-only `TIMER8`/`TIMER9` are in the chip file.
pub const TIM8_BASE: u32 = 0x4001_3400;

// --- DMA ----------------------------------------------------------------------

pub const DMA0_BASE: u32 = 0x4002_0000;
pub const DMA1_BASE: u32 = 0x4002_0400;

// --- SPI / I2C / USART --------------------------------------------------------

pub const SPI1_BASE: u32 = 0x4001_3000; // SPI0
pub const SPI2_BASE: u32 = 0x4000_3800; // SPI1
pub const SPI3_BASE: u32 = 0x4000_3C00; // SPI2

pub const I2C1_BASE: u32 = 0x4000_5400; // I2C0
pub const I2C2_BASE: u32 = 0x4000_5800; // I2C1

pub const USART1_BASE: u32 = 0x4001_3800; // USART0
pub const USART2_BASE: u32 = 0x4000_4400; // USART1
pub const USART3_BASE: u32 = 0x4000_4800; // USART2
pub const USART4_BASE: u32 = 0x4000_4C00; // USART3
pub const USART5_BASE: u32 = 0x4000_5000; // USART4

// --- USB device (full-speed, the one swindle uses) -----------------------------

pub const USBD0_BASE: u32 = 0x4000_5C00;
pub const USBD0_RAM_BASE: u32 = 0x4000_6000;
