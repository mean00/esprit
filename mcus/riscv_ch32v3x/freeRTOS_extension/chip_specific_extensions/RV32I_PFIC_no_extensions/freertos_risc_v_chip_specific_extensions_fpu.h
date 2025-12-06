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
 * RISC-V ISA), and code that tailors the port to a specific RISC-V chip:
 *
 * + FreeRTOS\Source\portable\GCC\RISC-V-RV32\portASM.S contains the code that
 *   is common to all currently supported RISC-V chips.  There is only one
 *   portASM.S file because the same file is built for all RISC-V target chips.
 *
 * + Header files called freertos_risc_v_chip_specific_extensions.h contain the
 *   code that tailors the FreeRTOS kernel's RISC-V port to a specific RISC-V
 *   chip.  There are multiple freertos_risc_v_chip_specific_extensions.h files
 *   as there are multiple RISC-V chip implementations.
 *
 * !!!NOTE!!!
 * TAKE CARE TO INCLUDE THE CORRECT freertos_risc_v_chip_specific_extensions.h
 * HEADER FILE FOR THE CHIP IN USE.  This is done using the assembler's (not the
 * compiler's!) include path.  For example, if the chip in use includes a core
 * local interrupter (CLINT) and does not include any chip specific register
 * extensions then add the path below to the assembler's include path:
 * FreeRTOS\Source\portable\GCC\RISC-V-RV32\chip_specific_extensions\RV32I_CLINT_no_extensions
 *
 */

#ifndef __FREERTOS_RISC_V_EXTENSIONS_H__
#define __FREERTOS_RISC_V_EXTENSIONS_H__

.altmacro

.macro fread rg offset
    	flw \rg, \offset * portWORD_SIZE( sp )
.endm

.macro fstore rg offset
    	fsw \rg, \offset * portWORD_SIZE( sp )
.endm


.macro CH32_FPU_STACK action
    \action f0, 1  
    \action f1, 2  
    \action f2, 3  
    \action f3, 4  
    \action f4, 5  
    \action f5, 6  
    \action f6, 7  
    \action f7, 8  
    \action f8, 9  
    \action f9, 10  
    \action f10, 11  
    \action f11, 12  
    \action f12, 13  
    \action f13, 14  
    \action f14, 15  
    \action f15, 16  
    \action f16, 17  
    \action f17, 18  
    \action f18, 19  
    \action f19, 20  
    \action f20, 21  
    \action f21, 22  
    \action f22, 23  
    \action f23, 24  
    \action f24, 25  
    \action f25, 26  
    \action f26, 27  
    \action f27, 28  
    \action f28, 29  
    \action f29, 30  
    \action f30, 31  
    \action f31, 0 
.endm


.macro portCheck_FPU_Dirty brnch
    csrr t0, mstatus
    srli t0,t0, 14         /* Bit 14:13 are FS bits, 1x means dirty */
    andi t0, t0, 1
    beq  t0,x0,\brnch  /* If 0, means not dirty */
.endm

.macro portasmSAVE_ADDITIONAL_REGISTERS
    portCheck_FPU_Dirty j00\@
    addi sp, sp, -(portFPU_SIZE) /* Only save FPU registers if FS bits are in dirty state*/
    CH32_FPU_STACK fstore
j00\@:    
	.endm

.macro portasmRESTORE_ADDITIONAL_REGISTERS
    portCheck_FPU_Dirty j01\@
    CH32_FPU_STACK fread
    addi sp, sp, (portFPU_SIZE)
j01\@:    
	.endm

#endif /* __FREERTOS_RISC_V_EXTENSIONS_H__ */
