//! High-level DmaTimer coordinating Timer and DMA peripherals.
//!
//! Exposes a clean, high-level API for PWM generation with DMA circular buffering.
//! Neither this module nor consumers need to know how hardware registers are accessed.

use crate::timer::{ChannelMode, Timer};
use rs_dma_bluepill::{DmaChannel, DmaEngine};
use rs_gpio_bluepill::{set_mode, Mode, Pin};
use rs_rcu_bluepill::{enable, Peripheral};

/// Error type for DmaTimer initialization.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DmaTimerError {
    PinNotSupported,
    InvalidClock,
    InvalidChannel,
}

/// High-level Timer + DMA controller for high-throughput PWM streams (e.g., WS2812B).
pub struct DmaTimer {
    timer: Timer,
    channel: u32,
    pin: Pin,
    dma: DmaChannel,
    rollover: u32,
}

impl DmaTimer {
    /// Create a new `DmaTimer` for the given GPIO pin.
    ///
    /// Automatically resolves the pin to its corresponding Timer peripheral,
    /// Channel, DMA engine, and DMA channel.
    pub fn new(pin: Pin) -> Result<Self, DmaTimerError> {
        let (timer_idx, channel, dma_engine, dma_channel_idx) =
            map_pin_to_timer_dma(pin).ok_or(DmaTimerError::PinNotSupported)?;

        let timer = Timer::new(timer_idx).map_err(|_| DmaTimerError::InvalidChannel)?;
        let dma = DmaChannel::new(dma_engine, dma_channel_idx);

        Ok(Self {
            timer,
            channel,
            pin,
            dma,
            rollover: 0,
        })
    }

    /// Return the auto-reload rollover count (ARR + 1) for the current PWM frequency.
    #[inline]
    pub fn rollover(&self) -> u32 {
        self.rollover
    }

    /// Configure the pin and timer for PWM output at `frequency_hz`.
    ///
    /// Returns the timer rollover period on success.
    pub fn pwm_setup(&mut self, frequency_hz: u32) -> Result<u32, DmaTimerError> {
        // Enable GPIO port clock and configure pin for PWM (50 MHz alternate push-pull)
        let gpio_periph = match (self.pin as u32) >> 4 {
            0 => Peripheral::GpioA,
            1 => Peripheral::GpioB,
            2 => Peripheral::GpioC,
            _ => return Err(DmaTimerError::PinNotSupported),
        };
        enable(gpio_periph);

        // PB4 and PB5 require AFIO JTAG release and TIM3 partial remap
        if self.pin == Pin::PB4 || self.pin == Pin::PB5 {
            enable(Peripheral::Afio);
            let afio = rs_afio_bluepill::registers::AfioRegisters::ptr();
            unsafe {
                let mut pcf0 = core::ptr::read_volatile(&mut (*afio).pcf0);
                // Free PB4 from JTAG NJTRST: SWJ_CFG = 010 (JTAG disabled, SW enabled)
                pcf0 &= !(7 << 24);
                pcf0 |= 2 << 24;
                // TIM3 partial remap: TIM3_REMAP = 01 (PB4=CH1, PB5=CH2)
                pcf0 &= !(3 << 10);
                pcf0 |= 1 << 10;
                core::ptr::write_volatile(&mut (*afio).pcf0, pcf0);
            }
        }

        set_mode(self.pin, Mode::Pwm, 50);

        // Configure timer via its high-level API
        self.timer.disable();
        let rollover = self
            .timer
            .set_frequency(frequency_hz)
            .map_err(|_| DmaTimerError::InvalidClock)?;

        self.timer.set_channel_mode(self.channel, ChannelMode::Pwm0);
        self.timer.set_channel_compare(self.channel, rollover / 2);
        self.timer.set_channel_output(self.channel, true);
        self.timer.set_auto_reload_preload(true);

        self.rollover = rollover;
        Ok(rollover)
    }

