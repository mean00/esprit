//! Common Rust Timer demo using the MCU-agnostic timer super traits.
//!
//! `rs_mcu_traits::timer::Timer` produces a waveform on the channel's own pin (a
//! one-shot pulse here) and `rs_mcu_traits::timer::DelayTimer` is the one-shot
//! microsecond delay. Both take a **typed** handler instead of a C cookie, so the
//! interrupt body is a normal `impl` block over atomics.
//!
//! Follows Esprit Rust Rule 1 (No Magic Numbers), Rule 2 (Two-Tier Architecture),
//! Rule 4 (Mandatory Examples), and Rule 6 (Idiomatic Rust).

#![no_std]

use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use rust_esprit::{
    DelayHandler, DelayTimerDriver, HalDelayTimer, HalTimer, TimerDriver, TimerHandler, delay_ms,
    logger, logger_init,
};

logger_init!();

static DELAY_EXPIRED: AtomicBool = AtomicBool::new(false);
static IRQ_TICKS: AtomicU32 = AtomicU32::new(0);

/// Hardware timer channel of the timing demo; the pulse leaves on that channel's pin.
const TIMER_INDEX: u32 = 1;
const DELAY_TIMER_INDEX: u32 = 1;
const PULSE_US: u32 = 10_000; // 10 ms single-shot pulse
const DELAY_US: u32 = 500; // 500 microseconds
const POLL_ROUNDS: u32 = 10;

/// Runs in the delay timer interrupt, so it only touches atomics.
struct Expiry;

impl DelayHandler for Expiry {
    fn on_elapsed(&self) {
        DELAY_EXPIRED.store(true, Ordering::SeqCst);
        IRQ_TICKS.fetch_add(1, Ordering::Relaxed);
    }
}

static EXPIRY: Expiry = Expiry;

/// Runs in the timer channel interrupt, once per period.
struct Ticks;

impl TimerHandler for Ticks {
    fn on_tick(&self) {
        IRQ_TICKS.fetch_add(1, Ordering::Relaxed);
    }
}

static TICKS: Ticks = Ticks;

#[unsafe(no_mangle)]
extern "C" fn user_init() {
    logger!("Starting Timer & DelayTimer test...\n");

    // 1. Single-shot hardware pulse through the timer channel.
    if let Ok(mut timer) = TimerDriver::new(TIMER_INDEX) {
        HalTimer::set_handler(&mut timer, Some(&TICKS));
        logger!("Generating 10ms single-shot pulse...\n");
        let _ = HalTimer::start_single_shot(&mut timer, PULSE_US, true);
    }

    // 2. One-shot microsecond delay with an interrupt handler.
    if let Ok(mut dt) = DelayTimerDriver::new(DELAY_TIMER_INDEX) {
        HalDelayTimer::set_handler(&mut dt, Some(&EXPIRY));
        HalDelayTimer::enable_interrupt(&mut dt);
        logger!("Arming DelayTimer for {} us...\n", DELAY_US);
        let _ = HalDelayTimer::arm_us(&mut dt, DELAY_US);

        for _ in 0..POLL_ROUNDS {
            if DELAY_EXPIRED.load(Ordering::SeqCst) {
                logger!(
                    "DelayTimer expired! Total ticks: {}\n",
                    IRQ_TICKS.load(Ordering::Relaxed)
                );
                break;
            }
            delay_ms(1);
        }
    }

    logger!("--end--\n");
}

