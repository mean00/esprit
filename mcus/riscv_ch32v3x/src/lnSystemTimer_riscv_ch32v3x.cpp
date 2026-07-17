#include "esprit.h"
#include "lnPeripheral_priv.h"
#include "lnSystemTimer_priv.h"

static uint64_t tickPerUs16 = 1;

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
uint32_t lnGetCycle32()
{
    uint32_t low;
    __asm volatile("csrr %0, mcycle\n" : "=r"(low));
    return low;
}
/**
 */
uint64_t lnGetCycle64()
{
    uint32_t high, low, high2;
    __asm volatile(
        "1:\n"
        "csrr %0, mcycleh\n"
        "csrr %1, mcycle\n"
        "csrr %2, mcycleh\n"
        "bne  %0, %2, 1b\n"
        : "=r"(high), "=r"(low), "=r"(high2)
    );
    return ((uint64_t)high << 32) | low;
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
