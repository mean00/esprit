/*
 * CH32V3x-specific FreeRTOS context definitions.
 *
 * This file defines the layout of the context saved on the stack during
 * a context switch.  CH32V3x is always a 32-bit RISC-V target, so all
 * register widths are hardwired to 32 bits.
 *
 * Included by both port.c and portASM.S.
 */
#pragma once

#define portCONTEXT_COUNT 28
#define portHEADER_COUNT 2
#if ARCH_FPU == 1
#define portFPU_COUNT 32
#else
#define portFPU_COUNT 0
#endif

/* Derived sizes (portWORD_SIZE is always 4 on this 32-bit target). */
#define portCONTEXT_SIZE (portCONTEXT_COUNT * 4)
#define portHEADER_SIZE (portHEADER_COUNT * 4)
#define portFPU_SIZE (portFPU_COUNT * 4)

/* FPU status field values (mstatus bits 14:13). */
#define CH32_FPU_OFF 0
#define CH32_FPU_INITIAL 1
#define CH32_FPU_CLEAN 2
#define CH32_FPU_DIRTY 3

#define CH32_FPU_STATE(yy) ((yy) << 13)
#define CH32_FPU_MASK(x) (~(3 << 13))
