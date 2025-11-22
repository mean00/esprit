/*
 *  (C) 2022/2023 MEAN00 fixounet@free.fr
 *  See license file
 *  PFIC is configured with 2 nested level, 1 bit for preemption
 *  That means interrupt priority between 0..7 ignoring preemption
 *
 *
 */

#ifdef USE_CH32v3x_HW_IRQ_STACK
#define HANDLER_DESC(x)                                                                                                \
    extern "C" void x();                                                                                               \
    extern "C" void x##_relay() LN_INTERRUPT_TYPE;
#define HANDLER_DESC_C(y)                                                                                              \
    extern "C" void y();                                                                                               \
    extern "C" void y##_relay() LN_INTERRUPT_TYPE;
extern "C" void unsupported_relay();
#define LOCAL_LN_INTERRUPT_TYPE __attribute__((used))
#define WCH_HW_STACK CH32_SYSCR_HWSTKEN
#else
#define LOCAL_LN_INTERRUPT_TYPE LN_INTERRUPT_TYPE __attribute__((used))
#define HANDLER_DESC(x) extern "C" void x() LOCAL_LN_INTERRUPT_TYPE;
#define HANDLER_DESC_C(y) extern "C" void y() LOCAL_LN_INTERRUPT_TYPE;
#define WCH_HW_STACK 0
#endif

#include "ch32v30x_isr_helper.h"
#include "ch32v3x_interrupt_table.h"
#include "esprit.h"
#include "lnIRQ.h"
#include "lnIRQ_riscv_priv_ch32v3x.h"
#include "lnRCU.h"

/**
 *
 */
static void PromoteIrqToFast(const LnIRQ &irq, int no);
void _enableDisable_direct(bool enableDisable, const int &irq_num);
extern "C" void Logger_crash(const char *st);
#define FAST_UNUSED 0xFF00

extern const uint32_t vecTable[] __attribute__((aligned(32)));
extern uint8_t vec_revert_table[];
uint16_t fastInterrupt[4] = {FAST_UNUSED, FAST_UNUSED, FAST_UNUSED, FAST_UNUSED};
CH32V3_INTERRUPT *pfic = (CH32V3_INTERRUPT *)LN_PFIC_ADR;

// Attribute [LEVEL1:LEVEL0][SHV] :
//      LEVEL:
//              0:0=LEVEL,
//              0:1=RISING EDGE,
//              1:0: FALLINGEDGE
//      SHV:
//              0 non vectored
//              1 vectored
//

#ifdef LN_ENABLE_I2C
void i2cIrqHandler(int instance, bool error);
#else
#define i2cIrqHandler(...) deadEnd(1)
#endif

#if !defined(LN_ENABLE_ETH)
ISR_CODE extern "C" void LOCAL_LN_INTERRUPT_TYPE ETH_IRQHandler(void)
{
    xAssert(0);
}
ISR_CODE extern "C" void LOCAL_LN_INTERRUPT_TYPE ETH_WKUP_IRQHandler(void)
{
    xAssert(0);
}
#endif
//--
ISR_CODE extern "C" void LOCAL_LN_INTERRUPT_TYPE I2C0_EV_IRQHandler(void)
{
    i2cIrqHandler(0, false);
}
/**
 * @brief
 *
 */
ISR_CODE extern "C" void LOCAL_LN_INTERRUPT_TYPE I2C0_ERR_IRQHandler(void)
{
    i2cIrqHandler(0, true);
}
/**
 * @brief
 *
 */
ISR_CODE extern "C" void LOCAL_LN_INTERRUPT_TYPE I2C1_EV_IRQHandler(void)
{
    i2cIrqHandler(1, false);
}
/**
 * @brief
 *
 */
ISR_CODE extern "C" void LOCAL_LN_INTERRUPT_TYPE I2C1_ERR_IRQHandler(void)
{
    i2cIrqHandler(1, true);
}

//---------------------------------------------------------------------
//---------------------------------------------------------------------

// API used by the freeRTOS port

/**
 * @brief
 *
 */
extern "C" void NVIC_trigger_sw_irq()
{
    const uint32_t sw_irq = 14;
    pfic->IPSR[sw_irq >> 5] = 1 << (sw_irq & 0x1F);
}
/**
 * @brief
 *
 */
extern "C" void NVIC_EnableIRQ(IRQn_Type IRQn)
{
    _enableDisable_direct(true, IRQn);
}

/**
 */
ISR_CODE static int lookupIrq(int irq)
{
    if (_irqs[irq].interrpt == irq)
        return _irqs[irq].irqNb;
    // deep search
    int n = sizeof(_irqs) / sizeof(_irqDesc);
    for (int i = 0; i < n; i++)
    {
        if (_irqs[i].interrpt == irq)
            return _irqs[i].irqNb;
    }
    return 0;
}

/**
 * @brief
 *
 */
