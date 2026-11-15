//! Addresses that exist only on the **ARM side** of the family — STM32F1 and
//! GD32F30x, i.e. `esprit/private_include/lnPeripheral_priv_stm32.h`.
//!
//! CH32V3 stops at two ADCs, and its timer above `TIMER6` is numbered differently
//! (see the note on `TIM8_BASE`). Everything both families share is in
//! `peripherals_common.rs`.

/// STM32F1 / GD32F30x ADCs have no input buffer bit (`ADC_CTL0[31:24]` is reserved).
pub const HAS_ADC_BUFFER: bool = false;

/// Third ADC. GD32F30x and STM32F1 high-density parts have it; CH32V3 does not.
pub const ADC3_BASE: u32 = 0x4001_3C00;

/// Register block of ADC `instance` (`0` = ADC1, `1` = ADC2, `2` = ADC3).
///
/// The lookup exists because the two families do not have the same number of ADCs:
/// this side has three, CH32V3 has two and answers `None` for instance 2.
pub fn adc_base(instance: u32) -> Option<u32> {
    match instance {
        0 => Some(super::peripherals_common::ADC1_BASE),
        1 => Some(super::peripherals_common::ADC2_BASE),
        2 => Some(ADC3_BASE),
        _ => None,
    }
}

/// `SYSTICK`, at the ARM private-peripheral address. CH32V3 uses its `STK`
/// block instead (`peripherals_ch32v3x::STK_BASE`).
pub const SYS_TIMER_BASE: u32 = 0xD100_0000;

/// ARM System Control Block (VTOR, NVIC, …). CH32V3 has `PFIC`/`STK` instead.
pub const ARM_SCB_BASE: u32 = 0xE000_ED00;

/// ARM Cortex-M SysTick timer (`CSR`, `RVR`, `CVR`, `CALIB`).
pub const ARM_SYSTICK_BASE: u32 = 0xE000_E010;

/// ARM Cortex-M NVIC (`ISER`, `ICER`, `ISPR`, `ICPR`, `IPR`).
pub const ARM_NVIC_BASE: u32 = 0xE000_E100;

/// MCU Debug unit (`DBG_ID`, `DBG_CTL0`).
pub const DBG_BASE: u32 = 0xE004_2000;

/// ARM Cortex-M4 FPU Media and VFP Feature Registers (`MVFR0`, `MVFR1`).
pub const ARM_FPU_MVFR_BASE: u32 = 0xE000_EF40;

/// GD32F3x / STM32F1 Flash/RAM density information register.
pub const MCU_DENSITY_ADDR: u32 = 0x1FFF_F7E0;

/// Fifth USART (UART4 in GD32 naming). CH32V3 puts UART5 at `0x4000_1800`
/// (`peripherals_ch32v3x::USART5_BASE`).
pub const USART5_BASE: u32 = 0x4000_5000;
