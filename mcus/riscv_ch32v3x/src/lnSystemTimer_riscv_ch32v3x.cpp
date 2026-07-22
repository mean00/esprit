#include "esprit.h"
#include "lnPeripheral_priv.h"
#include "lnSystemTimer_priv.h"

#include "lnAssert.h"

static uint64_t tickPerUs16 = 1;

extern "C" uint32_t lnGetMs_c();
volatile SysTick_Type *WCH_SysTick = (volatile SysTick_Type *)0xE000F000;

/**
 *  This should be called with interrupt disabled
 */
void lnSystemTimerInit()
{
    // number of ticks for a duration of 1 us
    tickPerUs16 = ((SystemCoreClock * 4)) / (1000 * 1000); // *16/4
}

/**
 */
uint64_t lnGetCycle64()
{
    // HARDWARE CHECK: Assert that FreeRTOS has enabled the SysTick timer
    xAssert(WCH_SysTick->CTLR & 1);

    uint32_t ticks;
    uint32_t count;
    uint32_t ticks2;
    uint32_t is_pending;

    do
    {
        ticks = lnGetMs_c();
        count = (uint32_t)WCH_SysTick->CNT; // Only need lower 32 bits since CMP is <= 32-bit
        is_pending = WCH_SysTick->SR & 1;   // Bit 0 is the CNTIF (Count Flag)
        ticks2 = lnGetMs_c();
    } while (ticks != ticks2);

    // If an interrupt is pending, SysTick->CNT has already wrapped to 0,
    // but the FreeRTOS xTickCount hasn't been incremented yet.
    // We only compensate if 'count' is small (wrapped) to avoid a race
    // where the flag is set just BEFORE we read 'count'.
    if (is_pending && (count < (WCH_SysTick->CMP / 2)))
    {
        ticks++;
    }

    // Total cycles = (completed ticks * cycles per tick) + current cycles
    uint32_t multiplier = (uint32_t)WCH_SysTick->CMP;
    return ((uint64_t)ticks * multiplier) + count;
}

/**
 */
uint32_t lnGetCycle32()
{
    return (uint32_t)lnGetCycle64();
}

/**
 *
 * @return
 */
uint64_t lnGetUs64()
{
    uint64_t tick = lnGetCycle64();
    // convert tick to us
    tick = (tick * 16) / tickPerUs16;
    return tick;
}
/**
 */
uint32_t lnGetUs()
{
    return (uint32_t)lnGetUs64();
}
// EOF
