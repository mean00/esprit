// See LICENSE for license details.

#include "riscv_encoding.h"

    .section .init
	
    .weak  eclic_msip_handler
    .weak  eclic_mtip_handler
    .weak  eclic_bwei_handler
    .weak  eclic_pmovi_handler
    
#define dont(x) unhandledException##x: \
                .global unhandledException##x \
                la a0, x \
                jal unhandledException


#define UNHANDLED(x) .word unhandledException##x

vector_base:
    j _start
    .align    2
        UNHANDLED(0)
        UNHANDLED(1)
        .word     eclic_msip_handler //2 eclic_msip_handler This is the system call to cause a task switch
        UNHANDLED(3)
        UNHANDLED(4)
        UNHANDLED(5)
        .word  	  eclic_mtip_handler //6 eclic_mtip_handler This is the timer interrupt
        UNHANDLED(7)
        UNHANDLED(8)
        UNHANDLED(9)
        UNHANDLED(10)
        UNHANDLED(11)
        UNHANDLED(12)
        UNHANDLED(13)
        UNHANDLED(14)
        UNHANDLED(16) //
        UNHANDLED(17) //.word  	  unhandledException //16 eclic_bwei_handler
        UNHANDLED(18) //.word  	  unhandledException //17 eclic_pmovi_handler
        UNHANDLED(19) //.word  	  unhandledException //WWDGT_IRQHandler
	UNHANDLED(20) //.word     unhandledException //LVD_IRQHandler
	UNHANDLED(21) //.word     unhandledException //TAMPER_IRQHandler
	UNHANDLED(22) //.word     unhandledException //RTC_IRQHandler
	UNHANDLED(23) //.word     unhandledException //FMC_IRQHandler
	UNHANDLED(24) //.word     unhandledException //RCU_IRQHandler
	.word EXTI0_IRQHandler //.word     unhandledException //EXTI0_IRQHandler
	.word EXTI1_IRQHandler //.word     unhandledException //EXTI1_IRQHandler
	.word EXTI2_IRQHandler //.word     unhandledException //EXTI2_IRQHandler
	.word EXTI3_IRQHandler //.word     unhandledException //EXTI3_IRQHandler
	.word EXTI4_IRQHandler //.word     unhandledException //EXTI4_IRQHandler
	.word DMA0_Channel0_IRQHandler //.word     unhandledException //DMA0_Channel0_IRQHandler
	.word DMA0_Channel1_IRQHandler //.word     unhandledException //DMA0_Channel1_IRQHandler
	.word DMA0_Channel2_IRQHandler //.word     unhandledException //DMA0_Channel2_IRQHandler SPI0 TX
	.word DMA0_Channel3_IRQHandler //UNHANDLED(33) //.word     unhandledException //DMA0_Channel3_IRQHandler
	.word DMA0_Channel4_IRQHandler //UNHANDLED(34) //.word     unhandledException //DMA0_Channel4_IRQHandler
	.word DMA0_Channel5_IRQHandler //UNHANDLED(35) //.word     unhandledException //DMA0_Channel5_IRQHandler
	.word DMA0_Channel6_IRQHandler //UNHANDLED(36) //.word     unhandledException //DMA0_Channel6_IRQHandler
	UNHANDLED(37) //.word     unhandledException //ADC0_1_IRQHandler
	UNHANDLED(38) //.word     unhandledException //CAN0_TX_IRQHandler
	UNHANDLED(39) //.word     unhandledException //CAN0_RX0_IRQHandler
	UNHANDLED(40) //.word     unhandledException //CAN0_RX1_IRQHandler
	UNHANDLED(41) //.word     unhandledException //CAN0_EWMC_IRQHandler
	.word EXTI5_9_IRQHandler //.word     unhandledException //EXTI5_9_IRQHandler
	UNHANDLED(43) //.word     unhandledException //TIMER0_BRK_IRQHandler
	UNHANDLED(44) //.word     unhandledException //TIMER0_UP_IRQHandler
	UNHANDLED(45) //.word     unhandledException //TIMER0_TRG_CMT_IRQHandler
	UNHANDLED(46) //.word     unhandledException //TIMER0_Channel_IRQHandler
	.word TIMER1_IRQHandler // UNHANDLED(47) //.word     unhandledException //TIMER1_IRQHandler
	UNHANDLED(48) //.word     unhandledException //TIMER2_IRQHandler
	UNHANDLED(49) //.word     unhandledException //TIMER3_IRQHandler
	.word I2C0_EV_IRQHandler  //UNHANDLED(50) //.word     unhandledException //I2C0_EV_IRQHandler
	.word I2C0_ERR_IRQHandler  //UNHANDLED(51) //.word     unhandledException //I2C0_ER_IRQHandler
	.word I2C1_EV_IRQHandler   //.word     unhandledException //I2C1_EV_IRQHandler
	.word I2C1_ERR_IRQHandler //UNHANDLED(53) //.word     unhandledException //I2C1_ER_IRQHandler
	UNHANDLED(54) //.word     unhandledException //SPI0_IRQHandler
	UNHANDLED(55) //.word     unhandledException //SPI1_IRQHandler
	.word USART0_IRQHandler //.word     unhandledException-56 //USART0_IRQHandler
	.word USART1_IRQHandler //.word     unhandledException //USART1_IRQHandler
	.word USART2_IRQHandler //.word     unhandledException //USART2_IRQHandler
	.word EXTI10_15_IRQHandler //.word     unhandledException //EXTI10_15_IRQHandler
	UNHANDLED(60) //.word     unhandledException //RTC_Alarm_IRQHandler
	UNHANDLED(61) //.word     unhandledException //USBFS_WKUP_IRQHandler
        UNHANDLED(62) //.word     unhandledException //0
        UNHANDLED(63) //.word     unhandledException //0
        UNHANDLED(64) //.word     unhandledException //0
        UNHANDLED(65) //.word     unhandledException //0
	UNHANDLED(66) //.word     unhandledException //0
	UNHANDLED(67) //.word     unhandledException //EXMC_IRQHandler
	UNHANDLED(68) //.word     unhandledException //0
	UNHANDLED(69) //.word     unhandledException //TIMER4_IRQHandler
	UNHANDLED(70) //.word     unhandledException //SPI2_IRQHandler
	.word USART3_IRQHandler //UNHANDLED(71) //.word     unhandledException //UART3_IRQHandler
	UNHANDLED(72) //.word     unhandledException //UART4_IRQHandler
	.word TIMER5_IRQHandler //.word     unhandledException //TIMER5_IRQHandler
	.word TIMER6_IRQHandler //.word     unhandledException //TIMER6_IRQHandler
	.word DMA1_Channel0_IRQHandler //UNHANDLED(75) //.word     unhandledException //DMA1_Channel0_IRQHandler
	.word DMA1_Channel1_IRQHandler //UNHANDLED(76) //.word     unhandledException //DMA1_Channel1_IRQHandler
	.word DMA1_Channel2_IRQHandler //UNHANDLED(77) //.word     unhandledException //DMA1_Channel2_IRQHandler
	.word DMA1_Channel3_IRQHandler //UNHANDLED(78) //.word     unhandledException //DMA1_Channel3_IRQHandler
	.word DMA1_Channel4_IRQHandler //UNHANDLED(79) //.word     unhandledException //DMA1_Channel4_IRQHandler
        UNHANDLED(80) //.word     unhandledException //0
        UNHANDLED(81) //.word     unhandledException // 0
	UNHANDLED(82) //.word     unhandledException //CAN1_TX_IRQHandler
	UNHANDLED(83) //.word     unhandledException //CAN1_RX0_IRQHandler
	UNHANDLED(84) //.word     unhandledException //CAN1_RX1_IRQHandler
	UNHANDLED(85) //.word     unhandledException //CAN1_EWMC_IRQHandler
	UNHANDLED(86) //.word     unhandledException //USBFS_IRQHandler

	.globl _start
	.type _start,@function

