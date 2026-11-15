#include "esprit.h"

// Get systick
struct LN_SystemTimer_Registersx
{
    uint32_t CSR;   // Control  & status register
    uint32_t RVR;   // Reload value register
    uint32_t CVR;   // Current value register
    uint32_t CALIB; // calibration
};

typedef volatile LN_SystemTimer_Registersx LN_SystemTimer_Registers;

LN_SystemTimer_Registers *aSysTick = (LN_SystemTimer_Registers *)0xE000E010;

/**
 *
 * @return
 */
uint32_t lnGetCycle32()
{
    return 0xffffffff - aSysTick->CVR;
}

/**
 *
 * @return
 */

uint64_t lnGetUs64()
{
    uint64_t before, after, us;

    while (1)
    {
        before = lnGetMs();
        us = aSysTick->CVR;
        after = lnGetMs();
        if (before == after)
            break;
    }
    us = aSysTick->RVR + 1 - us;                // counter goes downward
    uint64_t scale = SystemCoreClock / 1000000; // avoid too much overflow
    uint64_t tick = (us) / (uint64_t)scale;
    tick += before * 1000LL;
    return tick;
}
/**
 *
 * @return
 */
uint32_t lnGetUs()
{
    uint32_t before, after, us;

    while (1)
    {
        before = lnGetMs();
        us = aSysTick->CVR;
        after = lnGetMs();
        if (before == after)
            break;
    }
    us = aSysTick->RVR + 1 - us;                // counter goes downward
    uint32_t scale = SystemCoreClock / 1000000; // avoid too much overflow
    uint32_t tick = (us) / (uint32_t)scale;
    tick += before * 1000;
    return tick;
}
extern "C" uint32_t lnGetUs_C()
{
    return lnGetUs();
}
extern "C" uint32_t lnGetCycle32_C()
{
    return lnGetCycle32();
}