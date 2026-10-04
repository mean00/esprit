/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */

#include "esprit.h"
#include "lnIRQ.h"
#include "lnIRQ_arm.h"
#include "lnIRQ_arm_priv.h"
#include "lnNVIC_arm_priv.h"
#include "lnRCU.h"
#include "lnSCB_arm_priv.h"

#pragma clang diagnostic ignored "-Wextra"
/**
 *
 */

#define AIRCR_KEY (0x5FA << 16)

#define LN_NB_INTERRUPT 68
#define LN_VECTOR_OFFSET 16

LN_SCB_Registers *aSCB LN_USED = (LN_SCB_Registers *)0xE000ED00;

uint32_t *armCurrentInterrupt LN_USED = (uint32_t *)0xE000ED04;
uint32_t *armFaultStatusRegister LN_USED = (uint32_t *)0xE000ED28;

static uint32_t interruptVector[LN_NB_INTERRUPT] __attribute__((aligned(256)));

extern "C" void xPortPendSVHandler();
extern "C" void xPortSysTickHandler();
extern "C" void vPortSVCHandler();
extern "C" void __exc_reset();

static volatile uint32_t curInterrupt;
static volatile LnIRQ curLnInterrupt;

volatile LN_NVIC *anvic = (LN_NVIC *)0xE000E100;

#define LN_MSP_SIZE_UINT32 128
static uint32_t mspStack[LN_MSP_SIZE_UINT32] __attribute__((aligned(8))); // 128*4=512 bytes for msp

lnInterruptHandler *adcIrqHandler = NULL;
/**
 * \fn unsupportedInterrupt
 */

static void unsupportedInterrupt() LN_INTERRUPT_TYPE;
static void unsupportedInterrupt2() LN_INTERRUPT_TYPE;
void unsupportedInterrupt()
{
    curInterrupt = aSCB->ICSR & 0xff;
    curLnInterrupt = (LnIRQ)(curInterrupt - LN_VECTOR_OFFSET);
    __asm__("bkpt 1");
    xAssert(0);
}
void unsupportedInterrupt2()
{
    __asm__("bkpt 1");
    curInterrupt = aSCB->ICSR & 0xff;
    curLnInterrupt = (LnIRQ)(curInterrupt - LN_VECTOR_OFFSET);

    xAssert(0);
}
/**
 *  \brief SoftReset, only resets the core
 * */
void lnSoftSystemReset()
{
    lnScratchRegister = aSCB->AIRCR;
    uint32_t aircr = AIRCR_KEY + (1 << 0); // VECTRESET
    aSCB->VTOR = 0;                        // boot to bootloader, not the app
    aSCB->AIRCR = aircr;
    while (1)
    {
    }
}

/**
 * \brief Full reset, reset the full MCU
 */
void lnHardSystemReset()
{
    lnScratchRegister = aSCB->AIRCR;
    uint32_t aircr = AIRCR_KEY + (1 << 2); // SYSREQRESET
    aSCB->VTOR = 0;                        // boot to bootloader, not the app
    aSCB->AIRCR = aircr;
    while (1)
    {
    }
}
/**
 *
 * @param irq
 * @param prio
 */
void lnIrqSetPriority(const LnIRQ &irq, int prio)
{
    // Interrupt set should be between configLIBRARY_LOWEST_INTERRUPT_PRIORITY<<4 and
    // configLIBRARY_MAX_SYSCALL_INTERRUPT_PRIORITY<<4 Low interrupt means more urgent
    prio = configLIBRARY_LOWEST_INTERRUPT_PRIORITY - prio;
    if (prio > configLIBRARY_LOWEST_INTERRUPT_PRIORITY)
        prio = configLIBRARY_LOWEST_INTERRUPT_PRIORITY;
    if (prio < configLIBRARY_MAX_SYSCALL_INTERRUPT_PRIORITY)
        prio = configLIBRARY_MAX_SYSCALL_INTERRUPT_PRIORITY;

    if (irq < LN_IRQ_WWDG) // Non IRQ
    {
        return;
    }
    anvic->IP[irq] = prio << (8 - configPRIO_BITS);
}

/**
 *
 * @param enableDisable
 * @param zirq
 */
void _enableDisable(bool enableDisable, const LnIRQ &zirq)
{
    int irq = (int)zirq;

    xAssert(irq > 0);

    int offset = irq / 32;
    int bit = irq & 31;
    if (enableDisable)
    {
        anvic->ISER.data[offset] = 1 << bit;
    }
    else
    {
        anvic->ICER.data[offset] = 1 << bit;
    }
}
/**
 *
 * @param per
 */
void lnEnableInterrupt(const LnIRQ &irq)
{
    _enableDisable(true, irq);
}
/**
 *
 * @param irq
 * @param handler
 */
