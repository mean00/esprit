#![allow(unused_variables, unused_imports, unused_parens)]
#![allow(non_camel_case_types, non_snake_case, dead_code)]

#![no_std]

//! Hardware memory map and register structures for STM32F1 / GD32F3 / CH32V3 compatible MCUs.

// --- Base Addresses ---

// GPIO
pub const GPIOA_BASE: u32 = 0x4001_0800;
pub const GPIOB_BASE: u32 = 0x4001_0C00;
pub const GPIOC_BASE: u32 = 0x4001_1000;
pub const GPIOD_BASE: u32 = 0x4001_1400;
pub const GPIOE_BASE: u32 = 0x4001_1800;
pub const GPIOF_BASE: u32 = 0x4001_1C00;
pub const GPIOG_BASE: u32 = 0x4001_2000;

// Clock & Reset Control (RCU/RCC)
pub const RCU_BASE: u32 = 0x4002_1000;

// Alternate Function I/O (AFIO)
pub const AFIO_BASE: u32 = 0x4001_0000;

// EXTI
pub const EXTI_BASE: u32 = 0x4001_0400;

// ADC
pub const ADC1_BASE: u32 = 0x4001_2400;
pub const ADC2_BASE: u32 = 0x4001_2800;
pub const ADC3_BASE: u32 = 0x4001_3C00;

// Timers
pub const TIM1_BASE: u32 = 0x4001_2C00;
pub const TIM2_BASE: u32 = 0x4000_0000;
pub const TIM3_BASE: u32 = 0x4000_0400;
pub const TIM4_BASE: u32 = 0x4000_0800;
pub const TIM5_BASE: u32 = 0x4000_0C00;
pub const TIM6_BASE: u32 = 0x4000_1000;
pub const TIM7_BASE: u32 = 0x4000_1400;
pub const TIM8_BASE: u32 = 0x4001_3400;

// End of PAC
pub const DMA0_BASE: u32 = 0x4002_0000;
pub const DMA1_BASE: u32 = 0x4002_0400;
pub const SPI1_BASE: u32 = 0x4001_3000;
pub const SPI2_BASE: u32 = 0x4000_3800;
pub const SPI3_BASE: u32 = 0x4000_3C00;

pub const I2C1_BASE: u32 = 0x4000_5400;
pub const I2C2_BASE: u32 = 0x4000_5800;
