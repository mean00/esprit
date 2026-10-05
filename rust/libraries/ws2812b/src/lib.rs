#![no_std]
#![allow(dead_code)]

//! # WS2812B Timer + DMA Driver for Esprit
//!
//! A pure-Rust driver for WS2812B addressable RGB LEDs using Timer PWM + DMA,
//! mimicking the C++ `WS2812B_timer` driver from `esprit/libraries/WS2812B`.
//!
//! High-level architecture:
//! - Relies on [`rs_timer_bluepill::DmaTimer`] to perform all Timer and DMA configuration.
//! - Focuses purely on protocol formatting (GRB serialization, nibble lookup, double-buffering).
//!

pub mod color;
pub mod registers;

pub use color::{Color, scale_channel};
pub use registers::*;
pub use rs_gpio_bluepill::Pin;

#[cfg(feature = "bluepill")]
use rs_timer_bluepill::{DmaTimer, DmaTimerError};

/// Error types for WS2812B operations.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Ws2812bError {
    PinNotSupported,
    TimerClockError,
    HardwareInitFailed,
}

#[cfg(feature = "bluepill")]
impl From<DmaTimerError> for Ws2812bError {
    fn from(err: DmaTimerError) -> Self {
        match err {
            DmaTimerError::PinNotSupported => Ws2812bError::PinNotSupported,
            DmaTimerError::InvalidClock => Ws2812bError::TimerClockError,
            DmaTimerError::InvalidChannel => Ws2812bError::HardwareInitFailed,
        }
    }
}

#[cfg(feature = "bluepill")]
use rust_esprit::BinarySemaphore;

/// Aligned 48-byte buffer for DMA ping-pong transfer (2 LEDs * 24 bytes).
#[repr(align(4))]
struct PwmBuffer {
    data: [u8; BUFFER_SIZE_BYTES],
}

/// Safe, zero-allocation WS2812B LED strip driver with compile-time fixed capacity `N`.
pub struct Ws2812b<const N: usize> {
    pin: Pin,
    leds: [Color; N],
    led_brightness: [u8; N],
    global_brightness: u8,
    pwm_buffer: PwmBuffer,
    lookup: [u32; NIBBLE_LOOKUP_ENTRIES],
    #[cfg(feature = "bluepill")]
    timer: Option<DmaTimer>,
    one_ticks: u8,
    zero_ticks: u8,
    #[cfg(feature = "bluepill")]
    sem: Option<BinarySemaphore>,
    next_led: usize,
}

impl<const N: usize> Ws2812b<N> {
    /// Create a new WS2812B driver instance controlling `N` LEDs on `pin`.
    pub fn new(pin: Pin) -> Self {
        Self {
            pin,
            leds: [Color::BLACK; N],
            led_brightness: [DEFAULT_BRIGHTNESS; N],
            global_brightness: DEFAULT_BRIGHTNESS,
            pwm_buffer: PwmBuffer {
                data: [0u8; BUFFER_SIZE_BYTES],
            },
            lookup: [0u32; NIBBLE_LOOKUP_ENTRIES],
            #[cfg(feature = "bluepill")]
            timer: None,
            one_ticks: 0,
            zero_ticks: 0,
            #[cfg(feature = "bluepill")]
            sem: None,
            next_led: 0,
        }
    }

    /// Return the number of LEDs managed by this driver.
    #[inline]
    pub const fn num_leds(&self) -> usize {
        N
    }

