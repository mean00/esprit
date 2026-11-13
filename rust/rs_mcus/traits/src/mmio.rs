//! Concise volatile MMIO register access macros.
//!
//! All macros operate on raw register-block pointers (`*const T` / `*mut T`) via
//! `core::ptr::addr_of!` / `core::ptr::addr_of_mut!` so no intermediate `&mut`
//! reference to MMIO memory is ever created.
//!
//! `unsafe` is intentionally kept at the call site rather than hidden inside the
//! macros.

/// Volatile read of `$field` (or `$field[$idx]`) on a register block pointer.
#[macro_export]
macro_rules! reg_read {
    ($regs:expr, $field:ident) => {
        core::ptr::read_volatile(core::ptr::addr_of!((*$regs).$field))
    };
    ($regs:expr, $field:ident [$idx:expr]) => {
        core::ptr::read_volatile(core::ptr::addr_of!((*$regs).$field[$idx]))
    };
}

/// Volatile write to `$field` (or `$field[$idx]`) on a register block pointer.
#[macro_export]
macro_rules! reg_write {
    ($regs:expr, $field:ident, $val:expr) => {
        core::ptr::write_volatile(core::ptr::addr_of_mut!((*$regs).$field), $val)
    };
    ($regs:expr, $field:ident [$idx:expr], $val:expr) => {
        core::ptr::write_volatile(core::ptr::addr_of_mut!((*$regs).$field[$idx]), $val)
    };
}

/// Volatile read-modify-write of `$field` (or `$field[$idx]`), evaluating `$regs` once.
#[macro_export]
macro_rules! reg_modify {
    ($regs:expr, $field:ident, $f:expr) => {{
        let ptr = core::ptr::addr_of_mut!((*$regs).$field);
        let val = $f(core::ptr::read_volatile(ptr));
        core::ptr::write_volatile(ptr, val);
    }};
    ($regs:expr, $field:ident [$idx:expr], $f:expr) => {{
        let ptr = core::ptr::addr_of_mut!((*$regs).$field[$idx]);
        let val = $f(core::ptr::read_volatile(ptr));
        core::ptr::write_volatile(ptr, val);
    }};
}

/// Raw `*mut` pointer to `$field` (or `$field[$idx]`) on a register block pointer.
#[macro_export]
macro_rules! reg_ptr {
    ($regs:expr, $field:ident) => {
        core::ptr::addr_of_mut!((*$regs).$field)
    };
    ($regs:expr, $field:ident [$idx:expr]) => {
        core::ptr::addr_of_mut!((*$regs).$field[$idx])
    };
}

/// Generate crate-local `read_reg!`, `write_reg!`, and `modify_reg!` macros
/// bound to a singleton peripheral pointer expression (e.g. `RcuRegisters::ptr()`).
#[macro_export]
macro_rules! define_mmio_accessors {
    ($regs:expr) => {
        #[allow(unused_macros)]
        macro_rules! read_reg {
            ($field:ident) => {
                $crate::reg_read!($regs, $field)
            };
            ($field:ident [$idx:expr]) => {
                $crate::reg_read!($regs, $field[$idx])
            };
        }

        #[allow(unused_macros)]
        macro_rules! write_reg {
            ($field:ident, $val:expr) => {
                $crate::reg_write!($regs, $field, $val)
            };
            ($field:ident [$idx:expr], $val:expr) => {
                $crate::reg_write!($regs, $field[$idx], $val)
            };
        }

        #[allow(unused_macros)]
        macro_rules! modify_reg {
            ($field:ident, $f:expr) => {
                $crate::reg_modify!($regs, $field, $f)
            };
            ($field:ident [$idx:expr], $f:expr) => {
                $crate::reg_modify!($regs, $field[$idx], $f)
            };
        }
    };
}
