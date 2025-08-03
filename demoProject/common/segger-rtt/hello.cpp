#include "esprit.h"
//
extern "C"
{
#include "SEGGER_RTT.h"
}
#define LED LN_SYSTEM_LED
void setup()
{
    lnPinMode(LED, lnOUTPUT_OPEN_DRAIN);
}
extern void rttLoggerFunction(int n, const char *data);
/**
 *
 */
int roundup = 0;
void loop()
{
    bool onoff = true;
    lnDigitalWrite(LED, true);
    setLogger(rttLoggerFunction); // REDIRECT LOGGING TO RTT
    SEGGER_RTT_Init();
    while (1)
    {
        roundup++;
        xDelay(500);
        lnDigitalWrite(LED, onoff);
        onoff = !onoff;
        Logger(">Hello::%d*\n", roundup);
    }
}
