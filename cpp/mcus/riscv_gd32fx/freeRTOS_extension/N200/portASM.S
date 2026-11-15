#include "riscv_encoding.h"
#include "riscv_bits.h"
//#include "n200_timer.h"
//#include "n200_eclic.h"
#include "port_extra.h"
#include "portASM_macro.S"


	.section .text.entry
	.align 4

	.global vPortYield
	.global vPortYield_from_ulSynchTrap
	.global xPortStartScheduler
	.global vPortEndScheduler


.macro setUnderInterrupt x
	li t1, \x
	la t0, underInterrupt
        sw t1,(t0)
.endm


/* -------------------------------------------------------------------------------------------------------- */
/**
 * @brife trap入口函数
 */
	.global trap_entry
	.align 6
trap_entry:
#if USE_MSP
	csrrw sp, CSR_MSCRATCHCSWL, sp
#endif
	pushREGFILE sp  
#ifdef USE_MSP
        csrr a1, CSR_MSUBM
        andi a1, a1, (0x3 << 8)
        bnez a1, 8f
	csrr a1, CSR_MSCRATCH
	j 9f
8:
#endif
	mv a1, sp 
#ifdef __riscv_flen
	addi a1, a1, REGBYTES * 68   //36+32
#else
	addi a1, a1, REGBYTES * 36
#endif
9:
	STORE a1, 3 * REGBYTES(sp)                                     
	portSAVE_CONTEXT_EXCP sp

	csrr a0, mcause
	mv a1, sp
	jal ulSynchTrap
	mv sp, a0

	portRESTORE_CONTEXT_EXCP sp
	popREGFILE sp
#if USE_MSP
	csrrw sp, CSR_MSCRATCHCSWL, sp
#endif
	mret


/* -------------------------------------------------------------------------------------------------------- */
/**
 * @brife irq入口函数
 */
    .align 2
    .global irq_entry
irq_entry:
#if USE_MSP
	csrrw sp, CSR_MSCRATCHCSWL, sp
#endif
  	pushREGFILE sp
	portSAVE_CONTEXT_EXCP sp
#ifdef __riscv_flen
	csrr t2, mstatus
	li t0, (0x3 << 13)
	and t1,t2,t0
	bne t1,t0,1f
	pushVFPREGFILE sp
1:	
	CONFIG_FS_CLEAN  
#endif
// Also mark we are under interrupt 
zzz:
        setUnderInterrupt(1)
int_loop:
  	csrrw ra, CSR_JALMNXTI, ra

  	csrc CSR_MSTATUS, MSTATUS_MIE

#ifdef __riscv_flen
	csrr t2, mstatus
	li t0, (0x3 << 13)
	and t1,t2, t0
	bne t1,t0, 2f
	popVFPREGFILE sp
2:
#endif
        setUnderInterrupt(0)
	portRESTORE_CONTEXT_EXCP sp
  	popREGFILE sp
#if USE_MSP
	csrrw sp, CSR_MSCRATCHCSWL, sp
#endif
  	mret


/* -------------------------------------------------------------------------------------------------------- */
/**
 * @brife MTIME入口函数
 */
    .align 2
    .globl eclic_mtip_handler
eclic_mtip_handler:
#if USE_MSP
	csrrw sp, CSR_MSCRATCHCSWL, sp
#endif
  	pushREGFILE sp
	portSAVE_CONTEXT_EXCP sp 

	csrs CSR_MSTATUS, MSTATUS_MIE
	jal	vPortSysTickHandler
	csrc CSR_MSTATUS, MSTATUS_MIE
	 
	portRESTORE_CONTEXT_EXCP sp
	popREGFILE sp
#if USE_MSP
	csrrw sp, CSR_MSCRATCHCSWL, sp
#endif
	mret


/* -------------------------------------------------------------------------------------------------------- */
/**
 * @brife MSIP入口函数
 */
	.align 2
	.globl eclic_msip_handler

eclic_msip_handler:
	pushREGFILE sp
	portSAVE_CONTEXT_EXCP sp

	mv a0,sp
	call vPortClearMSIPInt
	jal taskswitch

	portRESTORE_CONTEXT_EXCP sp
	popREGFILE sp
	mret


/* -------------------------------------------------------------------------------------------------------- */
/**
 * @brife Trap模式请求切换任务函数 Mode request switching task function 
 */
    .align 6
vPortYield_from_ulSynchTrap:
	mv sp, a0
	portSAVE_CONTEXT_EXCP sp

	j _vPortYield


/* -------------------------------------------------------------------------------------------------------- */
/**
 * @brife MSIP模式请求切换任务函数
 */
    .align 6
vPortYield:
	mv sp, a0
_vPortYield:
	LOAD t0, pxCurrentTCB
	STORE sp, 0x0(t0)

#ifdef __riscv_flen
	csrr t2, mstatus
	li t0, (0x3 << 13)
	and t1,t2,t0
	bne t1,t0,1f
	pushVFPREGFILE sp
1:
	CONFIG_FS_CLEAN
#endif
	STORE a1, 33 * REGBYTES(sp)

#if USE_MSP
	csrr sp, CSR_MSCRATCH
#endif
	csrs CSR_MSTATUS, MSTATUS_MIE
	jal vDoTaskSwitchContext
	csrc CSR_MSTATUS, MSTATUS_MIE

	LOAD sp, pxCurrentTCB
	LOAD sp, 0x0(sp)

	portRESTORE_CONTEXT_EXCP sp
#ifdef __riscv_flen
	csrr t2, mstatus
	li t0, (0x3 << 13)
	and t1, t2, t0
	bne t1, t0, 2f
	popVFPREGFILE sp
2:
#endif
	popREGFILE sp
	mret


/* -------------------------------------------------------------------------------------------------------- */
/**
 * @brife freertos启动调度函数 Start scheduling function 
 */
xPortStartScheduler:
	jal	vPortSetup
	csrc CSR_MSTATUS, MSTATUS_MIE

#if USE_MSP
	la t0, _sp
	csrw CSR_MSCRATCH, t0
#endif

	LOAD sp, pxCurrentTCB
	LOAD sp, 0x0(sp)

	portRESTORE_CONTEXT_EXCP sp                            

	popREGFILE sp
	mret


/* -------------------------------------------------------------------------------------------------------- */
/**
 * @brife MSIP模式请求切换任务函数 Mode request switching task function
 */
vPortEndScheduler:
 	j vPortEndScheduler