    /// Start a circular DMA transfer from the memory buffer to the timer compare register.
    pub fn start_dma(&mut self, data: *const u8, length: usize) {
        let periph_addr = self.timer.channel_compare_reg_addr(self.channel);
        let mem_addr = data as usize as u32;

        // Configure DMA in circular mode (8-bit source, 16-bit destination)
        self.dma.begin_circular_tx_transfer(periph_addr, mem_addr, length as u32, false, true);

        // Connect Timer to DMA and start counting
        self.timer.enable_channel_dma(self.channel, true);
        self.timer.set_dma_on_compare_event(true);
        self.timer.reset_counter();
        self.timer.enable();
    }

    /// Check if DMA half-transfer event occurred.
    #[inline]
    pub fn is_half_transfer(&self) -> bool {
        self.dma.is_half_transfer()
    }

    /// Clear DMA half-transfer event flag.
    #[inline]
    pub fn clear_half_transfer(&mut self) {
        self.dma.clear_half_transfer();
    }

    /// Check if DMA transfer-complete event occurred.
    #[inline]
    pub fn is_transfer_complete(&self) -> bool {
        self.dma.is_transfer_complete()
    }

    /// Clear DMA transfer-complete event flag.
    #[inline]
    pub fn clear_transfer_complete(&mut self) {
        self.dma.clear_transfer_complete();
    }

    /// Stop Timer and DMA, forcing the output channel low.
    pub fn stop(&mut self) {
        self.timer.disable();
        self.timer.enable_channel_dma(self.channel, false);
        self.timer.set_channel_mode(self.channel, ChannelMode::ForceLow);
        self.dma.end_transfer();
    }
}

/// Map a Bluepill GPIO Pin to (timer_idx, channel, dma_engine, dma_channel_idx).
fn map_pin_to_timer_dma(pin: Pin) -> Option<(u32, u32, DmaEngine, usize)> {
    match pin {
        // PA0..PA3: Timer 1 (TIM2), CH 0..3
        Pin::PA0 => Some((1, 0, DmaEngine::Dma0, 4)),
        Pin::PA1 => Some((1, 1, DmaEngine::Dma0, 6)),
        Pin::PA2 => Some((1, 2, DmaEngine::Dma0, 0)),
        Pin::PA3 => Some((1, 3, DmaEngine::Dma0, 6)),

        // PA6, PA7: Timer 2 (TIM3), CH 0, 1
        Pin::PA6 => Some((2, 0, DmaEngine::Dma0, 5)),
        Pin::PA7 => Some((2, 1, DmaEngine::Dma0, 6)),

        // PA8..PA11: Timer 0 (TIM1), CH 0..3
        Pin::PA8 => Some((0, 0, DmaEngine::Dma0, 1)),
        Pin::PA9 => Some((0, 1, DmaEngine::Dma0, 2)),
        Pin::PA10 => Some((0, 2, DmaEngine::Dma0, 5)),
        Pin::PA11 => Some((0, 3, DmaEngine::Dma0, 3)),

        // PB0, PB1: Timer 2 (TIM3), CH 2, 3
        Pin::PB0 => Some((2, 2, DmaEngine::Dma0, 1)),
        Pin::PB1 => Some((2, 3, DmaEngine::Dma0, 2)),

        // PB4, PB5: Timer 2 (TIM3), CH 0, 1 (requires partial remap)
        Pin::PB4 => Some((2, 0, DmaEngine::Dma0, 5)),
        Pin::PB5 => Some((2, 1, DmaEngine::Dma0, 6)),

        // PB6..PB9: Timer 3 (TIM4), CH 0..3
        Pin::PB6 => Some((3, 0, DmaEngine::Dma0, 0)),
        Pin::PB7 => Some((3, 1, DmaEngine::Dma0, 3)),
        Pin::PB8 => Some((3, 2, DmaEngine::Dma0, 4)),
        Pin::PB9 => Some((3, 3, DmaEngine::Dma0, 4)),

        _ => None,
    }
}
