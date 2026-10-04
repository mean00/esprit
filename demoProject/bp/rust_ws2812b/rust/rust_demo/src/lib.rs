#![no_std]

use rust_esprit::delay_ms;
use rust_esprit::{logger, logger_init};
use ws2812b::{Pin, Ws2812b};

logger_init!();

/// 3000 ms / 250 steps = 12 ms per step (step size 1 or ~12 ms per single value).
/// 3000 ms / 255 steps ≈ 11.76 ms.
/// To accurately span 3000 ms across 250 brightness levels (0 to 250 with step 1),
/// 250 steps * 12 ms = 3000 ms exactly.
const STEP_DELAY_MS: u32 = 12;
const MAX_BRIGHTNESS: u8 = 250;

#[unsafe(no_mangle)]
extern "C" fn user_init() {
    logger!("Initialising WS2812B driver on pin PB4...\n");

    let mut strip = Ws2812b::<1>::new(Pin::PB4);
    if strip.begin().is_err() {
        logger!("Failed to initialise WS2812B on PB4\n");
        return;
    }

    logger!("WS2812B ready. Starting blue glow loop (3s ramp up, 3s ramp down)...\n");

    loop {
        // Glow up: dark (0) to pure blue (250) over 3 seconds (250 * 12 ms = 3000 ms)
        for b in 0..=MAX_BRIGHTNESS {
            strip.set_color(0, 0, b);
            strip.update();
            delay_ms(STEP_DELAY_MS);
        }

        // Glow down: pure blue (250) to dark (0) over 3 seconds (250 * 12 ms = 3000 ms)
        for b in (0..=MAX_BRIGHTNESS).rev() {
            strip.set_color(0, 0, b);
            strip.update();
            delay_ms(STEP_DELAY_MS);
        }
    }
}
