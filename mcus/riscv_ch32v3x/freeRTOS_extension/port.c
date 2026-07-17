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

/**
 * @file    port.c
 * @brief   FreeRTOS port layer implementation for RISC-V RV32 (CH32V3x series).
 * @details Implements the functions declared in portable.h for the RISC-V RV32
 *          port, tailored for WCH CH32V3x MCUs.  Provides the hardware-specific
 *          scheduler start/stop, critical sections, interrupt masking, SysTick
 *          timer setup, task stack initialisation and the SysTick interrupt
 *          handler.
 *
 *          The port uses a hardware-managed (mscratch-based) interrupt stack
 *          when @c USE_CH32v3x_HW_IRQ_STACK is defined; otherwise the standard
 *          @c __attribute__((interrupt)) attribute is applied to the SysTick
 *          handler.
 *
 *          FreeRTOS Kernel V10.4.6
 *          Copyright (C) 2021 Amazon.com, Inc. or its affiliates.  All Rights Reserved.
 *          SPDX-License-Identifier: MIT
 */

/*-----------------------------------------------------------
 * Implementation of functions defined in portable.h for the RISC-V RV32 port.
 *----------------------------------------------------------*/

/* Scheduler includes. */
#include "FreeRTOS.h"
#include "portmacro.h"
#include "task.h"
/* Standard includes. */
#include "port_common.h"
#include "string.h"

/**
 * @defgroup port_internal_macros Internal Macros
 * @{
 */

/**
 * @brief   Select interrupt attribute for SysTick_Handler.
 * @details When @c USE_CH32v3x_HW_IRQ_STACK is defined the handler does *not*
 *          use the compiler's interrupt prologue/epilogue (the hardware stack
 *          switch via @c mscratch handles it).  Otherwise the standard
 *          @c __attribute__((interrupt)) is applied.
 */
#ifdef USE_CH32v3x_HW_IRQ_STACK
#define LN_IRQ_FOS
#else
#define LN_IRQ_FOS __attribute__((interrupt))
#endif

/* SysTick Control Register Bits (WCH CH32V3x specific) */
#define SYSTICK_CTLR_STE (1 << 0)   /**< System counter enable.             */
#define SYSTICK_CTLR_STIE (1 << 1)  /**< Counter interrupt enable.          */
#define SYSTICK_CTLR_STCLK (1 << 2) /**< System clock source (HCLK).        */
#define SYSTICK_CTLR_STRE (1 << 3)  /**< Auto-reload count enable.          */

/** RISC-V mstatus MIE (Machine Interrupt Enable) bit. */
#define RISCV_MIE (1 << 3)
/** RISC-V mstatus MPIE (Machine Previous Interrupt Enable) bit. */
#define RISCV_MPIE (1 << 7)
/**
 * @brief   mstatus MPP field value for Machine mode on the Bumblebee core.
 * @details The WCH Bumblebee core (CH32V3x) encodes Machine mode as MPP = 0b10
 *          (bits [12:11]), unlike the standard RISC-V encoding which uses 0b11.
 *          This value is written into the task's initial stack frame so that
 *          the first @c mret restores the task to Machine privilege level.
 */
#define RISCV_MPP_MACHINE (2 << 11)

/**
 * @brief   Check ISR stack integrity (empty on this port).
 * @details Placeholder macro; no ISR stack overflow detection is implemented.
 */
#define portCHECK_ISR_STACK()                                                                                          \
    {                                                                                                                  \
    }

/** @} */ /* end of port_internal_macros */

/**
 * @defgroup port_isr_stack ISR Stack Configuration
 * @{
 *
 * The stack used by interrupt service routines.  Set
 * @c configISR_STACK_SIZE_WORDS to use a statically allocated array as the
 * interrupt stack.  Alternatively leave @c configISR_STACK_SIZE_WORDS undefined
 * and update the linker script so that the linker variable
 * @c __freertos_irq_stack_top has the same value as the top of the stack used
 * by @c main().  Using the linker script method will repurpose the stack that
 * was used by @c main() before the scheduler was started for use as the
 * interrupt stack after the scheduler has started.
 */
#ifdef configISR_STACK_SIZE_WORDS
#define ISR_ALIGN 16
static __attribute__((aligned(ISR_ALIGN))) StackType_t xISRStack[configISR_STACK_SIZE_WORDS] = {0};
const StackType_t xISRStackTop = (StackType_t) & (xISRStack[configISR_STACK_SIZE_WORDS & ~(ISR_ALIGN - 1)]);

/**
 * @brief   Fill byte used to initialise the ISR stack.
 * @note    Do not use 0xa5 as that is used by the kernel for task stacks, and
 *          so will legitimately appear in many positions within the ISR stack.
 */
#define portISR_STACK_FILL_BYTE 0xee
#else
/* __freertos_irq_stack_top defined by the linker script (.ld file). */
extern const uint32_t __freertos_irq_stack_top[];
const StackType_t xISRStackTop = (StackType_t)__freertos_irq_stack_top;
#endif

/** @} */ /* end of port_isr_stack */

