//! RP2040 IRQ helpers and `__unhandled_user_irq` fallback.

pub use rs_rp2xxx::irq::{enter_critical_from_isr, exit_critical_from_isr, in_isr, yield_from_isr};

/// Fallback handler referenced by pico-sdk `hardware_irq/irq.c` on RP2040.
#[unsafe(no_mangle)]
pub extern "C" fn __unhandled_user_irq() -> ! {
    loop {
        core::hint::spin_loop();
    }
}
