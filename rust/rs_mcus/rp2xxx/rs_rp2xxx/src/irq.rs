//! Shared Cortex-M ISR / critical-section / PendSV helpers for RP2040 / RP2350.

const SCB_ICSR: *mut u32 = 0xE000_ED04 as *mut u32;
const PENDSVSET_BIT: u32 = 1 << 28;

#[inline(always)]
pub fn in_isr() -> bool {
    let ipsr: u32;
    // SAFETY: Reading IPSR has no side effects.
    unsafe { core::arch::asm!("mrs {}, ipsr", out(reg) ipsr, options(nomem, nostack, preserves_flags)) };
    ipsr != 0
}

#[inline(always)]
pub fn enter_critical_from_isr() -> u32 {
    let old_primask: u32;
    // SAFETY: Saves PRIMASK and disables interrupts on Cortex-M0+/M33.
    unsafe {
        core::arch::asm!(
            "mrs {old}, primask",
            "cpsid i",
            "dsb",
            "isb",
            old = out(reg) old_primask,
            options(nomem, nostack, preserves_flags)
        );
    }
    old_primask
}

#[inline(always)]
pub fn exit_critical_from_isr(old_state: u32) {
    // SAFETY: Restores PRIMASK saved by `enter_critical_from_isr`.
    unsafe {
        core::arch::asm!(
            "msr primask, {old}",
            old = in(reg) old_state,
            options(nomem, nostack, preserves_flags)
        );
    }
}

#[inline(always)]
pub fn yield_from_isr() {
    // SAFETY: `SCB_ICSR` is the Cortex-M Interrupt Control and State Register.
    unsafe {
        core::ptr::write_volatile(SCB_ICSR, PENDSVSET_BIT);
        core::arch::asm!("dsb", "isb", options(nomem, nostack, preserves_flags));
    }
}