/**
 * @brief   Critical section nesting counter.
 * @details Initialised to 0xaaaaaaaa as a canary value to help detect
 *          corruption before the scheduler starts.
 */
static UBaseType_t uxCriticalNesting = 0xaaaaaaaa;

/**
 * @brief   Forward declaration: set up the timer that generates the tick
 *          interrupts.
 * @details The implementation in this file targets the WCH CH32V3x SysTick
 *          peripheral and is deliberately not weak so that single-source
 *          builds are unambiguous.  Application writers who wish to use a
 *          different timer should replace this function.
 */
void vPortSetupTimerInterrupt(void);

/*-----------------------------------------------------------*/

/**
 * @brief   Configure the WCH CH32V3x SysTick timer for the FreeRTOS tick
 *          interrupt.
 * @details This implementation does not use the RISC-V standard CLINT/MTIME;
 *          instead it programs the chip-specific SysTick peripheral with the
 *          compare value derived from @c configCPU_CLOCK_HZ and
 *          @c configTICK_RATE_HZ, and enables the counter, interrupt and
 *          auto-reload.
 */
void vPortSetupTimerInterrupt(void)
{
    SysTick->CTLR = 0;
    SysTick->SR = 0;
    SysTick->CNT = 0;
    SysTick->CMP = configCPU_CLOCK_HZ / configTICK_RATE_HZ;
    SysTick->CTLR = SYSTICK_CTLR_STE | SYSTICK_CTLR_STIE | SYSTICK_CTLR_STCLK | SYSTICK_CTLR_STRE;
}

/*-----------------------------------------------------------*/

/**
 * @brief   Start the FreeRTOS scheduler.
 * @details Performs the following steps:
 *          -# If @c configASSERT_DEFINED is 1, validates that:
 *             - The @c mtvec register has its low two bits set to @c 0b11
 *               (vectored mode).
 *             - The ISR stack top is aligned to @c portBYTE_ALIGNMENT.
 *             - If @c configISR_STACK_SIZE_WORDS is defined, fills the ISR
 *               stack array with a known canary byte.
 *          -# Calls @c vPortSetupTimerInterrupt() to initialise the SysTick
 *             timer.
 *          -# Enables the SysTick and software-interrupt IRQ lines in the
 *             NVIC/PLIC.
 *          -# Resets the critical nesting count to 0.
 *          -# Calls @c xPortStartFirstTask() (assembly) which restores the
 *             context of the highest-priority task and starts executing it
 *             with interrupts enabled.
 * @return This function never returns on success.  If it does return, it
 *         returns @c pdFAIL to indicate that the scheduler could not be
 *         started.
 */
BaseType_t xPortStartScheduler(void)
{
    extern void xPortStartFirstTask(void);

#if (configASSERT_DEFINED == 1)
    {
        volatile uint32_t mtvec = 0;

        /* Check the least significant two bits of mtvec are 0b11 - indicating
        multiply vector mode. */
        __asm volatile("csrr %0, mtvec" : "=r"(mtvec));
        configASSERT((mtvec & 0x03UL) == 0x3);

        /* Check alignment of the interrupt stack - which is the same as the
        stack that was being used by main() prior to the scheduler being
        started. */
        configASSERT((xISRStackTop & portBYTE_ALIGNMENT_MASK) == 0);

#ifdef configISR_STACK_SIZE_WORDS
        {
            memset((void *)xISRStack, portISR_STACK_FILL_BYTE, sizeof(xISRStack));
        }
#endif /* configISR_STACK_SIZE_WORDS */
    }
#endif /* configASSERT_DEFINED */

    /* If there is a CLINT then it is ok to use the default implementation
    in this file, otherwise vPortSetupTimerInterrupt() must be implemented to
    configure whichever clock is to be used to generate the tick interrupt. */
    vPortSetupTimerInterrupt();

    NVIC_EnableIRQ(SysTicK_IRQn);
    NVIC_EnableIRQ(Software_IRQn);

    /* Initialise the critical nesting count ready for the first task. */
    uxCriticalNesting = 0;
    xPortStartFirstTask();

    /* Should not get here as after calling xPortStartFirstTask() only tasks
    should be executing. */
    return pdFAIL;
}
/*-----------------------------------------------------------*/

/**
 * @brief   Stop the FreeRTOS scheduler.
 * @details This port does not implement scheduler stopping.  If called, the
 *          function enters an infinite loop.
 * @note    The standard FreeRTOS API expects this function to exist, but it
 *          is not normally used in resource-constrained embedded systems.
 */
void vPortEndScheduler(void)
{
    /* Not implemented. */
    for (;;)
        ;
}
/*-----------------------------------------------------------*/
/**
 * @brief   Handle the SysTick interrupt (FreeRTOS tick).
 * @details Switches to the ISR stack via @c ENTER_ISR_STACK(), clears the SysTick
 *          status register, calls @c xTaskIncrementTick() to update the
 *          kernel tick count, and yields if a context switch is required.
 *          The interrupt stack is released via @c EXIT_ISR_STACK() before return.
 * @note    The function is declared with @c LN_IRQ_FOS which selects between
 *          hardware stack switching (@c USE_CH32v3x_HW_IRQ_STACK) and the
 *          compiler's @c __attribute__((interrupt)) prologue/epilogue.
 */
