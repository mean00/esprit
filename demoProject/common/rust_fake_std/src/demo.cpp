
/*
    On the RP2040 gum stick boards the led is driven by GPIO26, it is NOT open drain
*/
#include "esprit.h"

#include "LN_RTT.h"

#define LED LN_SYSTEM_LED

extern void rttLoggerFunction(int n, const char *data);

extern "C" void user_init();
/**
 */
void setup()
{
    setLogger(rttLoggerFunction); // REDIRECT LOGGING TO RTT
    LN_RTT_Init();
    lnPinMode(LED, lnOUTPUT_OPEN_DRAIN);
}
void loop()
{
    Logger("Starting Rust fake_std test suite\n");
    user_init();
    bool onoff = true;
    while (1)
    {
        lnDelayMs(1000);
        lnOpenDrainClose(LED, onoff);
        onoff = !onoff;
    }
}
#ifndef __clang__
extern "C" void _exit()
{
    xAssert(0);
}
#endif
// EOF
