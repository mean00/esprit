//! C++ `lnSetInterruptHandler` / `lnEnableInterrupt` / `lnDisableInterrupt` ABI for RP2350.

use core::sync::atomic::{Ordering, fence};

pub use rs_rp2xxx::irq::{enter_critical_from_isr, exit_critical_from_isr, in_isr, yield_from_isr};

unsafe extern "C" {
    static mut ram_vector_table: [u32; 48];
    fn irq_set_enabled(num: u32, enabled: bool);
}

#[unsafe(export_name = "_Z21lnSetInterruptHandlerRK5LnIRQPFvvE")]
pub unsafe extern "C" fn ln_set_interrupt_handler(irq: *const i32, handler: Option<extern "C" fn()>) {
    // SAFETY: `irq` is a valid C++ `const LnIRQ &` and `real_irq` indexes into `ram_vector_table`.
    let real_irq = (unsafe { *irq } + 16) as usize;
    let addr = handler.map_or(0u32, |f| f as usize as u32);
    // SAFETY: `ram_vector_table` has 48 entries; `fence` issues a `dmb` barrier.
    unsafe {
        core::ptr::write_volatile(core::ptr::addr_of_mut!(ram_vector_table[real_irq]), addr);
    }
    fence(Ordering::SeqCst);
}

#[unsafe(export_name = "_Z17lnEnableInterruptRK5LnIRQ")]
pub unsafe extern "C" fn ln_enable_interrupt(irq: *const i32) {
    // SAFETY: `irq` is a valid C++ `const LnIRQ &`.
    unsafe { irq_set_enabled(*irq as u32, true) };
}

#[unsafe(export_name = "_Z18lnDisableInterruptRK5LnIRQ")]
pub unsafe extern "C" fn ln_disable_interrupt(irq: *const i32) {
    // SAFETY: `irq` is a valid C++ `const LnIRQ &`.
    unsafe { irq_set_enabled(*irq as u32, false) };
}
