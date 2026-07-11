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
 * The FreeRTOS kernel's RISC-V port is split between the the code that is
 * common across all currently supported RISC-V chips (implementations of the
 * RISC-V ISA), and code which tailors the port to a specific RISC-V chip:
 *
 * + The code that is common to all RISC-V chips is implemented in
 *   FreeRTOS\Source\portable\GCC\RISC-V-RV32\portASM.S.  There is only one
 *   portASM.S file because the same file is used no matter which RISC-V chip is
 *   in use.
 *
 * + The code that tailors the kernel's RISC-V port to a specific RISC-V
 *   chip is implemented in freertos_risc_v_chip_specific_extensions.h.  There
 *   is one freertos_risc_v_chip_specific_extensions.h that can be used with any
 *   RISC-V chip that both includes a standard CLINT and does not add to the
 *   base set of RISC-V registers.  There are additional
 *   freertos_risc_v_chip_specific_extensions.h files for RISC-V implementations
 *   that do not include a standard CLINT or do add to the base set of RISC-V
 *   registers.
 *
 * CARE MUST BE TAKEN TO INCLDUE THE CORRECT
 * freertos_risc_v_chip_specific_extensions.h HEADER FILE FOR THE CHIP
 * IN USE.  To include the correct freertos_risc_v_chip_specific_extensions.h
 * header file ensure the path to the correct header file is in the assembler's
 * include path.
 *
 * This freertos_risc_v_chip_specific_extensions.h is for use on RISC-V chips
 * that include a standard CLINT and do not add to the base set of RISC-V
 * registers.
 *
 */
#include "port_define.h"
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
    	load_x \rg, \offset * portWORD_SIZE( sp )
.endm
//------------------------
.macro xstore rg offset
    	store_x \rg, \offset * portWORD_SIZE( sp )
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
	load_x  t0, pxCurrentTCB			/* Load pxCurrentTCB. */
	store_x  sp, 0( t0 )				/* Write sp to first TCB member. */

	csrr   a1, mepc
	xstore a1,  0 						/* Save updated exception return address. */

	csrr t0, mstatus					/* Required for MPIE bit. */
	xstore t0,  1 

	addi a1, x0, 0x20  					// Disable HW stack 
	csrs 0x804, a1

	load_x sp, xISRStackTop				/* Switch to ISR stack before function call. */
	jal vTaskSwitchContext

processed_source:
	load_x  t1, pxCurrentTCB			/* Load pxCurrentTCB. */
	load_x  sp, 0( t1 )				 	/* Read sp from first TCB member. */

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

/* if it is an assembly entry code, the SP offset value is determined by the assembly code,
but the C code is determined by the compiler, so we subtract 512 here as a reservation.
When entering the interrupt function of C code, the compiler automatically presses the stack
into the task stack. We can only change the SP value used by the calling function after switching
the interrupt stack.This problem can be solved by modifying the interrupt to the assembly entry,
and there is no need to reserve 512 bytes. You only need to switch the interrupt stack at the
beginning of the interrupt function */
	lw t0, xISRStackTop /* MEANX : The systick interrupt handler will switch to the SP pointed by mscratch */
	addi t0, t0, -512
	csrw mscratch,t0

	load_x  sp, pxCurrentTCB	/* Load pxCurrentTCB. */
	load_x  sp, 0( sp )		 	/* Read sp from first TCB member. */

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
