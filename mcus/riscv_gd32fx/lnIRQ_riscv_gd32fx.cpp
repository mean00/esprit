/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */

#include "esprit.h"
#include "lnIRQ.h"
#include "lnIRQ_riscv_priv_gd32.h"
#include "lnRCU.h"
/**
 *
 */

#define ECLIC_CTLBIT 4                        // We have the 4 higher bits
#define ECLIC_CTLBIT_SHIFT (8 - ECLIC_CTLBIT) // # number of low bits
#define ECLIC_CTLBIT_LOW ((1 << (ECLIC_CTLBIT_SHIFT)) - 1)
#define ECLIC_NB_SOURCES 87

uint32_t underInterrupt = 0;

// Attribute [LEVEL1:LEVEL0][SHV] :
//      LEVEL:
//              0:0=LEVEL,
//              0:1=RISING EDGE,
//              1:0: FALLINGEDGE
//      SHV:
//              0 non vectored
//              1 vectored
//

void lnIrqSysInit()
{
    // start from a clean state
    *eclicCfg = 0;
    *eclicMth = 0;

    // Check the compilation value are the same as the runtime value
    xAssert((*eclicInfo & 0xfff) == ECLIC_NB_SOURCES);
    xAssert(((*eclicInfo >> 21) & 0xf) == ECLIC_CTLBIT);

    // See bumblebee Core Architecture Manual
    *eclicCfg = 4 << 1; // nlbits =4

    for (int i = 0; i < ECLIC_NB_SOURCES; i++)
    {
        eclicIrqs[i].ie = 0; // default : disabled
        eclicIrqs[i].ip = 0x0;
    }

    // Switch to ECLIC mode
    // If the 6 lower bits= 3 => eclic mode
    uint32_t tmp1, tmp2;

    asm volatile(
        "li  %0,0x3f\n" // https://stackoverflow.com/questions/64547741/how-is-risc-v-neg-instruction-imeplemented
        "not %1,%0\n"
        "csrr %0, mtvec\n"
        "and  %0,%0,%1\n"
        "ori  %0,%0,3\n"
        "csrw mtvec, %0\n" ::"r"(tmp1),
        "r"(tmp2));

    // Now reconfigure with real value
    for (int i = 0; i < ECLIC_NB_SOURCES; i++)
    {
        lnSetInterruptLevelDirect(i, 1, false);
    }
    // Preconfigure timer & syscall
    // these 2 are in vector mode, lowest prio
    lnSetInterruptLevelDirect(3, 0, true);
    lnSetInterruptLevelDirect(7, 0, true);
    return;
}

void _enableDisable(bool enableDisable, const LnIRQ &irq)
{
    const _irqDesc *i = _irqs + (int)irq;
    xAssert(i->interrpt == irq)

        LN_ECLIC_irq *iclic = eclicIrqs + i->irqNb;
    if (enableDisable)
        iclic->ie |= 1;
    else
        iclic->ie &= ~1;
    LN_FENCE();
}
/**
 *
 * @param irq
 */
extern "C" void lnEnableInterruptDirect(int irq)
{
    LN_ECLIC_irq *iclic = eclicIrqs + irq;
    iclic->ie |= 1;
    LN_FENCE();
}
/**
 *
 * @param irq
 */
extern "C" void lnDisableInterruptDirect(int irq)
{
    LN_ECLIC_irq *iclic = eclicIrqs + irq;
    iclic->ie &= ~1;
    LN_FENCE();
}

/**
 *
 * @param per
 */
void lnEnableInterrupt(const LnIRQ &irq)
{
    _enableDisable(true, irq);
}
void lnIrqSetPriority(const LnIRQ &irq, int prio)
{
    lnSetInterruptLevelDirect(irq, 1 + prio, false);
}
/**
 *
 * @param irq
 */
void lnSetInterruptLevelDirect(int irq, int prio, bool vectored)
{
    LN_ECLIC_irq *iclic = eclicIrqs + irq;
    bool attr = 0;
    if (vectored)
        attr |= 1;
    // Assume level interrupt (?)
    iclic->attr = attr;
    // control
    //  4 bits then 1 1 1 1
    // nlbits=4 => level = 4 bits, priority=0 bits
    iclic->control = (prio << ECLIC_CTLBIT_SHIFT) + ECLIC_CTLBIT_LOW;
    LN_FENCE();
}
/**
 *
 * @param per
 */
void lnDisableInterrupt(const LnIRQ &irq)
{
    _enableDisable(false, irq);
}

void lnWriteMthDirect(int mth)
{
    *eclicMth = mth;
    LN_FENCE();
}
int lnReadMthDirect()
{
    return *eclicMth;
}

#define DMA_IRQ(d, c)                                                                                                  \
    extern "C" void DMA##d##_Channel##c##_IRQHandler(void)                                                             \
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

extern "C" void deadEnd(int code)
{
    // No interrrupt
    ENTER_CRITICAL();
    while (1)
    {
        // blink red light...
    }
}
// EOF