_start:

    csrc CSR_MSTATUS, MSTATUS_MIE
	/* Jump to logical address first to ensure correct operation of RAM region  */
    la		a0,	_start
    li		a1,	1
    slli	a1,	a1, 29
    bleu	a1, a0, _start0800
    srli	a1,	a1, 2
    bleu	a1, a0, _start0800
    la		a0,	_start0800
    add		a0, a0, a1
	jr      a0

_start0800:

    /* Set the the NMI base to share with mtvec by setting CSR_MMISC_CTL */
    li t0, 0x200
    csrs CSR_MMISC_CTL, t0

	/* Intial the mtvt*/
    la t0, vector_base
    csrw CSR_MTVT, t0

	/* Intial the mtvt2 and enable it*/
    la t0, irq_entry
    csrw CSR_MTVT2, t0
    csrs CSR_MTVT2, 0x1

    /* Intial the CSR MTVEC for the Trap ane NMI base addr*/
    la t0, trap_entry
    csrw CSR_MTVEC, t0

#ifdef __riscv_flen
	/* Enable FPU */
	li t0, MSTATUS_FS
	csrs mstatus, t0
	csrw fcsr, x0
#endif

.option push
.option norelax
	la gp, __global_pointer$
.option pop
	la sp, _sp

	/* Load data section */
	la a0, _data_lma
	la a1, _data
	la a2, _edata
	bgeu a1, a2, 2f
