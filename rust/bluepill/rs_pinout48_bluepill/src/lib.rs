#![allow(non_camel_case_types, non_snake_case)]
#![cfg_attr(not(test), no_std)]

//! Pin -> peripheral function mapping for bluepill compatible MCUs (GD32F3 / CH32V3xx).
//!
//! This is the Rust equivalent of `lnPinMapping.cpp`. The other bluepill crates
//! (timer, adc, i2c, spi, uart, ...) query this crate to know which timer / ADC /
//! I2C / SPI / UART / DAC is wired to a given pin.
//!
//! One (and only one) package feature MUST be enabled:
//! * `bluepill48` : 48 pins package (CMake: `USE_48PIN_PACKAGE`)
//! * `bluepill64` : 64 pins package (CMake: `USE_64PIN_PACKAGE`)
//!
//! The mapping is the default (non remapped) one, except where a `needs_remap`
//! flag says otherwise.

#[cfg(all(feature = "bluepill48", feature = "bluepill64"))]
compile_error!("Features `bluepill48` and `bluepill64` are mutually exclusive");

#[cfg(not(any(feature = "bluepill48", feature = "bluepill64")))]
compile_error!("One of the features `bluepill48` or `bluepill64` must be enabled (USE_48PIN_PACKAGE / USE_64PIN_PACKAGE)");

pub mod registers;
use registers::*;

pub use rs_gpio_bluepill::Pin;

// --- Types ---