    /// Initialise the hardware timer and pre-compute the PWM lookup table.
    pub fn begin(&mut self) -> Result<(), Ws2812bError> {
        #[cfg(feature = "bluepill")]
        {
            let mut timer = DmaTimer::new(self.pin)?;
            let rollover = timer.pwm_setup(WS2812B_PWM_FREQUENCY_HZ)?;

            let one = (rollover * DUTY_ONE_NUMERATOR + DUTY_ROUNDING_OFFSET) / DUTY_ONE_DENOMINATOR;
            let zero =
                (rollover * DUTY_ZERO_NUMERATOR + DUTY_ROUNDING_OFFSET) / DUTY_ZERO_DENOMINATOR;

            self.one_ticks = one as u8;
            self.zero_ticks = zero as u8;

            // Pre-compute PWM duty cycle values for each 4-bit nibble (0..15).
            // Each 4-bit value maps to 4 PWM duty-cycle bytes packed into a u32.
            for value in 0..NIBBLE_LOOKUP_ENTRIES {
                let mut copy = value as u8;
                let mut word: u32 = 0;
                for i in 0..BITS_PER_NIBBLE {
                    let byte = if (copy & 0x8) != 0 {
                        self.one_ticks
                    } else {
                        self.zero_ticks
                    };
                    word |= (byte as u32) << (i * 8);
                    copy <<= 1;
                }
                self.lookup[value] = word;
            }

            self.sem = Some(BinarySemaphore::new());
            self.timer = Some(timer);
            Ok(())
        }

        #[cfg(not(feature = "bluepill"))]
        {
            Err(Ws2812bError::HardwareInitFailed)
        }
    }

    /// Set the global brightness factor (0..255).
    #[inline]
    pub fn set_global_brightness(&mut self, brightness: u8) {
        self.global_brightness = brightness;
    }

    /// Set all LEDs to the same RGB color.
    pub fn set_color(&mut self, r: u8, g: u8, b: u8) {
        let color = Color::new(r, g, b);
        for i in 0..N {
            self.leds[i] = color;
        }
    }

    /// Set a single LED's color.
    pub fn set_led_color(&mut self, led: usize, r: u8, g: u8, b: u8) {
        if led < N {
            self.leds[led] = Color::new(r, g, b);
        }
    }

    /// Set a block of LEDs from a raw RGB byte slice.
    pub fn set_led_colors(&mut self, start: usize, data: &[u8]) {
        let count = data.len() / BYTES_PER_LED;
        for i in 0..count {
            let led_idx = start + i;
            if led_idx >= N {
                break;
            }
            let offset = i * BYTES_PER_LED;
            self.leds[led_idx] = Color::new(data[offset], data[offset + 1], data[offset + 2]);
        }
    }

    /// Set individual LED brightness factor (0..255).
    pub fn set_led_brightness(&mut self, led: usize, brightness: u8) {
        if led < N {
            self.led_brightness[led] = brightness;
        }
    }

    /// ISR callback triggered on DMA half-transfer (`half = true`) or transfer-complete (`half = false`).
    pub(crate) fn handle_dma_interrupt(&mut self, half: bool) {
        // Single LED: give semaphore on half (24 bytes sent = 1 LED done)
        if N == 1 && half {
            if let Some(ref sem) = self.sem {
                sem.give_from_isr();
            }
            self.next_led += 1;
            return;
        }

        // More LEDs to send: write next LED into the half that was just consumed
        if self.next_led < N {
            // half == true -> first half was just sent, write there (false = offset 0)
            // half == false -> second half was just sent, write there (true = offset 24)
            let grb = get_grb_for_led(
                &self.leds,
                &self.led_brightness,
                self.global_brightness,
                self.next_led,
            );
            write_pwm_samples(&self.lookup, &mut self.pwm_buffer.data, !half, grb);
            self.next_led += 1;
            return;
        }

        // All LEDs written, last batch still being sent
        if self.next_led == N {
            if half {
                self.pwm_buffer.data[0..HALF_BUFFER_BYTES].fill(0);
            } else {
                self.pwm_buffer.data[HALF_BUFFER_BYTES..BUFFER_SIZE_BYTES].fill(0);
            }
            self.next_led += 1;
            return;
        }

        // Last batch fully sent -> signal completion
        if self.next_led == N + 1 {
            if let Some(ref sem) = self.sem {
                sem.give_from_isr();
            }
            self.next_led += 1;
        }
    }

