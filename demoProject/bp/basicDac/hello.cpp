#include "esprit.h"
#include "lnDAC.h"
#include "lnTimer.h"
#include "math.h"

#define PWM PB9
#define TIMER 3
#define CHANNEL 3

void setup()
{
    lnPinMode(LN_SYSTEM_LED, lnOUTPUT);
}
/**
 *
 */

int val;
void loop()
{
    bool onoff = true;
    lnDigitalWrite(LN_SYSTEM_LED, true);
    int roundup = 0;

    lnPinMode(PA4, lnDAC_MODE);
    lnPinMode(PA5, lnDAC_MODE);

    lnDAC *dac = new lnDAC(0);

    int signalFrequency = 30 * 1000;

    dac->startDmaMode(signalFrequency * 100);
    int actualFq = dac->getDmaFrequency();

    float pointFloat = (float)(actualFq + signalFrequency / 2) / (float)signalFrequency;
    int nbPoints = floor(pointFloat);
    // Logger("In Fq=%d outFq=%d, # points=%d pointsF=%f\n", signalFrequency, actualFq, nbPoints, pointFloat);
    Logger("In Fq=%d outFq=%d, # points=%d pointsF=\n", signalFrequency, actualFq, nbPoints);

    uint16_t *xsin = new uint16_t[nbPoints];
    for (int i = 0; i < nbPoints; i++)
    {
        float angle = 2.0f * (float)M_PI;
        angle /= (float)nbPoints;
        angle *= (float)i;
        xsin[i] = 2048.f + 2047.f * (float)sin(angle);
    }

    dac->dmaWrite(nbPoints, xsin, true);

    while (1)
    {
        xDelay(10);
    }
#if 0         
    xAssert(0);
    dac->simpleMode();
    dac->enable();

    while(1)
    {
        dac->setValue(val);
        val++;
        val&=(1<<13)-1;
    }
#endif
}
