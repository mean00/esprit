//! CH32V3x C/Rust runtime startup (`drop_c_part2.md` Phase C5).
//!
//! Replaces `esprit/cpp/mcus/riscv_ch32v3x/src/lnBoot_riscv_ch32.cpp` in
//! `LN_ENABLE_RUST` builds.

use core::arch::global_asm;
use crate::irq::deadEnd;

global_asm!(
    r#"
    .section .init,"ax",@progbits
    .global _start
    .align 1
_start:
    j handle_reset
    .word 0x1234
    .word 0x5678
    .align 1
    .section .text.handle_reset,"ax",@progbits
    .align 1
handle_reset:
    .option push
    .option norelax
    la gp, __global_pointer$
    .option pop
    la sp, _bss_end
    li t0, 0x1f
    csrw 0xbc0, t0
    j start_c
    mret
    "#
);

unsafe extern "C" {
    fn __libc_init_array();
    fn main() -> i32;
}

#[unsafe(link_section = ".isr_code")]
#[unsafe(no_mangle)]
pub extern "C" fn start_c() -> ! {
    // SAFETY: Linker script guarantees 4-byte alignment of `.data` and `.bss` bounds,
    // and `_data_lma` holds the ROM load address of `.data`.
    unsafe {
        core::arch::asm!(
            "lui  t0, %hi(_data_lma)",
            "addi t0, t0, %lo(_data_lma)",
            "lui  t1, %hi(_data_begin)",
            "addi t1, t1, %lo(_data_begin)",
            "lui  t2, %hi(_data_end)",
            "addi t2, t2, %lo(_data_end)",
            "beq  t1, t2, 4f",
            "li   t4, 32",
            "sub  t3, t2, t1",
            "blt  t3, t4, 2f",
            "1:",
            "lw   a0, 0(t0)",
            "lw   a1, 4(t0)",
            "lw   a2, 8(t0)",
            "lw   a3, 12(t0)",
            "lw   a4, 16(t0)",
            "lw   a5, 20(t0)",
            "lw   a6, 24(t0)",
            "lw   a7, 28(t0)",
            "sw   a0, 0(t1)",
            "sw   a1, 4(t1)",
            "sw   a2, 8(t1)",
            "sw   a3, 12(t1)",
            "sw   a4, 16(t1)",
            "sw   a5, 20(t1)",
            "sw   a6, 24(t1)",
            "sw   a7, 28(t1)",
            "addi t0, t0, 32",
            "addi t1, t1, 32",
            "sub  t3, t2, t1",
            "bge  t3, t4, 1b",
            "2:",
            "beq  t1, t2, 4f",
            "3:",
            "lw   t3, 0(t0)",
            "sw   t3, 0(t1)",
            "addi t0, t0, 4",
            "addi t1, t1, 4",
            "blt  t1, t2, 3b",
            "4:",
            "lui  t0, %hi(_bss_begin)",
            "addi t0, t0, %lo(_bss_begin)",
            "lui  t1, %hi(_bss_end)",
            "addi t1, t1, %lo(_bss_end)",
            "beq  t0, t1, 8f",
            "li   t4, 32",
            "sub  t2, t1, t0",
            "blt  t2, t4, 6f",
            "5:",
            "sw   x0, 0(t0)",
            "sw   x0, 4(t0)",
            "sw   x0, 8(t0)",
            "sw   x0, 12(t0)",
            "sw   x0, 16(t0)",
            "sw   x0, 20(t0)",
            "sw   x0, 24(t0)",
            "sw   x0, 28(t0)",
            "addi t0, t0, 32",
            "sub  t2, t1, t0",
            "bge  t2, t4, 5b",
            "6:",
            "beq  t0, t1, 8f",
            "7:",
            "sw   x0, 0(t0)",
            "addi t0, t0, 4",
            "blt  t0, t1, 7b",
            "8:",
            out("a0") _,
            out("a1") _,
            out("a2") _,
            out("a3") _,
            out("a4") _,
            out("a5") _,
            out("a6") _,
            out("a7") _,
            out("t0") _,
            out("t1") _,
            out("t2") _,
            out("t3") _,
            out("t4") _,
        );
        __libc_init_array();
        main();
    }
    deadEnd(0);
}
