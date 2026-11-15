/*
 * FreeRTOS Kernel V10.4.6
 * Copyright (C) 2021 Amazon.com, Inc. or its affiliates.  All Rights Reserved.
 *
 * SPDX-License-Identifier: MIT
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 * FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 * COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 * IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 *
 * https://www.FreeRTOS.org
 * https://github.com/FreeRTOS
 *
 */

/*
 * CH32V3x FreeRTOS context-switch assembly.
 *
 * SW_Handler performs the full context save/restore using the Bumblebee
 * core's native GPR/FPU stack frame layout.  xPortStartFirstTask loads
 * the first task's initial stack and starts it with interrupts enabled.
 *
 * FPU support (if enabled via ARCH_FPU) is delegated to the
 * freertos_risc_v_chip_specific_extensions_fpu.h header.
 */
#include "port_common.h"
/* we define float registers at specific_extensions.h, V30x support hardware floating point unit
can be enable by defined ARCH_FPU*/
#if ARCH_FPU
	#include "freertos_risc_v_chip_specific_extensions_fpu.h"
#else
	#include "freertos_risc_v_chip_specific_extensions_no_fpu.h"
#endif



.global xPortStartFirstTask
.global SW_Handler
.global pxPortInitialiseStack_asm

.extern pxCurrentTCB
.extern vTaskSwitchContext
.extern xISRStackTop
/*---------------- */
// xstore
// xread
//------------------------
.macro xread rg offset
    lw \rg, \offset * 4( sp )
.endm
//------------------------
.macro xstore rg offset
    sw \rg, \offset * 4( sp )
.endm
//------------------------
.macro CH32_GPR_STACK action
	\action x1, 0
	\action x5, 1 
	\action x6, 2 
	\action x7, 3 
	\action x8, 4 
	\action x9, 5 
	\action x10, 6 
	\action x11, 7 
	\action x12, 8 
	\action x13, 9 
	\action x14, 10 
	\action x15, 11 
	\action x16, 12 
	\action x17, 13 
	\action x18, 14 
	\action x19, 15 
	\action x20, 16 
	\action x21, 17 
	\action x22, 18 
	\action x23, 19 
	\action x24, 20 
	\action x25, 21 
	\action x26, 22 
	\action x27, 23 
	\action x28, 24 
	\action x29, 25 
	\action x30, 26 
	\action x31, 27 
.endm


/*-----------------------------------------------------------*/
.align 8
SW_Handler:
	addi sp, sp, -portCONTEXT_SIZE  // GPR size
	CH32_GPR_STACK xstore	

	portasmSAVE_ADDITIONAL_REGISTERS	// FPU if used

	addi sp, sp, -portHEADER_SIZE 		// MEPC + MSTATUS
	lw  t0, pxCurrentTCB			/* Load pxCurrentTCB. */
	sw  sp, 0( t0 )				/* Write sp to first TCB member. */

	csrr   a1, mepc
	xstore a1,  0 						/* Save updated exception return address. */

	csrr t0, mstatus					/* Required for MPIE bit. */
	xstore t0,  1 

	addi a1, x0, 0x20  					// Disable HW stack 
	csrs 0x804, a1 /* INTSYSCR: Interrupt System Control Register (Bumblebee core, CSR 0x804) -- disable hardware stack while switching context. */

	lw sp, xISRStackTop				/* Switch to ISR stack before function call. */
	jal vTaskSwitchContext

processed_source:
	lw  t1, pxCurrentTCB			/* Load pxCurrentTCB. */
	lw  sp, 0( t1 )				 	/* Read sp from first TCB member. */

	/* Load mret with the address of the next instruction in the task to run next. */
	xread t0, 0
	csrw mepc, t0
	xread t0, 1
	csrw mstatus, t0						/* Required for MPIE bit. */

	addi sp, sp, portHEADER_SIZE
	portasmRESTORE_ADDITIONAL_REGISTERS	/* Defined in freertos_risc_v_chip_specific_extensions.h to restore any registers unique to the RISC-V implementation. */	
	CH32_GPR_STACK xread
	addi sp, sp, portCONTEXT_SIZE

	mret
	
/*-----------------------------------------------------------*/

.align 8

xPortStartFirstTask:

/*
 * Assembly entry code computes SP offsets at assembly-time; C code leaves them
 * to the compiler, so we reserve 512 bytes here as a safety margin.
 *
 * When a C interrupt function is entered, the compiler automatically pushes
 * registers onto the task stack.  We can only change the SP value used by the
 * calling function *after* switching to the interrupt stack.  Using an assembly
 * entry point (as we do for SW_Handler) avoids this problem entirely -- no 512
 * byte reservation is needed, and the interrupt stack switch happens at the
 * very beginning of the handler.
 */
	lw t0, xISRStackTop /* The SysTick handler will switch SP to the value held in mscratch. */
	addi t0, t0, -512
	csrw mscratch,t0

	lw  sp, pxCurrentTCB	/* Load pxCurrentTCB. */
	lw  sp, 0( sp )		 	/* Read sp from first TCB member. */

	xread  x1, 0 				/* MEPC Note for starting the scheduler the exception return address is used as the function return address. */
	xread  x5, 1 				/* MSTATUS Initial mstatus into x5 (t0) */
	ori   x5, x5, 0x08			/* Set MIE bit so the first task starts with interrupts enabled - required as returns with ret not eret. */
	csrrw  x0, mstatus, x5		/* Interrupts enabled from here! , dont touch the other bits*/

	addi sp, sp, portHEADER_SIZE	

	xread x10, 6 /* a0 */
	xread x5,  1 /* t0 */
	addi	sp, sp, portCONTEXT_SIZE
	ret
// EOF
