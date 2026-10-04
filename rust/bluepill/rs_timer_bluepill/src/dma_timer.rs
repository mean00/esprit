//! High-level DmaTimer coordinating Timer and DMA peripherals.
//!
//! Exposes a clean, high-level API for PWM generation with DMA circular buffering.
//! Neither this module nor consumers need to know how hardware registers are accessed.

use crate::timer::{ChannelMode, Timer};
use rs_dma_bluepill::{DmaChannel, DmaEngine};
use rs_gpio_bluepill::Pin;
use rs_pinout48_bluepill::setup_timer_pin;

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
    /// Channel, DMA engine, and DMA channel using `rs_pinout48_bluepill`.
    pub fn new(pin: Pin) -> Result<Self, DmaTimerError> {
        let (timer_idx, channel, dma_engine, dma_channel_idx) =
            setup_timer_pin(pin).ok_or(DmaTimerError::PinNotSupported)?;

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
        // Configure timer via its high-level API
        self.timer.disable();
        let rollover = self
            .timer
            .set_frequency(frequency_hz)
            .map_err(|_| DmaTimerError::InvalidClock)?;

        self.timer.set_channel_mode(self.channel, ChannelMode::Pwm0);
        self.timer.set_channel_compare(self.channel, rollover / 2);
        self.timer.set_channel_output(self.channel, true);

        self.rollover = rollover;
        Ok(rollover)
    }

    /// Start a circular DMA transfer from the memory buffer to the timer compare register.
    pub fn start_dma(&mut self, data: *const u8, length: usize) {
        let periph_addr = self.timer.channel_compare_reg_addr(self.channel);
        let mem_addr = data as usize as u32;

        // Configure DMA in circular mode (8-bit source, 16-bit destination)
        self.dma.begin_circular_tx_transfer(periph_addr, mem_addr, length as u32, false, true);

        // Preload compare register with rollover / 2 and restore PWM mode
        self.timer.set_channel_compare(self.channel, self.rollover / 2);
        self.timer.enable_channel_dma(self.channel, true);
        self.timer.set_dma_on_compare_event(true);
        self.timer.set_channel_mode(self.channel, ChannelMode::Pwm0);

        // Preload CNT = CAR - 1.
        // At CNT = CAR - 1, CNT >= CHCV (rollover / 2), so output is LOW.
        // On the next clock cycle (~10 ns), CNT rolls over to 0, triggering the
        // update event. DMAS routes this update event to DMA, immediately loading
        // data[0] into CHCV and starting the first PWM pulse with proper timing.
        self.timer.disable();
        let car = self.timer.auto_reload();
        self.timer.set_counter(car.saturating_sub(1));
        self.timer.set_channel_output(self.channel, true);
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

    /// Return the number of remaining transfers in DMA CNDTR.
    #[inline]
    pub fn remaining_dma_transfers(&self) -> u32 {
        self.dma.remaining_transfers()
    }

    /// Stop Timer and DMA, forcing the output channel low.
    pub fn stop(&mut self) {
        self.timer.disable();
        self.timer.set_channel_output(self.channel, false);
        self.timer.enable_channel_dma(self.channel, false);
        self.timer.set_channel_compare(self.channel, 0);
        self.timer.set_channel_mode(self.channel, ChannelMode::ForceLow);
        self.dma.end_transfer();
    }
}
