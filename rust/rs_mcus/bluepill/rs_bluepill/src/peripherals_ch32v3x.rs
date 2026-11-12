//! Addresses that exist only on **CH32V3** —
//! `esprit/private_include/lnPeripheral_priv_ch32v3x.h`.
//!
//! Everything both families share is in `peripherals_common.rs`; the ARM-only
//! blocks are in `peripherals_stm32.rs`.

/// USART5..7, the three extra serial ports (STM32F1/GD32F30x stop at USART4).
pub const USART5_BASE: u32 = 0x4000_1800;
pub const USART6_BASE: u32 = 0x4000_1C00;
pub const USART7_BASE: u32 = 0x4000_2000;

/// The advanced timers above `TIMER6`. Note that `TIMER7_BASE` is the address the
/// ARM side calls `TIM8_BASE` — same base, different timer, so the shared
/// `TIM8_BASE` covers the timer driver's 8th slot on both.
pub const TIMER7_BASE: u32 = 0x4001_3400;
pub const TIMER8_BASE: u32 = 0x4001_4C00;
pub const TIMER9_BASE: u32 = 0x4001_5000;

/// Register block of ADC `instance` (`0` = ADC1, `1` = ADC2).
///
/// This family has two ADCs: instance 2 (the ARM side's third ADC) does not exist
/// here, so the lookup answers `None` for it.
pub fn adc_base(instance: u32) -> Option<u32> {
    match instance {
        0 => Some(super::peripherals_common::ADC1_BASE),
        1 => Some(super::peripherals_common::ADC2_BASE),
        _ => None,
    }
}

/// RISC-V interrupt controller and system tick, i.e. this family's answer to
/// NVIC/SysTick (the ARM side has `ARM_SCB_BASE` instead).
pub const PFIC_BASE: u32 = 0xE000_E000;
pub const STK_BASE: u32 = 0xE000_F000;

/// The system timer register, at the same address as on the ARM side.
pub const SYS_TIMER_BASE: u32 = 0xD100_0000;
