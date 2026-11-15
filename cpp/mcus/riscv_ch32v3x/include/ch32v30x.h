/********************************** (C) COPYRIGHT  *******************************
 * File Name          : ch32v30x.h
 * Author             : WCH
 * Version            : V1.0.0
 * Date               : 2021/06/06
 * Description        : CH32V30x Device Peripheral Access Layer Header File.
 * Copyright (c) 2021 Nanjing Qinheng Microelectronics Co., Ltd.
 * SPDX-License-Identifier: Apache-2.0
 *******************************************************************************/
#ifndef __CH32V30x_H
#define __CH32V30x_H

#ifdef __cplusplus
extern "C"
{
#endif

    /* Interrupt Number Definition, according to the selected device */
    typedef enum IRQn
    {
        /******  RISC-V Processor Exceptions Numbers *******************************************************/
        NonMaskableInt_IRQn = 2, /* 2 Non Maskable Interrupt                             */
        EXC_IRQn = 3,            /* 3 Exception Interrupt                                */
        Ecall_M_Mode_IRQn = 5,   /* 5 Ecall M Mode Interrupt                             */
        Ecall_U_Mode_IRQn = 8,   /* 8 Ecall U Mode Interrupt                             */
        Break_Point_IRQn = 9,    /* 9 Break Point Interrupt                              */
        SysTicK_IRQn = 12,       /* 12 System timer Interrupt                            */
        Software_IRQn = 14,      /* 14 software Interrupt                                */
                                 /* RTC Alarm through EXTI Line Interrupt                */
    } IRQn_Type;

#define __I volatile  /* defines 'read only' permissions */
#define __IO volatile /* defines 'read / write' permissions */

    /* memory mapped structure for SysTick */
    typedef struct
    {
        volatile uint32_t CTLR;
        volatile uint32_t SR;
        volatile uint64_t CNT;
        volatile uint64_t CMP;
    } SysTick_Type;
    extern volatile SysTick_Type *WCH_SysTick;

    extern void NVIC_EnableIRQ(IRQn_Type IRQn);

#ifdef __cplusplus
}
#endif

#endif