void lnSetInterruptHandler(const LnIRQ &irq, lnInterruptHandler *handler)
{
    switch (irq)
    {
    case LN_IRQ_ADC0_1:
        adcIrqHandler = handler;
        break;
    default:
        xAssert(0);
        break;
    }
}

/**
 *
 * @param per
 */
void lnDisableInterrupt(const LnIRQ &irq)
{
    _enableDisable(false, irq);
}
//
// Forward declaration of interrupt handlers
//

extern "C" void USART0_IRQHandler();
extern "C" void USART1_IRQHandler();
extern "C" void USART2_IRQHandler();

#define DMA_IRQ(d, c)                                                                                                  \
    extern "C" void DMA##d##_Channel##c##_IRQHandler(void) LN_INTERRUPT_TYPE;                                          \
    void DMA##d##_Channel##c##_IRQHandler(void)                                                                        \
    {                                                                                                                  \
        dmaIrqHandler(d, c);                                                                                           \
    }
/**
 *
 * @param dma
 * @param channel
 */
void dmaIrqHandler(int dma, int channel);
DMA_IRQ(0, 0)
DMA_IRQ(0, 1)
DMA_IRQ(0, 2)
DMA_IRQ(0, 3)
DMA_IRQ(0, 4)
DMA_IRQ(0, 5)
DMA_IRQ(0, 6)
DMA_IRQ(1, 0)
DMA_IRQ(1, 1)
DMA_IRQ(1, 2)
DMA_IRQ(1, 3)
DMA_IRQ(1, 4)
DMA_IRQ(1, 5)
DMA_IRQ(1, 6)
//--
extern "C" void TIMER6_IRQHandler();
extern "C" void TIMER5_IRQHandler();
extern "C" void TIMER1_IRQHandler();
extern "C" void TIMER2_IRQHandler();
extern "C" void TIMER3_IRQHandler();

extern void USB_TX_IRQHandler();
extern void USB_RX_IRQHandler();
extern void USB_WAKEUP_IRQHandler();

/**
 *
 */
void ADC01_IRQHandler(void) LN_INTERRUPT_TYPE;
void ADC01_IRQHandler(void)
{
    if (!adcIrqHandler)
        xAssert(0);
    adcIrqHandler();
}

/**
 * I2C
 * @param code
 */
#ifdef LN_ENABLE_I2C
extern "C" void i2cIrqHandler(int instance, bool error);
#else
#define i2cIrqHandler(...) deadEnd(1)
#endif

#define I2C_IRQ(d)                                                                                                     \
    extern "C"                                                                                                         \
    {                                                                                                                  \
        void I2C##d##_EV_IRQHandler(void) LN_INTERRUPT_TYPE;                                                           \
        void I2C##d##_EV_IRQHandler(void)                                                                              \
        {                                                                                                              \
            i2cIrqHandler(d, false);                                                                                   \
        }                                                                                                              \
    }                                                                                                                  \
    extern "C"                                                                                                         \
    {                                                                                                                  \
        void I2C##d##_ERR_IRQHandler(void) LN_INTERRUPT_TYPE;                                                          \
        void I2C##d##_ERR_IRQHandler(void)                                                                             \
        {                                                                                                              \
            i2cIrqHandler(d, true);                                                                                    \
        }                                                                                                              \
    }

I2C_IRQ(0)
I2C_IRQ(1)

extern "C"
{
    extern void EXTI0_IRQHandler();
    extern void EXTI1_IRQHandler();
    extern void EXTI2_IRQHandler();
    extern void EXTI3_IRQHandler();
    extern void EXTI4_IRQHandler();
    extern void EXTI5_9_IRQHandler();
    extern void EXTI10_15_IRQHandler();
}
/**
 *
 * @param code
 */
extern "C" void Logger_crash(const char *st);
extern "C" void deadEnd(int code)
{
    static int lastErrorCode;
    __asm__("cpsid if"); // disable interrupt
    lastErrorCode = code;
    Logger_crash("**** CRASH ***\n");
    Logger_crash("**** CRASH ***\n");
    Logger_crash("**** CRASH ***\n");
    __asm__("bkpt 1");
    while (1)
    {
        // blink red light...
        __asm__("nop");
        __asm__("nop"); // do something to avoid deadlock
    }
}
// https://developer.arm.com/documentation/dui0552/a/the-cortex-m3-processor/exception-model/exception-entry-and-return
// https://interrupt.memfault.com/blog/arm-cortex-m-exceptions-and-nvic

struct registerStack
{
    uint32_t R0, R1, R2, R3, R12, LR, PC, xPSR;
};
volatile registerStack *crashStructure;
/**
 *
 * @param sp
 */
extern "C" void crashHandler2(void *sp) LN_INTERRUPT_TYPE;
/**
 *
 * @param sp
 */
void crashHandler2(void *sp)
{
    crashStructure = (registerStack *)sp;
    __asm__("cpsid if    \n");
    deadEnd(0x90);
}

