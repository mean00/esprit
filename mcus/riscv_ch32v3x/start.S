/********************************** (C) COPYRIGHT *******************************
* File Name          : startup_ch32v30x_D8C.s
* Author             : WCH
* Version            : V1.0.0
* Date               : 2021/06/06
* Description        : CH32V307-CH32V305 vector table for eclipse toolchain.
* Copyright (c) 2021 Nanjing Qinheng Microelectronics Co., Ltd.
* SPDX-License-Identifier: Apache-2.0
*******************************************************************************/

	.section	.init,"ax",@progbits
	.global	_start	    
	.align	1
	.extern start_c
_start:
	j	handle_reset
	.word   0x1234; /* Notify the bootloader not to verify the signature but there is an app here */
	.word   0x5678  /* Notify the bootloader not to verify the signature but there is an app here */

	.align  1
_vector_base:
	.option norvc;
	.section	.text.handle_reset,"ax",@progbits	
	.align	1
handle_reset:
.option push 
.option	norelax 
	la gp, __global_pointer$
.option	pop 
1:
	la sp, _bss_end
	li t0, 0x1f
	csrw 0xbc0, t0 /* Bumblebee-core CSR 0xbc0 (MINTSTATUS): enable all 5 interrupt lines (SW, timer, external). */
	J start_c
	mret


