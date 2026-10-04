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
//! Follows `esprit/codingrules.md`:
//! - **Rule 1: No Magic Numbers** (protocol constants in [`registers`]).
//! - **Rule 2: Two-Tier Architecture** (high-level driver relying on underlying timer crate).
//! - **Rule 3: Re-exporting Types** (re-exports [`Pin`], [`Color`], etc.).
//! - **Rule 4: Examples Mandatory** (see `examples/basic.rs`).
//! - **Rule 5: Scope & API Parity** (full parity with C++ `WS2812B_base` and `WS2812B_timer`).
//! - **Rule 6: Idiomatic Rust & Legacy C++ Compatibility** (safe API + C ABI aliases).
//! - **Rule 7: Target build** (`bluepill` feature for GD32F3 / CH32V3xx).

pub mod color;
pub mod registers;

pub use color::{scale_channel, Color};
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
            let zero = (rollover * DUTY_ZERO_NUMERATOR + DUTY_ROUNDING_OFFSET) / DUTY_ZERO_DENOMINATOR;

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

    /// Commit the current LED colors to the strip via Timer + DMA transfer.
    pub fn update(&mut self) {
        if N == 0 {
            return;
        }

        #[cfg(feature = "bluepill")]
        {
            let Self {
                timer,
                pwm_buffer,
                leds,
                led_brightness,
                global_brightness,
                lookup,
                ..
            } = self;

            let timer = match timer.as_mut() {
                Some(t) => t,
                None => return,
            };

            if N == 1 {
                let grb = get_grb_for_led(leds, led_brightness, *global_brightness, 0);
                write_pwm_samples(lookup, &mut pwm_buffer.data, false, grb);
                pwm_buffer.data[HALF_BUFFER_BYTES..BUFFER_SIZE_BYTES].fill(0);

                timer.start_dma(pwm_buffer.data.as_ptr(), BUFFER_SIZE_BYTES);

                let mut timeout: u32 = 200_000;
                while !timer.is_half_transfer() && timeout > 0 {
                    timeout -= 1;
                }
                timer.clear_half_transfer();

                timer.stop();
                delay_us(WS2812B_RESET_DELAY_US);
                return;
            }

            // Ping-pong double buffering for N > 1 LEDs
            let grb0 = get_grb_for_led(leds, led_brightness, *global_brightness, 0);
            write_pwm_samples(lookup, &mut pwm_buffer.data, false, grb0);

            let grb1 = get_grb_for_led(leds, led_brightness, *global_brightness, 1);
            write_pwm_samples(lookup, &mut pwm_buffer.data, true, grb1);

            let mut next_led = 2;
            timer.start_dma(pwm_buffer.data.as_ptr(), BUFFER_SIZE_BYTES);

            let mut waiting_for_half = true;
            let mut timeout: u32 = 2_000_000;

            while next_led <= N + 1 && timeout > 0 {
                if waiting_for_half {
                    if timer.is_half_transfer() {
                        timer.clear_half_transfer();
                        if next_led < N {
                            let grb = get_grb_for_led(leds, led_brightness, *global_brightness, next_led);
                            write_pwm_samples(lookup, &mut pwm_buffer.data, false, grb);
                        }
                        next_led += 1;
                        waiting_for_half = false;
                    }
                } else {
                    if timer.is_transfer_complete() {
                        timer.clear_transfer_complete();
                        if next_led < N {
                            let grb = get_grb_for_led(leds, led_brightness, *global_brightness, next_led);
                            write_pwm_samples(lookup, &mut pwm_buffer.data, true, grb);
                        }
                        next_led += 1;
                        waiting_for_half = true;
                    }
                }
                timeout -= 1;
            }

            timer.stop();
            delay_us(WS2812B_RESET_DELAY_US);
        }
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
        core::slice::from_raw_parts_mut(
            pwm_data.as_mut_ptr().add(offset) as *mut u32,
            6,
        )
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

/// Simple microsecond busy delay loop.
#[inline(always)]
fn delay_us(us: u32) {
    let iterations = us * 12;
    for _ in 0..iterations {
        core::hint::spin_loop();
    }
}

// -----------------------------------------------------------------------------
// Legacy C ABI Aliases (Esprit Rust Rule 6)
// -----------------------------------------------------------------------------

/// Concrete instance handle for C interoperability (supports up to 64 LEDs).
pub struct Ws2812bCInstance {
    inner: Ws2812b<64>,
}

#[unsafe(no_mangle)]
pub extern "C" fn ws2812b_create(pin: u32) -> *mut Ws2812bCInstance {
    let _pin = Pin::from(pin);
    // For no_std without global allocator, caller can manage instance storage
    core::ptr::null_mut()
}

#[unsafe(no_mangle)]
pub extern "C" fn ws2812b_begin(handle: *mut Ws2812bCInstance) -> bool {
    if handle.is_null() {
        return false;
    }
    unsafe { (*handle).inner.begin().is_ok() }
}

#[unsafe(no_mangle)]
pub extern "C" fn ws2812b_set_global_brightness(handle: *mut Ws2812bCInstance, brightness: u8) {
    if !handle.is_null() {
        unsafe { (*handle).inner.set_global_brightness(brightness) }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ws2812b_set_color(handle: *mut Ws2812bCInstance, r: u8, g: u8, b: u8) {
    if !handle.is_null() {
        unsafe { (*handle).inner.set_color(r, g, b) }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ws2812b_set_led_color(
    handle: *mut Ws2812bCInstance,
    led: u32,
    r: u8,
    g: u8,
    b: u8,
) {
    if !handle.is_null() {
        unsafe { (*handle).inner.set_led_color(led as usize, r, g, b) }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ws2812b_update(handle: *mut Ws2812bCInstance) {
    if !handle.is_null() {
        unsafe { (*handle).inner.update() }
    }
}
