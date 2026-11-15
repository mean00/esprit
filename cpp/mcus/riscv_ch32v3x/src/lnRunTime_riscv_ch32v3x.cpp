#include "esprit.h"

extern void lnExtiJtagNoResetOnly();
extern void lnSystemTimerInit();
extern void lnInitFlash_ch32v3x();
/**
 */
void lnRunTimeInit()
{
}
/**
 */
void lnRunTimeInitPostPeripherals()
{
    lnSystemTimerInit();
    // We use fake SWD, no jtag stuff lnExtiJtagNoResetOnly();
    lnInitFlash_ch32v3x(); // fast mode for flash
}
// EOF