ISR_CODE void lnIrqSysInit()
{
    // Disable fast
    for (int i = 0; i < 4; i++)
    {
        pfic->VTFIDR[i] = 0;
        pfic->VTFADDR[i] = 0;
        fastInterrupt[i] = FAST_UNUSED;
    }
// Set all priorities to DEFAULT_PRIO (assuming 2 nested levels)
#define DEFAULT_PRIO 5
    uint32_t prio32 =
        (DEFAULT_PRIO << 4) | (DEFAULT_PRIO << (4 + 8)) | (DEFAULT_PRIO << (4 + 16)) | (DEFAULT_PRIO << (4 + 24));
    for (int i = 0; i < 64; i++)
    {
        pfic->IPRIOIR[i] = prio32;
    }
    //
    extern const uint32_t size_of_vec_table;
    ;
    // Prepare invert able
    for (int i = 0; i < size_of_vec_table; i++)
    {
        int inverted = lookupIrq(i);
        xAssert(inverted < 256);
        vec_revert_table[i] = inverted;
    }

    // allow fast path for these 2 interrupts
    PromoteIrqToFast(LN_IRQ_SYSTICK, 1);
    PromoteIrqToFast(LN_IRQ_SW, 2);
    // Set these two to higher number = lower priority
    lnIrqSetPriority(LN_IRQ_SYSTICK, 6);
    lnIrqSetPriority(LN_IRQ_SW, 7);

    // relocate vector table
    // Initialise WCH enhance interrutp controller,
    // WCH code puts 0x6088 in mstatus
    uint32_t syscr = WCH_HW_STACK | CH32_SYSCR_INESTEN | CH32_SYSCR_MPTCFG_2NESTED | CH32_SYSCR_HWSTKOVEN;
    // uint32_t syscr = WCH_HW_STACK | CH32_SYSCR_HWSTKOVEN;
    uint32_t mstatus = LN_RISCV_FPU_MODE(ARCH_FPU) + LN_RISCV_MPP(0); // enable FPU if ARCH_FPU=1

    asm volatile("mv t0, %1\n"      // load syscr
                 "csrw 0x804, t0\n" // INTSYSCR : hw stack etc...

                 "mv t0, %2\n"
                 "csrw mstatus, t0\n" // Enable floating point and disable interrupts

                 "mv t0, %0 \n"
                 "ori t0, t0, 3\n"   //      Use vectored mode + relocate vector table
                 "csrw mtvec, t0 \n" //

                 ::"r"(vecTable),
                 "r"(syscr), "r"(mstatus));
}
/**

*/
ISR_CODE bool xPortIsInsideInterrupt()
{
    uint32_t gisr = pfic->GISR;
    return (gisr >> 8) & 1; // under interrupt
}

/**
 */
ISR_CODE void _enableDisable_direct(bool enableDisable, const int &irq_num)
{
    if (enableDisable)
    {
        pfic->IENR[irq_num >> 5] = 1 << (irq_num & 0x1f); // 32 bits per register
    }
    else
    {
        pfic->IRER[irq_num >> 5] = 1 << (irq_num & 0x1f);
    }
    // fence ?
}
/**


*/
ISR_CODE void _enableDisable(bool enableDisable, const LnIRQ &irq)
{
    int irq_num = vec_revert_table[irq]; //_irqs[irq].irqNb;
    xAssert(irq_num);
    _enableDisable_direct(enableDisable, irq_num);
}
/**
 * @brief
 *
 * @param irq
 * @param no
 */
ISR_CODE void PromoteIrqToFast(const LnIRQ &irq, int no)
{
    if (no < 1 || no > 4)
    {
        xAssert(0);
    }
    no--;                                // between 0 and 3 now
    int irq_num = vec_revert_table[irq]; //_irqs[irq].irqNb;
    xAssert(irq_num);
    uint32_t adr = vecTable[irq_num];
    fastInterrupt[no] = irq;
    pfic->VTFIDR[no] = irq_num;
    pfic->VTFADDR[no] = adr | 1; // fast path enabled by default, bit0
}
/**
 * @brief
 *
 * @param irq
 */
ISR_CODE void lnEnableInterrupt(const LnIRQ &irq)
{
    _enableDisable(true, irq);
}
/**
 * @brief
 *
 * @param irq_num
 * @param prio
 */
ISR_CODE void lnIrqSetPriority_direct(const int &irq_num, int prio)
{
    int s = (irq_num & 3) * 8;
    int r = irq_num >> 2;
    uint32_t b = pfic->IPRIOIR[r];
    b &= ~(0xff << s);
    b |= (prio << 4) << s;
    pfic->IPRIOIR[r] = b;
}
/**
 * @brief
 *
 * @param irq
 * @param prio
 */
ISR_CODE void lnIrqSetPriority(const LnIRQ &irq, int prio)
{
    int irq_num = vec_revert_table[irq]; //_irqs[irq].irqNb;
    xAssert(irq_num);
    lnIrqSetPriority_direct(irq_num, prio);
}
/**
 * @brief
 *
 * @param irq
 */
ISR_CODE void lnDisableInterrupt(const LnIRQ &irq)
{
    _enableDisable(false, irq);
}

void dmaIrqHandler(int dma, int channel);
#define DMA_IRQ(d, c)                                                                                                  \
    ISR_CODE extern "C" void LOCAL_LN_INTERRUPT_TYPE DMA##d##_Channel##c##_IRQHandler(void)                            \
    {                                                                                                                  \
        dmaIrqHandler(d, c);                                                                                           \
    }

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
/**
 * @brief
 *
 */

/**
 * @brief
 *
 */
extern "C" void __attribute__((noinline)) deadEnd(int code)
{
    // No interrrupt
    __asm__("ebreak");
    ENTER_CRITICAL();
    Logger_crash("**** CRASH *****");
    Logger_crash("**** CRASH *****");
    Logger_crash("**** CRASH *****");
    __asm__("ebreak");
    while (1)
    {
        // blink red light...
        asm volatile("nop");
    }
}

/**
 * @brief
 *
 */
extern "C" void LN_INTERRUPT_TYPE __attribute__((used)) Break_Point_Handler(void)
{
    deadEnd(5);
}

/**
 * @brief
 *
 */
void lnSoftSystemReset()
{
    volatile uint32_t *pfic_sctlr = (volatile uint32_t *)(LN_PFIC_ADR + 0xd10);
    *pfic_sctlr = (1 << 31);
    while (1)
    {
        __asm__("nop");
    }
}

// EOF
