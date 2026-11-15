/// The C entries of the clock/reset driver (`drop_C.md` P6).
///
/// `esprit/mcus/common_bluepill/lnRCU.cpp` is not compiled in a Rust build: this crate
/// owns the per-family APB/AHB tables. `esprit/include/lnRCU.h` turns the static
/// `lnPeripherals::` members into inline forwarders to these when `LN_RUST_BLUEPILL` is
/// defined, so the family C++ (`lnRCU_armgd32.cpp`, `lnRCU_systemClock_*`,
/// `lnUSBD*.cpp`, `lnRunTime_*`, the inline `lnTimerWatch`) is unchanged.
///
/// [`Peripheral`] carries the C++ `enum Peripherals` values verbatim (`pNONE = 0` ..
/// `pUSBFS_OTG_CH32v3x`, then the clock-only `pAPB1 = 100` .. `pSYSCLOCK`), which is why
/// the conversion is a range check plus a cast.

use crate::{disable, enable, reset, Peripheral};

/// `enum Peripherals` spans 0..=37 and then 100..=102 (`esprit/include/lnPeripherals.h`).
fn peripheral_of(value: i32) -> Option<Peripheral> {
    if (0..=37).contains(&value) || (100..=102).contains(&value) {
        // SAFETY: every one of those values is a declared discriminant.
        Some(unsafe { core::mem::transmute::<i32, Peripheral>(value) })
    } else {
        None
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lnPeripherals_enable(periph: i32) {
    if let Some(p) = peripheral_of(periph) {
        enable(p);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lnPeripherals_disable(periph: i32) {
    if let Some(p) = peripheral_of(periph) {
        disable(p);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lnPeripherals_reset(periph: i32) {
    if let Some(p) = peripheral_of(periph) {
        reset(p);
    }
}

/// `LN_RCU *arcu = (LN_RCU *)LN_RCU_ADR;` - the register pointer every family file
/// dereferences (`arcu->CFG0`, ...). `lnRCU.cpp` held it; the family C++ only reads it,
/// so it lives here as a plain pointer variable in `.data`.
#[unsafe(no_mangle)]
pub static mut arcu: *mut u32 = rs_bluepill::RCU_BASE as *mut u32;