/**
 *
 * @param code
 */
void crashHandler(int code) LN_USED __attribute__((naked));

// https://www.freertos.org/Debugging-Hard-Faults-On-Cortex-M-Microcontrollers.html
#define IRQ_STUBS(name, code)                                                                                          \
    extern "C" void name() LN_USED __attribute__((naked));                                                             \
    extern "C" void name()                                                                                             \
    {                                                                                                                  \
        __asm__(" .balign 4 \n"                                                                                        \
                " mrs r1, ipsr \n"                                                                                     \
                " tst lr, #4                                                \n"                                        \
                " ite eq                                                    \n"                                        \
                " mrseq r0, msp                                             \n"                                        \
                " mrsne r0, psp                                             \n"                                        \
                " bkpt 1                                                     \n"                                       \
                " ldr r2, handler2_address_const" #name "                     \n"                                      \
                " bx r2                                                     \n"                                        \
                " handler2_address_const" #name ": .word crashHandler2    \n" ::);                                     \
    }

extern "C" void UsageFault_IrqHandler() LN_USED __attribute__((naked));
extern "C" void UsageFault_IrqHandler()
{
    __asm__(" .balign 4 \n"
            " bkpt 1                                                    \n"
            " mrs r1, ipsr \n"
            " tst lr, #4                                                \n"
            " ite eq                                                    \n"
            " mrseq r0, msp                                             \n"
            " mrsne r0, psp                                             \n"
            " ldr r2, handler2_address_constUsageFault_IrqHandler       \n"
            " bx r2                                                     \n"
            " handler2_address_constUsageFault_IrqHandler: .word crashHandler2    \n" ::);
}

/**
 *
 * @param code
 */

IRQ_STUBS(RTC_IrqHandler, 0x21);
IRQ_STUBS(WDG_IrqHandler, 0x22);
IRQ_STUBS(Tamper_IrqHandler, 0x23);
IRQ_STUBS(RCU_IrqHandler, 0x24);
IRQ_STUBS(FMC_IrqHandler, 0x25);
IRQ_STUBS(HardFault_IrqHandler, 0x26);
IRQ_STUBS(NMI_IrqHandler, 0x27);
IRQ_STUBS(MemManage_IrqHandler, 0x28);
IRQ_STUBS(BusFault_IrqHandler, 0x29);
IRQ_STUBS(DebugMon_IrqHandler, 0x2b);
// IRQ_STUBS(UsageFault_IrqHandler, 0x30);

#define unsupported unsupportedInterrupt
#define unsupported2 unsupportedInterrupt2
#include "lnIRQ_arm_vector.h"

/**
 * \fn lnIrqSysInit
 * https://www.freertos.org/RTOS-Cortex-M3-M4.html
 */
void lnIrqSysInit()
{
    // by default disable everything
    anvic->ICER.data[0] = 0xffffffffUL;
    anvic->ICER.data[1] = 0xffffffffUL;
    anvic->ICER.data[2] = 0xffffffffUL;
    anvic->ICER.data[3] = 0xffffffffUL;

    // Set AIRCR to 3 (i.e. 4 bits), see https://interrupt.memfault.com/blog/arm-cortex-m-exceptions-and-nvic
    uint32_t aircr = aSCB->AIRCR & 7;
    aircr |= AIRCR_KEY + (3 << 8);
    aSCB->AIRCR = aircr;

    // Set priority to 14 for all interrupts
    for (int i = LN_IRQ_WWDG; i < LN_IRQ_ARM_LAST; i++)
        lnIrqSetPriority((LnIRQ)i, 0); // by default lesss urgent

    // set exception priority, SVC and friends will be set by FreeRTOS
    uint8_t exceptionPriority = configMAX_SYSCALL_INTERRUPT_PRIORITY;
    for (int i = 0; i < 4; i++)
    {
        aSCB->SHP[i] = exceptionPriority; // Usage fault, page fault,mem manage
    }
    // 5..8 SVC priority => dont change, it is used only once anyway
    // 9..11 => done by FreeRTOS

    // Relocate vector to there
    aSCB->VTOR = (uint32_t)LnVectorTable;
    aSCB->CR |= 0 +
                // (1<<3)+ // unaligned access
                (1 << 4) + // div by zero
                (1 << 9);  // stack aligned to 8 bytes

    aSCB->SHCSR |= (7 << 16); // enable fault, div by zero,...
    __asm__ __volatile__("dsb sy");
    return;
}
void __attribute__((weak)) USB_WAKEUP_IRQHandler()
{
    xAssert(0);
}
void __attribute__((weak)) USB_TX_IRQHandler()
{
    xAssert(0);
}
void __attribute__((weak)) USB_RX_IRQHandler()
{
    xAssert(0);
}
// EOF
