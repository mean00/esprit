//! Common Rust Timer demo using high-level `rust_esprit` API.
//!
//! Follows Esprit Rust Rule 1 (No Magic Numbers), Rule 2 (Two-Tier Architecture),
//! Rule 4 (Mandatory Examples), and Rule 6 (Idiomatic Rust).

#![no_std]

use core::ffi::c_void;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use rust_esprit::delay_ms;
use rust_esprit::{logger, logger_init};
use rust_esprit::{DelayTimer, Pin, Timer};

logger_init!();

static DELAY_EXPIRED: AtomicBool = AtomicBool::new(false);
static IRQ_TICKS: AtomicU32 = AtomicU32::new(0);

const DELAY_US: u32 = 500; // 500 microseconds

#[cfg(any(feature = "rp2040"))]
const TIMER_PIN: Pin = Pin::GPIO10;
#[cfg(not(any(feature = "rp2040")))]
const TIMER_PIN: Pin = Pin::PB6;

unsafe extern "C" fn on_delay_timer_irq(_cookie: *mut c_void) {
    DELAY_EXPIRED.store(true, Ordering::SeqCst);
    IRQ_TICKS.fetch_add(1, Ordering::Relaxed);
}

#[unsafe(no_mangle)]
extern "C" fn user_init() {
    logger!("Starting Timer & DelayTimer test...\n");

    // 1. Single-shot hardware pulse via Timer
    let mut timer = Timer::from_pin(TIMER_PIN);
    logger!("Generating 10ms single-shot pulse...\n");
    timer.single_shot(10, true);

    // 2. Hardware microsecond delay timer with update interrupt callback
    if let Ok(mut dt) = DelayTimer::new(1) {
        dt.set_interrupt(Some(on_delay_timer_irq), core::ptr::null_mut());
        logger!("Arming DelayTimer for {} µs...\n", DELAY_US);
        dt.arm(DELAY_US);

        for _ in 0..10 {
            if DELAY_EXPIRED.load(Ordering::SeqCst) {
                logger!("DelayTimer expired! Total ticks: {}\n", IRQ_TICKS.load(Ordering::Relaxed));
                break;
            }
            delay_ms(1);
        }
    }

    logger!("--end--\n");
}
