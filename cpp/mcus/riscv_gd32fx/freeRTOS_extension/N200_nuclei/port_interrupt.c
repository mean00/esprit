#include "FreeRTOS.h"
#include "task.h"
#include <stdio.h>

#define ECLIC_IRQ_TIMER 7
#define ECLIC_IRQ_SFT 3
#define ECLIC_IRQ_BWEI 17
#define ECLIC_IRQ_PMOVI 18

extern void lnSystemTimerInit();
extern void lnEnableInterruptDirect(int irq);
/**
 *
 */
void vPortSetupTimerInterrupt()
{
    portENTER_CRITICAL();
    lnSystemTimerInit();
    portEXIT_CRITICAL();

    lnEnableInterruptDirect(ECLIC_IRQ_TIMER);
    lnEnableInterruptDirect(ECLIC_IRQ_SFT);
}
