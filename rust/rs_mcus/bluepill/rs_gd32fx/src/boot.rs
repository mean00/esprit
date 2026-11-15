//! GD32F3x C/C++ runtime startup (`drop_c_part3.md` Phase G5).
//!
//! Replaces `esprit/cpp/mcus/arm_gd32fx/src/start.cpp` in `LN_ENABLE_RUST`
//! builds while leaving the C++ file untouched for `LN_ENABLE_RUST=OFF`.

use core::arch::global_asm;
use core::ptr::{read_volatile, write_volatile};

global_asm!(
    r#"
    .section .text.__start__, "ax", %progbits
    .code 16
    .thumb_func
    .globl __start__
    .globl __trap
    .type __start__, %function
    .type __trap, %function
__start__:
    ldr r1, =__msp_init-128
    mov sp, r1
    ldr r1, =start_c
    bx r1
    .pool
__trap:
    bkpt #1
    b __trap

    .section .stm32.interrupt_vector, "a", %progbits
    .globl __stm32_vector_table
    .type __stm32_vector_table, %object
__stm32_vector_table:
    .long __msp_init-4
    .long __exc_reset
    .word 0x1234
    .word 0x5678
    .size __stm32_vector_table, . - __stm32_vector_table
"#
);

unsafe extern "C" {
    static _lm_rom_img_cfgp: u32;
    static mut __data_start__: u32;
    static mut __data_end__: u32;
    static mut __bss_start__: u32;
    static mut __bss_end__: u32;

    fn __libc_init_array();
    fn main() -> i32;
}

/// Stub `_init` required by `__libc_init_array`.
#[unsafe(no_mangle)]
pub extern "C" fn _init() {}

/// `extern "C" void start_c()` — copies `.data`, zeroes `.bss`, runs C++ ctors, enters `main`.
#[unsafe(no_mangle)]
pub extern "C" fn start_c() -> ! {
    // SAFETY: Linker script guarantees 4-byte alignment of `.data` and `.bss` bounds
    // and `_lm_rom_img_cfgp` holds the LMA of `.data`.
    unsafe {
        let mut src = read_volatile(&raw const _lm_rom_img_cfgp) as *const u32;
        let mut dst = &raw mut __data_start__;
        let end = &raw mut __data_end__;
        while dst < end {
            write_volatile(dst, read_volatile(src));
            dst = dst.add(1);
            src = src.add(1);
        }

        let mut z = &raw mut __bss_start__;
        let zend = &raw mut __bss_end__;
        while z < zend {
            write_volatile(z, 0);
            z = z.add(1);
        }

        __libc_init_array();
        let _ = main();
    }
    panic!("main() returned");
}