    /// Commit the current LED colors to the strip via Timer + DMA transfer.
    pub fn update(&mut self) {
        if N == 0 {
            return;
        }

        #[cfg(feature = "bluepill")]
        {
            if self.timer.is_none() || self.sem.is_none() {
                return;
            }

            self.sem.as_ref().unwrap().try_take(); // Clear any stale semaphore

            if N == 1 {
                self.next_led = 1;
                let grb =
                    get_grb_for_led(&self.leds, &self.led_brightness, self.global_brightness, 0);
                write_pwm_samples(&self.lookup, &mut self.pwm_buffer.data, false, grb);
                self.pwm_buffer.data[HALF_BUFFER_BYTES..BUFFER_SIZE_BYTES].fill(0);
            } else {
                self.next_led = 2;
                let grb0 =
                    get_grb_for_led(&self.leds, &self.led_brightness, self.global_brightness, 0);
                write_pwm_samples(&self.lookup, &mut self.pwm_buffer.data, false, grb0);
                let grb1 =
                    get_grb_for_led(&self.leds, &self.led_brightness, self.global_brightness, 1);
                write_pwm_samples(&self.lookup, &mut self.pwm_buffer.data, true, grb1);
            }

            let cookie = self as *mut Self as *mut core::ffi::c_void;
            let buf_ptr = self.pwm_buffer.data.as_ptr();

            {
                let timer = self.timer.as_mut().unwrap();
                timer.attach_dma_callback(dma_irq_trampoline::<N>, cookie);
                timer.start_dma(buf_ptr, BUFFER_SIZE_BYTES);
            }

            // Block task on FreeRTOS semaphore until transfer completes (or timeout)
            self.sem
                .as_ref()
                .unwrap()
                .take_timeout(WS2812B_DMA_TIMEOUT_MS);

            delay_us(WS2812B_STOP_DELAY_US);
            {
                let timer = self.timer.as_mut().unwrap();
                timer.stop();
                timer.detach_dma_callback();
            }
            delay_us(WS2812B_RESET_DELAY_US);
        }
    }
}

unsafe extern "C" fn dma_irq_trampoline<const N: usize>(
    half: bool,
    cookie: *mut core::ffi::c_void,
) {
    if !cookie.is_null() {
        let this = unsafe { &mut *(cookie as *mut Ws2812b<N>) };
        this.handle_dma_interrupt(half);
    }
}

/// Helper: Get effective scaled GRB wire-format bytes for a specific LED.
#[inline]
fn get_grb_for_led<const N: usize>(
    leds: &[Color; N],
    led_brightness: &[u8; N],
    global_brightness: u8,
    led: usize,
) -> [u8; 3] {
    let c = leds[led];
    let led_bright = led_brightness[led];
    let combined_bright = scale_channel(global_brightness, led_bright);
    c.to_grb_scaled(combined_bright)
}

/// Helper: Convert GRB color bytes into 24 PWM duty-cycle samples in the ping-pong buffer.
fn write_pwm_samples(
    lookup: &[u32; NIBBLE_LOOKUP_ENTRIES],
    pwm_data: &mut [u8; BUFFER_SIZE_BYTES],
    second_half: bool,
    grb: [u8; 3],
) {
    let offset = if second_half { HALF_BUFFER_BYTES } else { 0 };
    let target = unsafe {
        core::slice::from_raw_parts_mut(pwm_data.as_mut_ptr().add(offset) as *mut u32, 6)
    };

    // Green channel (MSB first)
    let high_g = ((grb[0] & NIBBLE_HIGH_MASK) >> NIBBLE_SHIFT) as usize;
    let low_g = (grb[0] & NIBBLE_LOW_MASK) as usize;
    target[0] = lookup[high_g];
    target[1] = lookup[low_g];

    // Red channel
    let high_r = ((grb[1] & NIBBLE_HIGH_MASK) >> NIBBLE_SHIFT) as usize;
    let low_r = (grb[1] & NIBBLE_LOW_MASK) as usize;
    target[2] = lookup[high_r];
    target[3] = lookup[low_r];

    // Blue channel
    let high_b = ((grb[2] & NIBBLE_HIGH_MASK) >> NIBBLE_SHIFT) as usize;
    let low_b = (grb[2] & NIBBLE_LOW_MASK) as usize;
    target[4] = lookup[high_b];
    target[5] = lookup[low_b];
}

unsafe extern "C" {
    fn lnDelayUs(us: u32);
}

/// Precise microsecond delay using Esprit system timer.
#[inline(always)]
fn delay_us(us: u32) {
    unsafe {
        lnDelayUs(us);
    }
}

pub mod shim;
pub use shim::*;