void SysTick_Handler(void) __attribute__((used)) LN_IRQ_FOS;
void SysTick_Handler(void)
{
    ENTER_ISR_STACK();
    portDISABLE_INTERRUPTS();
    SysTick->SR = 0; /* Clear the CNTIF flag (Bumblebee SysTick does not auto-acknowledge). */
    if (xTaskIncrementTick() != pdFALSE)
    {
        portYIELD();
    }
    portENABLE_INTERRUPTS();
    EXIT_ISR_STACK();
}

/*-----------------------------------------------------------*/
/**
 * @brief   Enter a critical section by disabling interrupts.
 * @details Disables interrupts via @c portDISABLE_INTERRUPTS() and
 *          increments the nesting counter @c uxCriticalNesting.
 *          Critical sections may be nested safely.
 */
void vPortEnterCritical(void)
{
    portDISABLE_INTERRUPTS();
    uxCriticalNesting++;
}

/*-----------------------------------------------------------*/
/**
 * @brief   Exit a critical section, re-enabling interrupts when the nesting
 *          count reaches zero.
 * @details Decrements @c uxCriticalNesting.  When the count reaches zero
 *          interrupts are re-enabled via @c portENABLE_INTERRUPTS().
 * @note    Asserts that the nesting count is non-zero before decrementing.
 */
void vPortExitCritical(void)
{
    configASSERT(uxCriticalNesting);
    uxCriticalNesting--;

    if (uxCriticalNesting == 0)
    {
        portENABLE_INTERRUPTS();
    }
}
/*-----------------------------------------------------------*/
/**
 * @brief   Mask all interrupts by clearing the MIE and MPIE bits in @c mstatus.
 * @details Atomically reads the current @c mstatus, clears bits 0x88 (MIE=bit 3
 *          and MPIE=bit 7), and returns the original value so it can be restored
 *          later via @c vPortClearInterruptMask().  Uses the RISC-V @c csrrc
 *          instruction to perform the read-modify-write in a single atomic step.
 * @return The original @c mstatus value before interrupts were masked.
 */
portUBASE_TYPE xPortSetInterruptMask(void)
{
    portUBASE_TYPE uvalue;
    __asm volatile("csrrc %0, mstatus, %1" : "=r"(uvalue) : "r"(0x88));
    return uvalue;
}

/*-----------------------------------------------------------*/
/**
 * @brief   Restore a previously saved interrupt mask.
 * @details Writes the supplied value back to the @c mstatus CSR, restoring
 *          the previous interrupt enable state.
 * @param   uvalue  The @c mstatus value to restore (previously returned by
 *                  @c xPortSetInterruptMask()).
 */
void vPortClearInterruptMask(portUBASE_TYPE uvalue)
{
    __asm volatile("csrw  mstatus, %0" ::"r"(uvalue));
}

/*----*/

/**
 * @brief   Initialise the stack frame of a new task.
 * @details Builds the full context save frame that will be restored by the
 *          assembly context-switch code.  The frame layout is:
 *          @code
 *          (high address)  pxCode   (MEPC)     -- header 0
 *                          mstatus  (MSTATUS)  -- header 1
 *                          (FPU regs)           -- if ARCH_FPU == 1
 *                          x1..x31  (GPRs)     -- 28 words
 *          (low address)  <- SP
 *          @endcode
 *
 *          Only the exception-return address (pxCode) and machine status
 *          (mstatus with MPIE set and MPP=2 for machine mode) are written in
 *          the header area.  Among the GPR slots, only @c x10/a0 is set to
 *          @c pvParameters (the task parameter).  The FPU state is set to
 *          "initial" so that the first context switch lazily saves nothing.
 *
 * @param   pxTopOfStack   The top of the allocated stack (highest address).
 * @param   pxCode         The task entry function.
 * @param   pvParameters   The void* parameter to pass to the task function.
 * @return  A pointer to the new stack top (after the frame has been reserved).
 */
StackType_t *pxPortInitialiseStack(StackType_t *pxTopOfStack, TaskFunction_t pxCode, void *pvParameters)
{
    uint32_t mstatus = 0;

    mstatus |= RISCV_MPIE;        // Interrupt enabled
    mstatus |= RISCV_MPP_MACHINE; // restore in machine mode

#if ARCH_FPU == 1
    mstatus |= CH32_FPU_STATE(CH32_FPU_INITIAL); // Set FS bits to "initial"
#endif
    pxTopOfStack -= (portCONTEXT_COUNT + portHEADER_COUNT);
    StackType_t *newStack = pxTopOfStack;
    pxTopOfStack[0] = (StackType_t)pxCode; // fill in headers
    pxTopOfStack[1] = mstatus;
    pxTopOfStack += portHEADER_COUNT;            // jump to GPR registers
    pxTopOfStack[6] = (StackType_t)pvParameters; // 6 is x10=A0
    return newStack;
}
//--
