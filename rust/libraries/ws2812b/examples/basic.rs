//! Basic example showing how to initialize and drive a WS2812B LED strip.
//!
//! Follows Esprit Rust Rule 4 (Mandatory examples).

#![no_std]
#![no_main]

use ws2812b::{Color, Pin, Ws2812b};

const NUM_LEDS: usize = 8;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    // 1. Initialise WS2812B strip on PB8 (maps to Timer 3, Channel 2, DMA0 Channel 4)
    let mut strip = Ws2812b::<NUM_LEDS>::new(Pin::PB8);
    if strip.begin().is_err() {
        loop {}
    }

    // 2. Set brightness to 50%
    strip.set_global_brightness(128);

    // 3. Set distinct colors across the strip
    strip.set_led_color(0, 255, 0, 0);     // Red
    strip.set_led_color(1, 0, 255, 0);     // Green
    strip.set_led_color(2, 0, 0, 255);     // Blue
    strip.set_led_color(3, 255, 255, 0);   // Yellow
    strip.set_led_color(4, 0, 255, 255);   // Cyan
    strip.set_led_color(5, 255, 0, 255);   // Magenta
    strip.set_led_color(6, 255, 255, 255); // White
    strip.set_led_color(7, 30, 30, 30);    // Dim White

    // 4. Commit pixels to the LED strip via Timer + DMA
    strip.update();

    // 5. Main loop: simple chaser / color cycle
    let colors = [
        Color::RED,
        Color::GREEN,
        Color::BLUE,
        Color::YELLOW,
        Color::CYAN,
        Color::MAGENTA,
    ];

    let mut color_idx = 0;
    loop {
        let c = colors[color_idx % colors.len()];
        strip.set_color(c.r, c.g, c.b);
        strip.update();

        // Delay ~100ms
        for _ in 0..1_000_000 {
            core::hint::spin_loop();
        }

        color_idx += 1;
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
