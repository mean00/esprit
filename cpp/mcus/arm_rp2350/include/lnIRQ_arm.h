/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */
#pragma once
#include "esprit.h"
#include "lnPeripherals.h"

#define LN_IRQ_OFFSET 0
#define LN_INTERRUPT_TYPE __attribute__((used)) __attribute__((interrupt("IRQ")))
#define underInterrupt xPortIsInsideInterrupt()

enum LnIRQ : int
{
    // These are exception, not really IRQS
    // but it"s simpler to manage the vector table
    LN_IRQ_RESET = LN_IRQ_OFFSET - 15,
    LN_IRQ_NMI = LN_IRQ_OFFSET - 14,
    LN_IRQ_HARDFAULT = LN_IRQ_OFFSET - 13,
    LN_IRQ_MEMORYMANAGEMENT = LN_IRQ_OFFSET - 12,
    LN_IRQ_BUSFAULT = LN_IRQ_OFFSET - 11,
    LN_IRQ_USAGEFAULT = LN_IRQ_OFFSET - 10,
    LN_IRQ_SECUREFAULT = LN_IRQ_OFFSET - 9,
    LN_IRQ_RESERVED = LN_IRQ_OFFSET - 6,
    LN_IRQ_SVCALL = LN_IRQ_OFFSET - 5,
    LN_IRQ_DEBUGMON = LN_IRQ_OFFSET - 4,
    LN_IRQ_RESERVED2 = LN_IRQ_OFFSET - 3,
    LN_IRQ_PENDSV = LN_IRQ_OFFSET - 2,
    LN_IRQ_SYSTICK = LN_IRQ_OFFSET - 1,

    // The real IRQs start here

    LN_IRQ_TIMER0_0 = LN_IRQ_OFFSET + 0, // 0
    LN_IRQ_TIMER0_1 = LN_IRQ_OFFSET + 1, // 0
    LN_IRQ_TIMER0_2 = LN_IRQ_OFFSET + 2, // 0
    LN_IRQ_TIMER0_3 = LN_IRQ_OFFSET + 3, // 0
    LN_IRQ_TIMER1_0 = LN_IRQ_OFFSET + 4, // 0
    LN_IRQ_TIMER1_1 = LN_IRQ_OFFSET + 5, // 0
    LN_IRQ_TIMER1_2 = LN_IRQ_OFFSET + 6, // 0
    LN_IRQ_TIMER1_3 = LN_IRQ_OFFSET + 7, // 0
                                         //
                                         //
    LN_IRQ_DMA_0 = LN_IRQ_OFFSET + 10,   // 0
    LN_IRQ_DMA_1 = LN_IRQ_OFFSET + 11,   // 0
    LN_IRQ_DMA_2 = LN_IRQ_OFFSET + 12,   // 0
    LN_IRQ_DMA_3 = LN_IRQ_OFFSET + 13,   // 0

    LN_IRQ_USB = LN_IRQ_OFFSET + 14, // 0

    LN_IRQ_SPI0 = LN_IRQ_OFFSET + 31, // 0
    LN_IRQ_SPI1 = LN_IRQ_OFFSET + 32, // 0

    LN_IRQ_UART0 = LN_IRQ_OFFSET + 33, // 0
    LN_IRQ_UART1 = LN_IRQ_OFFSET + 34, // 0

    LN_IRQ_ADC = LN_IRQ_OFFSET + 35, // 0

    LN_IRQ_I2C0 = LN_IRQ_OFFSET + 36, // 0
    LN_IRQ_I2C1 = LN_IRQ_OFFSET + 37, // 0

    LN_IRQ_ARM_LAST
};
