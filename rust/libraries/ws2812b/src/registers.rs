#![allow(dead_code)]

//! Protocol constants for WS2812B LED driver.
//!
//! Follows Esprit Rust Rule 1: No magic numbers in driver logic.

/// Carrier PWM frequency for WS2812B (830 kHz corresponds to ~1.20 µs period).
pub const WS2812B_PWM_FREQUENCY_HZ: u32 = 830_000;

/// Minimum reset / latch low duration in microseconds (>280 µs required by modern WS2812B/SK6812).
pub const WS2812B_RESET_DELAY_US: u32 = 300;


/// Size of the circular DMA ping-pong buffer in bytes (2 LEDs * 24 bytes/LED).
pub const BUFFER_SIZE_BYTES: usize = 48;

/// Bytes per LED in the PWM stream (24 bits = 24 PWM duty-cycle samples).
pub const LED_PWM_BYTES: usize = 24;

/// Size of one half of the ping-pong buffer (1 LED worth).
pub const HALF_BUFFER_BYTES: usize = 24;

/// Bytes per LED in RGB color format (Green, Red, Blue).
pub const BYTES_PER_LED: usize = 3;

/// Number of bits in a byte.
pub const BITS_PER_BYTE: usize = 8;

/// Number of bits per nibble.
pub const BITS_PER_NIBBLE: usize = 4;

/// Number of entries in the 4-bit nibble PWM lookup table.
pub const NIBBLE_LOOKUP_ENTRIES: usize = 16;

/// Mask for high nibble (bits 7..4).
pub const NIBBLE_HIGH_MASK: u8 = 0xF0;

/// Mask for low nibble (bits 3..0).
pub const NIBBLE_LOW_MASK: u8 = 0x0F;

/// Bit shift to extract high nibble.
pub const NIBBLE_SHIFT: u32 = 4;

/// Duty cycle calculation for bit '1': ~66% high (4/6 of period).
pub const DUTY_ONE_NUMERATOR: u32 = 4;
pub const DUTY_ONE_DENOMINATOR: u32 = 6;

/// Duty cycle calculation for bit '0': ~33% high (2/6 of period).
pub const DUTY_ZERO_NUMERATOR: u32 = 2;
pub const DUTY_ZERO_DENOMINATOR: u32 = 6;

/// Rounding offset added before division by 6.
pub const DUTY_ROUNDING_OFFSET: u32 = 3;

/// Default maximum brightness (255 = 100%).
pub const DEFAULT_BRIGHTNESS: u8 = 255;
