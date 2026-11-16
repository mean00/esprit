//! Bluepill (GD32F3 / CH32V3) platform driver selections and re-exports.

#[cfg(all(
    feature = "bluepill",
    any(feature = "ch32v3x", target_arch = "riscv32")
))]
pub use rs_ch32v3x;
#[cfg(all(
    feature = "bluepill",
    not(feature = "ch32v3x"),
    any(feature = "gd32fx", target_arch = "arm")
))]
pub use rs_gd32fx;

#[cfg(all(
    feature = "bluepill",
    any(feature = "ch32v3x", target_arch = "riscv32")
))]
pub(crate) use rs_ch32v3x as mcu;
#[cfg(all(
    feature = "bluepill",
    not(feature = "ch32v3x"),
    any(feature = "gd32fx", target_arch = "arm")
))]
pub(crate) use rs_gd32fx as mcu;

#[cfg(not(feature = "external_std"))]
#[path = "../bluepill_sys_init.rs"]
mod bluepill_sys_init;

/// The SPI driver of the current target (`rs_spi_bluepill::Spi`).
pub use rs_spi_bluepill::Spi as SpiDriver;
/// The single-shot ADC driver of the current target (`rs_adc_bluepill::SimpleAdc`).
pub use rs_adc_bluepill::SimpleAdc as AdcDriver;
/// The scan (DMA / multi-sample) ADC driver of the current target (`rs_adc_bluepill::TimingAdc`).
pub use rs_adc_bluepill::TimingAdc as ScanAdcDriver;
/// The external-interrupt driver of the current target (`rs_exti_bluepill::Exti`).
pub use rs_exti_bluepill::Exti as ExtiDriver;
/// The UART driver of the current target (`rs_uart_bluepill::Uart`).
pub use rs_uart_bluepill::Uart as UartDriver;
/// The I²C driver of the current target (`rs_i2c_bluepill::I2c`).
pub use rs_i2c_bluepill::I2c as I2cDriver;
/// The timer channel driver of the current target (`rs_timer_bluepill::Timer`).
pub use rs_timer_bluepill::Timer as TimerDriver;
/// The one-shot delay driver of the current target (`rs_timer_bluepill::DelayTimer`).
pub use rs_timer_bluepill::DelayTimer as DelayTimerDriver;

/// The flash controller (`rs_fmc_bluepill`): kept in the link for C++ callers
/// (`lnCpuID_arm_gd32fx.cpp` -> `FMC_hasFastUnlock()`, `lnRunTime_riscv_ch32v3x.cpp` -> `lnInitFlash_ch32v3x()`).
#[cfg(feature = "bluepill")]
mod fmc_anchor {
    #[used]
    static KEEP_INIT_FLASH: fn() = rs_fmc_bluepill::init_flash;
    #[used]
    static KEEP_FAST_UNLOCK: fn() -> bool = rs_fmc_bluepill::has_fast_unlock;
}

/// Endpoint numbers of this platform's USB device controller, EP0 included.
#[cfg(feature = "bluepill")]
pub const USB_MAX_ENDPOINT: usize = rs_bluepill::USB_MAX_ENDPOINT;

#[cfg(feature = "bluepill")]
pub mod bluepill {
    //! Low-level Bluepill (GD32F3 / CH32V3) peripheral driver crates.
    pub use rs_adc_bluepill as adc;
    pub use rs_afio_bluepill as afio;
    pub use rs_dma_bluepill as dma;
    pub use rs_exti_bluepill as exti;
    pub use rs_gpio_bluepill as gpio;
    pub use rs_i2c_bluepill as i2c;
    pub use rs_multi_pulse_bluepill as multi_pulse;
    #[cfg(feature = "bluepill48")]
    pub use rs_pinout48_bluepill as pinout;
    pub use rs_rcu_bluepill as rcu;
    pub use rs_spi_bluepill as spi;
    pub use rs_timer_bluepill as timer;
    pub use rs_uart_bluepill as uart;
}

#[cfg(feature = "bluepill")]
pub use {
    rs_adc_bluepill, rs_afio_bluepill, rs_dma_bluepill, rs_exti_bluepill, rs_gpio_bluepill,
    rs_i2c_bluepill, rs_multi_pulse_bluepill, rs_rcu_bluepill, rs_spi_bluepill, rs_timer_bluepill,
    rs_uart_bluepill,
};
