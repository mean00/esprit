//! ESP32 / C-ABI wrapped platform driver selections and re-exports.

/// The SPI driver of the current target (`rs_spi_wrapped::SpiWrapped`).
pub use rs_spi_wrapped::SpiWrapped as SpiDriver;
/// The single-shot ADC driver of the current target (`rs_adc_wrapped::SimpleAdcWrapped`).
pub use rs_adc_wrapped::SimpleAdcWrapped as AdcDriver;
/// The scan (DMA / multi-sample) ADC driver of the current target (`rs_adc_wrapped::TimingAdcWrapped`).
pub use rs_adc_wrapped::TimingAdcWrapped as ScanAdcDriver;
/// The external-interrupt driver of the current target (`rs_exti_wrapped::ExtiWrapped`).
pub use rs_exti_wrapped::ExtiWrapped as ExtiDriver;
/// The UART driver of the current target (`rs_uart_wrapped::Uart`).
pub use rs_uart_wrapped::Uart as UartDriver;
/// The I²C driver of the current target (`rs_i2c_wrapped::I2cWrapped`).
pub use rs_i2c_wrapped::I2cWrapped as I2cDriver;
/// The timer channel driver of the current target (`rs_timer_wrapped::TimerWrapped`).
pub use rs_timer_wrapped::TimerWrapped as TimerDriver;
/// The one-shot delay driver of the current target (`rs_timer_wrapped::DelayTimerWrapped`).
pub use rs_timer_wrapped::DelayTimerWrapped as DelayTimerDriver;