1:
	lw t0, (a0)
	sw t0, (a1)
	addi a0, a0, 4
	addi a1, a1, 4
	bltu a1, a2, 1b
2:
	/* Clear bss section */
	la a0, __bss_start
	la a1, _end
	bgeu a0, a1, 2f
1:
	sw zero, (a0)
	addi a0, a0, 4
	bltu a0, a1, 1b
2:
	/*enable mcycle_minstret*/
    csrci CSR_MCOUNTINHIBIT, 0x5
	/* Call global constructors */
	la a0, __libc_fini_array
	call atexit
	call __libc_init_array


	/* argc = argv = 0 */
	li a0, 0
	li a1, 0
	call main
	tail exit

1:
	j 1b
	
	.global disable_mcycle_minstret
disable_mcycle_minstret:
        csrsi CSR_MCOUNTINHIBIT, 0x5
	ret

	.global enable_mcycle_minstret
enable_mcycle_minstret:
        csrci CSR_MCOUNTINHIBIT, 0x5
	ret


.macro unhandled x
unhandledException\x:
        .global unhandledException\x
	la a0, \x
        jal unhandledException
.endm

    unhandled 0
    unhandled 1
    unhandled 2
    unhandled 3
    unhandled 4
    unhandled 5
    unhandled 6
    unhandled 7
    unhandled 8
    unhandled 9
    unhandled 10
    unhandled 11
    unhandled 12
    unhandled 13
    unhandled 14
    unhandled 15
    unhandled 16
    unhandled 17
    unhandled 18
    unhandled 19
    unhandled 20
    unhandled 21
    unhandled 22
    unhandled 23
    unhandled 24
    unhandled 25
    unhandled 26
    unhandled 27
    unhandled 28
    unhandled 29
    unhandled 30
    unhandled 31
    unhandled 32
    unhandled 33
    unhandled 34
    unhandled 35
    unhandled 36
    unhandled 37
    unhandled 38
    unhandled 39
    unhandled 40
    unhandled 41
    unhandled 42
    unhandled 43
    unhandled 44
    unhandled 45
    unhandled 46
    unhandled 47
    unhandled 48
    unhandled 49
    unhandled 50
    unhandled 51
    unhandled 52
    unhandled 53
    unhandled 54
    unhandled 55
    unhandled 56
    unhandled 57
    unhandled 58
    unhandled 59  
    unhandled 60
    unhandled 61
    unhandled 62
    unhandled 63
    unhandled 64
    unhandled 65
    unhandled 66
    unhandled 67
    unhandled 68
    unhandled 69  
    unhandled 70
    unhandled 71
    unhandled 72
    unhandled 73
    unhandled 74
    unhandled 75
    unhandled 76
    unhandled 77
    unhandled 78
    unhandled 79  
    unhandled 80
    unhandled 81
    unhandled 82
    unhandled 83
    unhandled 84
    unhandled 85
    unhandled 86

