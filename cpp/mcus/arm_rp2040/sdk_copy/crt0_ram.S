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
.word 0 // we may need to decode pointers that are in RAM at runtime.
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
_reset_handler:

  // This is a full ram build, loaded to ram and exectuted in ram
  // Make sure we have everything setup
    ldr r0, =2 // initial control, be sure we are in process mode
    msr control, r0
    cpsid i // disable interrupt
    ldr r0, =0x20041ff0 // initial msp
    msr msp, r0
    ldr r0, =0x20041ef0 // initial psp
    msr psp, r0
    ldr r2, =main_ram
    bx r2
