/*
 *  (C) 2022/2023 MEAN00 fixounet@free.fr
 *  See license file
 *  PFIC is configured with 2 nested level, 1 bit for preemption
 *  That means interrupt priority between 0..7 ignoring preemption
 *
 */
#include "ch32v30x_isr_helper.h"
#include "esprit.h"
#include "lnIRQ.h"
#include "lnIRQ_riscv_priv_ch32v3x.h"
#include "lnRCU.h"

#ifdef USE_CH32v3x_HW_IRQ_STACK
#define HANDLER_DESC(x)                                                                                                \
    extern "C" void x();                                                                                               \
    extern "C" void x##_relay() LN_INTERRUPT_TYPE;
#define HANDLER_DESC_C(y)                                                                                              \
    extern "C" void y();                                                                                               \
    extern "C" void y##_relay() LN_INTERRUPT_TYPE;
extern "C" void unsupported_relay() __attribute__((used));
#define LOCAL_LN_INTERRUPT_TYPE __attribute__((used))
#define WCH_HW_STACK CH32_SYSCR_HWSTKEN
#else
#define LOCAL_LN_INTERRUPT_TYPE LN_INTERRUPT_TYPE __attribute__((used))
#define HANDLER_DESC(x) extern "C" void x() LOCAL_LN_INTERRUPT_TYPE;
#define HANDLER_DESC_C(y) extern "C" void y() LOCAL_LN_INTERRUPT_TYPE;
#define WCH_HW_STACK 0
#endif
#define HANDLER_DESC_RAW(y) extern "C" void y() LOCAL_LN_INTERRUPT_TYPE;

#include "ch32v3x_interrupt_table.h"
/**
 *
 */

LIST_OF_HANDLERS

/**
 * @brief
 *
 */
extern "C" void __attribute__((noinline)) __attribute__((used)) unsupported()
{
    deadEnd(11);
}
/**
 * @brief
 *
 */
void LOCAL_LN_INTERRUPT_TYPE HardFault()
{
    deadEnd(12);
}
/**
 * @brief
 *
 */
void LOCAL_LN_INTERRUPT_TYPE HardFault_relay()
{
    deadEnd(12);
}

#define unsupported_no_decl(y)                                                                                         \
    static void unsupported_##y()                                                                                      \
    {                                                                                                                  \
        deadEnd(y);                                                                                                    \
    }
#define unsupported_no(y) unsupported_##y

unsupported_no_decl(1) unsupported_no_decl(2) unsupported_no_decl(3) unsupported_no_decl(4) unsupported_no_decl(5)
    unsupported_no_decl(6) unsupported_no_decl(7) unsupported_no_decl(8)

//---------------------------------------------------------------------
//---------------------------------------------------------------------
/*- Create vector table -*/
#undef INTERRUPT_DESC
//--
#define INTERRUPT_DESC_RAW(y) (uint32_t)y
#ifdef USE_CH32v3x_HW_IRQ_STACK
#define INTERRUPT_DESC(y) (uint32_t)y##_relay
#else
#define INTERRUPT_DESC(y) (uint32_t)y
#endif
//--
#define UNSUPPORTED_NO(y) (uint32_t)unsupported_##y

#define VECTOR_TABLE __attribute__((section(".vector_table")))
    //--
    extern VECTOR_TABLE const uint32_t vecTable[] __attribute__((aligned(32)));
VECTOR_TABLE const uint32_t vecTable[] __attribute__((aligned(32))) = {LIST_OF_INTERRUPTS};
//--
#define SIZE_OF_VEC_TABLE sizeof(vecTable) / sizeof(uint32_t)
extern const uint32_t size_of_vec_table = SIZE_OF_VEC_TABLE;

uint8_t vec_revert_table[SIZE_OF_VEC_TABLE];

#undef INTERRUPT_DESC

#define WEAK_INTERRUPT(y)                                                                                              \
    extern "C" void __attribute__((used)) __attribute__((weak)) y()                                                    \
    {                                                                                                                  \
        xAssert(0);                                                                                                    \
    }

WEAK_INTERRUPT(USB_WAKEUP_IRQHandler)
WEAK_INTERRUPT(USB_TX_IRQHandler)
WEAK_INTERRUPT(USB_RX_IRQHandler)
WEAK_INTERRUPT(USBHS_IRQHandler)
WEAK_INTERRUPT(USART1_IRQHandler)
WEAK_INTERRUPT(USART2_IRQHandler)
WEAK_INTERRUPT(OTG_FS_IRQHandler)
WEAK_INTERRUPT(ETH_IRQHandler)
WEAK_INTERRUPT(ETH_WKUP_IRQHandler)

#define RELAY_FUNC(x)                                                                                                  \
    ISR_CODE extern "C" void __attribute__((naked)) __attribute((used)) x##_relay()                                    \
    {                                                                                                                  \
        __asm__("jal " #x "\n"                                                                                         \
                "mret");                                                                                               \
    }
#define RELAY_DMA(d, c) RELAY_FUNC(DMA##d##_Channel##c##_IRQHandler)

//---- Relay func
#ifdef USE_CH32v3x_HW_IRQ_STACK
RELAY_FUNC(USART0_IRQHandler)
RELAY_FUNC(USART1_IRQHandler)
RELAY_FUNC(USART2_IRQHandler)
RELAY_FUNC(I2C1_EV_IRQHandler)
RELAY_FUNC(I2C1_ERR_IRQHandler)
RELAY_FUNC(I2C0_EV_IRQHandler)
RELAY_FUNC(I2C0_ERR_IRQHandler)
RELAY_FUNC(Break_Point_Handler)
RELAY_FUNC(SysTick_Handler)
RELAY_FUNC(OTG_FS_IRQHandler)
RELAY_FUNC(USBHS_IRQHandler)
RELAY_FUNC(ETH_IRQHandler)
RELAY_FUNC(ETH_WKUP_IRQHandler)
RELAY_FUNC(unsupported)
RELAY_DMA(0, 0)
RELAY_DMA(0, 1)
RELAY_DMA(0, 2)
RELAY_DMA(0, 3)
RELAY_DMA(0, 4)
RELAY_DMA(0, 5)
RELAY_DMA(0, 6)
RELAY_DMA(1, 0)
RELAY_DMA(1, 1)
RELAY_DMA(1, 2)
RELAY_DMA(1, 3)
RELAY_DMA(1, 4)
RELAY_DMA(1, 5)
RELAY_DMA(1, 6)

#endif
// EOF
