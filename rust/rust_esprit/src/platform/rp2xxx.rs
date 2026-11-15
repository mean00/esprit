//! RP2040 / RP2350 platform driver selections and re-exports.

#[cfg(feature = "rp2040")]
pub use rs_rp2040_mcu;
#[cfg(feature = "rp2350")]
pub use rs_rp2350_mcu;

#[cfg(feature = "rp2040")]
pub(crate) use rs_rp2040_mcu as mcu;
#[cfg(feature = "rp2350")]
pub(crate) use rs_rp2350_mcu as mcu;

#[cfg(not(feature = "external_std"))]
#[path = "../rp_sys_init.rs"]
mod rp_sys_init;

/// The SPI driver of the current target (`rs_spi_rp2xxx::Spi`).
pub use rs_spi_rp2xxx::Spi as SpiDriver;
/// The single-shot ADC driver of the current target (`rs_adc_rp2xxx::SimpleAdc`).
pub use rs_adc_rp2xxx::SimpleAdc as AdcDriver;
/// The scan (DMA / multi-sample) ADC driver of the current target (`rs_adc_rp2xxx::TimingAdc`).
pub use rs_adc_rp2xxx::TimingAdc as ScanAdcDriver;
/// The external-interrupt driver of the current target (`rs_exti_rp2xxx::Exti`).
pub use rs_exti_rp2xxx::Exti as ExtiDriver;
/// The UART driver of the current target (`rs_uart_rp2xxx::Uart`).
pub use rs_uart_rp2xxx::Uart as UartDriver;
/// The I²C driver of the current target (`rs_i2c_rp2xxx::I2c`).
pub use rs_i2c_rp2xxx::I2c as I2cDriver;
/// The timer channel driver of the current target (`rs_timer_rp2xxx::Timer`).
pub use rs_timer_rp2xxx::Timer as TimerDriver;
/// The one-shot delay driver of the current target (`rs_timer_rp2xxx::DelayTimer`).
pub use rs_timer_rp2xxx::DelayTimer as DelayTimerDriver;

/// The RP2040 / RP2350 pins (`rs_gpio_rp2xxx`).
pub use rs_gpio_rp2xxx as gpio_rp2xxx;
/// The RP2040 / RP2350 UART driver (`rs_uart_rp2xxx`).
pub use rs_uart_rp2xxx as uart_rp2xxx;

/// RP2040/RP2350: "Endpoints 0 -> 15 in both in and out directions" (`rp2040.md`).
pub const USB_MAX_ENDPOINT: usize = 16;

pub mod rp2xxx {
    //! Low-level RP2xxx (RP2040 / RP2350) peripheral driver crates.
    pub use rs_adc_rp2xxx as adc;
    pub use rs_dma_rp2xxx as dma;
    pub use rs_gpio_rp2xxx as gpio;
    pub use rs_i2c_rp2xxx as i2c;
    pub use rs_multi_pulse_rp2xxx as multi_pulse;
    pub use rs_rcu_rp2xxx as rcu;
    pub use rs_spi_rp2xxx as spi;
    pub use rs_timer_rp2xxx as timer;
}

pub use {
    rs_adc_rp2xxx, rs_dma_rp2xxx, rs_gpio_rp2xxx, rs_i2c_rp2xxx, rs_multi_pulse_rp2xxx,
    rs_rcu_rp2xxx, rs_spi_rp2xxx, rs_timer_rp2xxx,
};
