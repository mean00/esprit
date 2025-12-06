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
	.word   0x1234; /* base +4 */
	.word   0x5678  /* base +8 */

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
2:
	
2:
	li t0, 0x1f
	csrw 0xbc0, t0
	J start_c
	mret


