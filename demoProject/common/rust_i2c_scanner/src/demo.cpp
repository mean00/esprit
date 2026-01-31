
/*
    On the RP2040 gum stick boards the led is driven by GPIO26, it is NOT open drain
*/
#include "esprit.h"

#define LED LN_SYSTEM_LED

extern "C" void user_init();
extern void rttLoggerFunction(int n, const char *data);
/**
 */
void setup()
{
    // setLogger(rttLoggerFunction); // REDIRECT LOGGING TO RTT
}
void loop()
{
    Logger("Starting Rust demo\n");
    user_init();
    bool onoff = true;
    while (1)
    {
        lnDelayMs(1000);
    }
}
// EOF
