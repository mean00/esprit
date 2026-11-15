/*
 * Copyright (c) 2020 Raspberry Pi (Trading) Ltd.
 *
 * SPDX-License-Identifier: BSD-3-Clause

NDEBUG
COLLAPSE_IRQS
PICO_NO_BINARY_INFO
PICO_NO_FLASH             <= 
PICO_ASSEMBLER_IS_CLANG   <= SET

 */

#include "pico.h"
#include "pico/asm_helper.S"

#include "hardware/regs/m0plus.h"
#include "hardware/regs/addressmap.h"
#include "hardware/regs/sio.h"
#include "pico/binary_info/defs.h"

#ifdef NDEBUG
#ifndef COLLAPSE_IRQS
#define COLLAPSE_IRQS
#endif
#endif

pico_default_asm_setup

.section .vectors, "ax"
.align 2

.global __vectors, __VECTOR_TABLE
__VECTOR_TABLE:
__vectors:
.word __StackTop
.word _reset_handler
// all default exception handlers do nothing, and we can check for them being set to our
// default values by seeing if they point to somewhere between __defaults_isrs_start and __default_isrs_end
.global __default_isrs_start
__default_isrs_start:
.global __default_isrs_end
__default_isrs_end:

// ----------------------------------------------------------------------------

.section .binary_info_header, "a"

// Header must be in first 256 bytes of main image (i.e. excluding flash boot2).
// For flash builds we put it immediately after vector table; for NO_FLASH the
// vectors are at a +0x100 offset because the bootrom enters RAM images directly
// at their lowest address, so we put the header in the VTOR alignment hole.

#if !PICO_NO_BINARY_INFO
binary_info_header:
.word BINARY_INFO_MARKER_START
.word __binary_info_start
.word __binary_info_end
.word data_cpy_table // we may need to decode pointers that are in RAM at runtime.
.word BINARY_INFO_MARKER_END
#endif

// ----------------------------------------------------------------------------

.section .reset, "ax"

// On flash builds, the vector table comes first in the image (conventional).
// On NO_FLASH builds, the reset handler section comes first, as the entry
// point is at offset 0 (fixed due to bootrom), and VTOR is highly-aligned.
// Image is entered in various ways:
//
// - NO_FLASH builds are entered from beginning by UF2 bootloader
//
// - Flash builds vector through the table into _reset_handler from boot2
//
// - Either type can be entered via _entry_point by the debugger, and flash builds
//   must then be sent back round the boot sequence to properly initialise flash

// ELF entry point:
.type _entry_point,%function
.thumb_func
.global _entry_point
_entry_point:

    // Debugger tried to run code after loading, so SSI is in 03h-only mode.
    // Go back through bootrom + boot2 to properly initialise flash.
    movs r0, #0
    ldr r1, =(PPB_BASE + M0PLUS_VTOR_OFFSET)
    str r0, [r1]
    ldmia r0!, {r1, r2}
    msr msp, r1
    bx r2

// Reset handler:
// - initialises .data
// - clears .bss
// - calls runtime_init
// - calls main
// - calls exit (which should eventually hang the processor via _exit)

.type _reset_handler,%function
.thumb_func
_reset_handler:
    // Only core 0 should run the C runtime startup code; core 1 is normally
    // sleeping in the bootrom at this point but check to be sure
    ldr r0, =(SIO_BASE + SIO_CPUID_OFFSET)
    ldr r0, [r0]
    cmp r0, #0
    bne hold_non_core0_in_bootrom

    adr r4, data_cpy_table

    // assume there is at least one entry
1:
    ldmia r4!, {r1-r3}
    cmp r1, #0
    beq 2f
    bl data_cpy
    b 1b
2:

bss_header_start:
    // Zero out the BSS
    //--------------------------
    // Header to align to 64
    //--------------------------
    ldr r1, =__bss_start__
    ldr r2, =__bss_end__
    movs r0, #0

    // Header to align to 64
    ldr r4, = 63 // 64 round down
3:
    mov r5, r4 // 64 round down
    ands r5, r1
    beq bss_header_done
    stmia r1!,{r0} 
    b 3b
    //--------------------------
    // No r1 is 64 aligned
    // Body 64 bytes at a time 
    //--------------------------

bss_header_done:
    ldr r0, = 0xffffffc0 // 64 round down
    ands r2, r0          // round down end address
    movs r3, #0
    movs r4, #0
    movs r5, #0
    movs r6, #0
    b bss_fill_test
bss_fill_loop:                // 4x16 bytes = 64 bytes at a time
    stmia r1!, {r3,r4,r5,r6}  // 16 bytes at a time
    stmia r1!, {r3,r4,r5,r6}  // 16 bytes at a time
    stmia r1!, {r3,r4,r5,r6}  // 16 bytes at a time
    stmia r1!, {r3,r4,r5,r6}  // 16 bytes at a time
bss_fill_test:
    cmp r1, r2
    bne bss_fill_loop
    //--------------------------
    // Trailer to align to 64
    //--------------------------

    ldr r2, =__bss_end__
    b bss_trailer_end
    
bss_trailer: 
    stmia r1!,{r3}
bss_trailer_end:
    cmp r1, r2
    bne bss_trailer


platform_entry: // symbol for stack traces
    // Use 32-bit jumps, in case these symbols are moved out of branch range
    // (e.g. if main is in SRAM and crt0 in flash)
    ldr r1, =runtime_init
    blx r1
    ldr r1, =main
    blx r1
    ldr r1, =exit
    blx r1
    // exit should not return.  If it does, hang the core.
    // (fall thru into our hang _exit impl
1: // separate label because _exit can be moved out of branch range
    bkpt #0
    b 1b

data_cpy_loop:
    ldm r1!, {r0}
    stm r2!, {r0}
data_cpy:
    cmp r2, r3
    blo data_cpy_loop
    bx lr

// Note the data copy table is still included for NO_FLASH builds, even though
// we skip the copy, because it is listed in binary info

.align 2
data_cpy_table:

.word __ln_copy_ram_text_source__
.word __ln_copy_ram_text_start__
.word __ln_copy_ram_text_end__

.word __etext
.word __data_start__
.word __data_end__

.word __scratch_x_source__
.word __scratch_x_start__
.word __scratch_x_end__

.word __scratch_y_source__
.word __scratch_y_start__
.word __scratch_y_end__


.word 0 // null terminator

// ----------------------------------------------------------------------------
// If core 1 somehow gets into crt0 due to a spectacular VTOR mishap, we need to
// catch it and send back to the sleep-and-launch code in the bootrom. Shouldn't
// happen (it should sleep in the ROM until given an entry point via the
// cross-core FIFOs) but it's good to be defensive.

hold_non_core0_in_bootrom:
    b hold_non_core0_in_bootrom
   