/// A timer output channel connected to a pin.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TimerChannel {
    /// Timer index, 0 = TIM1 ... 4 = TIM5
    pub timer: u8,
    /// Channel index, 0 = CH1 ... 3 = CH4
    pub channel: u8,
    /// True if an AFIO remap is required to reach this timer channel on this pin
    pub needs_remap: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum I2cRole {
    Scl,
    Sda,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SpiRole {
    Sck,
    Miso,
    Mosi,
    Nss,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UartRole {
    Tx,
    Rx,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct I2cPin {
    pub instance: u8,
    pub role: I2cRole,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SpiPin {
    pub instance: u8,
    pub role: SpiRole,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct UartPin {
    pub instance: u8,
    pub role: UartRole,
}

/// DMA engine/channel serving a timer channel.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DmaChannel {
    pub engine: u8,
    pub channel: u8,
}

/// Everything known about a pin. `None` = the function is not available on that pin.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PinInfo {
    /// ADC input channel (usable on ADC1 and ADC2)
    pub adc: Option<u8>,
    pub timer: Option<TimerChannel>,
    pub dac: Option<u8>,
    pub i2c: Option<I2cPin>,
    pub spi: Option<SpiPin>,
    pub uart: Option<UartPin>,
}

impl PinInfo {
    const fn none() -> Self {
        PinInfo { adc: None, timer: None, dac: None, i2c: None, spi: None, uart: None }
    }
    const fn with_adc(mut self, channel: u8) -> Self {
        self.adc = Some(channel);
        self
    }
    const fn with_timer(mut self, timer: u8, channel: u8) -> Self {
        self.timer = Some(TimerChannel { timer, channel, needs_remap: false });
        self
    }
    const fn with_timer_remap(mut self, timer: u8, channel: u8) -> Self {
        self.timer = Some(TimerChannel { timer, channel, needs_remap: true });
        self
    }
    const fn with_dac(mut self, channel: u8) -> Self {
        self.dac = Some(channel);
        self
    }
    const fn with_i2c(mut self, instance: u8, role: I2cRole) -> Self {
        self.i2c = Some(I2cPin { instance, role });
        self
    }
    const fn with_spi(mut self, instance: u8, role: SpiRole) -> Self {
        self.spi = Some(SpiPin { instance, role });
        self
    }
    const fn with_uart(mut self, instance: u8, role: UartRole) -> Self {
        self.uart = Some(UartPin { instance, role });
        self
    }
}

const NONE: PinInfo = PinInfo::none();

// --- Mapping table, indexed by Pin value (PA0..PC15) ---

static PIN_TABLE: [PinInfo; TABLE_PINS] = [
    // PA0..PA15
    NONE.with_adc(ADC_IN0).with_timer(TIM2, CH1),
    NONE.with_adc(ADC_IN1).with_timer(TIM2, CH2),
    NONE.with_adc(ADC_IN2).with_timer(TIM2, CH3).with_uart(UART1, UartRole::Tx),
    NONE.with_adc(ADC_IN3).with_timer(TIM2, CH4).with_uart(UART1, UartRole::Rx),
    NONE.with_adc(ADC_IN4).with_dac(DAC_OUT0).with_spi(SPI0, SpiRole::Nss),
    NONE.with_adc(ADC_IN5).with_dac(DAC_OUT1).with_spi(SPI0, SpiRole::Sck),
    NONE.with_adc(ADC_IN6).with_timer(TIM3, CH1).with_spi(SPI0, SpiRole::Miso),
    NONE.with_adc(ADC_IN7).with_timer(TIM3, CH2).with_spi(SPI0, SpiRole::Mosi),
    NONE.with_timer(TIM1, CH1),
    NONE.with_timer(TIM1, CH2).with_uart(UART0, UartRole::Tx),
    NONE.with_timer(TIM1, CH3).with_uart(UART0, UartRole::Rx),
    NONE.with_timer(TIM1, CH4),
    NONE,
    NONE,
    NONE,
    NONE.with_spi(SPI2, SpiRole::Nss),
    // PB0..PB15
    NONE.with_adc(ADC_IN8).with_timer(TIM3, CH3),
    NONE.with_adc(ADC_IN9).with_timer(TIM3, CH4),
    NONE,
    NONE.with_spi(SPI2, SpiRole::Sck),
    // PB4/PB5 : TIM3 CH1/CH2 requires a partial timer 3 remap
    NONE.with_timer_remap(TIM3, CH1).with_spi(SPI2, SpiRole::Miso),
    NONE.with_timer_remap(TIM3, CH2).with_spi(SPI2, SpiRole::Mosi),
    NONE.with_timer(TIM4, CH1).with_i2c(I2C0, I2cRole::Scl),
    NONE.with_timer(TIM4, CH2).with_i2c(I2C0, I2cRole::Sda),
    NONE.with_timer(TIM4, CH3),
    NONE.with_timer(TIM4, CH4),
    NONE.with_i2c(I2C1, I2cRole::Scl).with_uart(UART2, UartRole::Tx),
    NONE.with_i2c(I2C1, I2cRole::Sda).with_uart(UART2, UartRole::Rx),
    NONE.with_spi(SPI1, SpiRole::Nss),
    NONE.with_spi(SPI1, SpiRole::Sck),
    NONE.with_spi(SPI1, SpiRole::Miso),
    NONE.with_spi(SPI1, SpiRole::Mosi),
    // PC0..PC15 (PC0..PC12 only exist in the 64 pins package)
    NONE.with_adc(ADC_IN10),
    NONE.with_adc(ADC_IN11),
    NONE.with_adc(ADC_IN12),
    NONE.with_adc(ADC_IN13),
    NONE.with_adc(ADC_IN14),
    NONE.with_adc(ADC_IN15),
    NONE,
    NONE,
    NONE,
    NONE,
    NONE,
    NONE,
    NONE,
    NONE,
    NONE,
    NONE,
];

/// (timer, channel) -> DMA engine/channel, same as `timerMappings` in lnPinMapping.cpp.
static TIMER_DMA_TABLE: [(u8, u8, DmaChannel); 18] = [
    (TIM1, CH1, DmaChannel { engine: DMA_ENGINE0, channel: 1 }),
    (TIM1, CH2, DmaChannel { engine: DMA_ENGINE0, channel: 2 }),
    (TIM1, CH3, DmaChannel { engine: DMA_ENGINE0, channel: 5 }),
    (TIM1, CH4, DmaChannel { engine: DMA_ENGINE0, channel: 3 }),
    (TIM2, CH3, DmaChannel { engine: DMA_ENGINE0, channel: 0 }),
    (TIM2, CH1, DmaChannel { engine: DMA_ENGINE0, channel: 4 }),
    (TIM2, CH2, DmaChannel { engine: DMA_ENGINE0, channel: 6 }),
    (TIM2, CH4, DmaChannel { engine: DMA_ENGINE0, channel: 6 }),
    (TIM3, CH3, DmaChannel { engine: DMA_ENGINE0, channel: 1 }),
    (TIM3, CH4, DmaChannel { engine: DMA_ENGINE0, channel: 2 }),
    (TIM3, CH1, DmaChannel { engine: DMA_ENGINE0, channel: 5 }),
    (TIM4, CH1, DmaChannel { engine: DMA_ENGINE0, channel: 0 }),
    (TIM4, CH2, DmaChannel { engine: DMA_ENGINE0, channel: 3 }),
    (TIM4, CH3, DmaChannel { engine: DMA_ENGINE0, channel: 4 }),
    (TIM5, CH1, DmaChannel { engine: DMA_ENGINE1, channel: 4 }),
    (TIM5, CH2, DmaChannel { engine: DMA_ENGINE1, channel: 3 }),
    (TIM5, CH3, DmaChannel { engine: DMA_ENGINE1, channel: 1 }),
    (TIM5, CH4, DmaChannel { engine: DMA_ENGINE1, channel: 0 }),
];

// --- Public API ---

/// True if the pin physically exists in the selected package.
pub fn pin_exists(pin: Pin) -> bool {
    let value = pin as u32;
    let port = value / PINS_PER_PORT;
    let bit = value % PINS_PER_PORT;
    match port {
        PORT_A | PORT_B => true,
        #[cfg(feature = "bluepill48")]
        PORT_C => bit >= PC_FIRST_48,
        #[cfg(all(feature = "bluepill64", not(feature = "bluepill48")))]
        PORT_C => true,
        #[cfg(feature = "bluepill48")]
        PORT_D => bit <= PD_LAST_48,
        #[cfg(all(feature = "bluepill64", not(feature = "bluepill48")))]
        PORT_D => bit <= PD_LAST_64,
        _ => false,
    }
}

/// Function mapping of a pin. `None` if the pin does not exist in the selected package.
pub fn pin_info(pin: Pin) -> Option<PinInfo> {
    if !pin_exists(pin) {
        return None;
    }
    Some(PIN_TABLE.get(pin as usize).copied().unwrap_or(NONE))
}

/// ADC channel connected to the pin.
pub fn adc_channel(pin: Pin) -> Option<u8> {
    pin_info(pin).and_then(|i| i.adc)
}

/// Timer channel connected to the pin.
pub fn timer_channel(pin: Pin) -> Option<TimerChannel> {
    pin_info(pin).and_then(|i| i.timer)
}

/// DAC channel connected to the pin.
pub fn dac_channel(pin: Pin) -> Option<u8> {
    pin_info(pin).and_then(|i| i.dac)
}

/// I2C instance/role connected to the pin.
pub fn i2c_pin(pin: Pin) -> Option<I2cPin> {
    pin_info(pin).and_then(|i| i.i2c)
}

/// SPI instance/role connected to the pin.
pub fn spi_pin(pin: Pin) -> Option<SpiPin> {
    pin_info(pin).and_then(|i| i.spi)
}

/// UART instance/role connected to the pin.
pub fn uart_pin(pin: Pin) -> Option<UartPin> {
    pin_info(pin).and_then(|i| i.uart)
}

/// DMA engine/channel serving a timer channel.
pub fn timer_dma(timer: u8, channel: u8) -> Option<DmaChannel> {
    TIMER_DMA_TABLE.iter().find(|e| e.0 == timer && e.1 == channel).map(|e| e.2)
}

/// Setup a pin for timer PWM output and resolve its Timer and DMA assignments.
///
/// Automatically handles GPIO configuration (Pwm mode, 50MHz) and AFIO remapping
/// / JTAG pin release if required by the board pinout.
/// Returns `Some((timer_idx, channel_idx, dma_engine, dma_channel_idx))` on success.
pub fn setup_timer_pin(pin: Pin) -> Option<(u32, u32, rs_dma_bluepill::DmaEngine, usize)> {
    let t_chan = timer_channel(pin)?;
    let dma = timer_dma(t_chan.timer, t_chan.channel)?;

    if t_chan.needs_remap {
        // PB4 and PB5 need JTAG pins released and Timer 2 (TIM3) partially remapped
        if pin == Pin::PB4 || pin == Pin::PB5 {
            rs_afio_bluepill::release_jtag_pins();
            rs_afio_bluepill::remap_timer2_partial();
        }
    }

    rs_gpio_bluepill::set_mode(pin, rs_gpio_bluepill::Mode::Pwm, 50);

    let dma_engine = match dma.engine {
        registers::DMA_ENGINE0 => rs_dma_bluepill::DmaEngine::Dma0,
        _ => rs_dma_bluepill::DmaEngine::Dma1,
    };

    Some((
        t_chan.timer as u32,
        t_chan.channel as u32,
        dma_engine,
        dma.channel as usize,
    ))
}

// --- C API aliases (-1 = not available, like the legacy pinMappings table) ---

const C_NONE: i32 = -1;

#[unsafe(no_mangle)]
pub extern "C" fn rs_pinout_adc(pin: u32) -> i32 {
    adc_channel(Pin::from(pin)).map_or(C_NONE, i32::from)
}

#[unsafe(no_mangle)]
pub extern "C" fn rs_pinout_dac(pin: u32) -> i32 {
    dac_channel(Pin::from(pin)).map_or(C_NONE, i32::from)
}

#[unsafe(no_mangle)]
pub extern "C" fn rs_pinout_timer(pin: u32) -> i32 {
    timer_channel(Pin::from(pin)).map_or(C_NONE, |t| i32::from(t.timer))
}

#[unsafe(no_mangle)]
pub extern "C" fn rs_pinout_timer_channel(pin: u32) -> i32 {
    timer_channel(Pin::from(pin)).map_or(C_NONE, |t| i32::from(t.channel))
}

#[unsafe(no_mangle)]
pub extern "C" fn rs_pinout_timer_dma_engine(timer: u8, channel: u8) -> i32 {
    timer_dma(timer, channel).map_or(C_NONE, |d| i32::from(d.engine))
}

#[unsafe(no_mangle)]
pub extern "C" fn rs_pinout_timer_dma_channel(timer: u8, channel: u8) -> i32 {
    timer_dma(timer, channel).map_or(C_NONE, |d| i32::from(d.channel))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adc_timer_dac() {
        assert_eq!(adc_channel(Pin::PA0), Some(0));
        assert_eq!(adc_channel(Pin::PB1), Some(9));
        assert_eq!(adc_channel(Pin::PA8), None);
        assert_eq!(timer_channel(Pin::PA8).map(|t| (t.timer, t.channel)), Some((0, 0)));
        assert_eq!(timer_channel(Pin::PB9).map(|t| (t.timer, t.channel)), Some((3, 3)));
        assert!(timer_channel(Pin::PB4).unwrap().needs_remap);
        assert_eq!(dac_channel(Pin::PA5), Some(1));
    }

    #[test]
    fn serial_buses() {
        assert_eq!(i2c_pin(Pin::PB7), Some(I2cPin { instance: 0, role: I2cRole::Sda }));
        assert_eq!(spi_pin(Pin::PB13), Some(SpiPin { instance: 1, role: SpiRole::Sck }));
        assert_eq!(uart_pin(Pin::PA9), Some(UartPin { instance: 0, role: UartRole::Tx }));
    }

    #[test]
    fn dma() {
        assert_eq!(timer_dma(1, 0), Some(DmaChannel { engine: 0, channel: 4 }));
        assert_eq!(timer_dma(4, 3), Some(DmaChannel { engine: 1, channel: 0 }));
        assert_eq!(timer_dma(0, 0), Some(DmaChannel { engine: 0, channel: 1 }));
        assert_eq!(timer_dma(0, 4), None);
    }

    #[test]
    #[cfg(feature = "bluepill48")]
    fn package_48() {
        assert!(!pin_exists(Pin::PC0));
        assert!(pin_exists(Pin::PC13));
        assert_eq!(adc_channel(Pin::PC0), None);
        assert!(!pin_exists(Pin::PD2));
    }

    #[test]
    #[cfg(all(feature = "bluepill64", not(feature = "bluepill48")))]
    fn package_64() {
        assert!(pin_exists(Pin::PC0));
        assert_eq!(adc_channel(Pin::PC0), Some(10));
        assert!(pin_exists(Pin::PD2));
        assert!(!pin_exists(Pin::PD3));
    }
}
