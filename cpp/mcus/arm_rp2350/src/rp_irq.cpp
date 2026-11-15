
#include "hardware/irq.h"
#include "hardware/regs/intctrl.h"
#include "esprit.h"
#include "ln_rp.h"
#include "stdio.h"

#include "hardware/structs/scb.h"
volatile armv8m_scb_hw_t *scb = ((volatile armv8m_scb_hw_t *)(PPB_BASE + M33_CPUID_OFFSET));
#define NVIC_IPR0_OFFSET M33_NVIC_IPR0_OFFSET

/*
0 msp
1 reset

    SVC => 11
    PEND => 14
    SYSTICK => 15
*/

extern uint32_t ram_vector_table[48]; // this is the vector table in RAM

void lnSetInterruptHandler(const LnIRQ &irq, lnInterruptHandler *handler)
{
    int real_irq = (int)irq + 16;
    ram_vector_table[real_irq] = (uint32_t)handler;
    __asm__("dmb" : : : "memory");
}
/**
 * @brief
 *
 * @param irq
 */
void lnEnableInterrupt(const LnIRQ &irq)
{
    irq_set_enabled(irq, true);
}
/**
 * @brief
 *
 * @param irq
 */
void lnDisableInterrupt(const LnIRQ &irq)
{
    irq_set_enabled(irq, false);
}
/**
 * @brief
 *
 */
extern "C" void dummy_caller(int code)
{
    volatile uint32_t vtor = scb->vtor;
    __asm__("bkpt #0");
}
// EOF
