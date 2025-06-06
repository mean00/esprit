#include "lnADC.h"
#include "esprit.h"
#include "lnDAC.h"
#include "lnTimer.h"
#include "math.h"
#if defined(USE_RP2040) || defined(USE_RP2350)
#else
#define LED PA2
#define PWM PB9
#endif
#define TIMER 3
#define CHANNEL 3

void setup()
{
    lnPinMode(LN_SYSTEM_LED, lnOUTPUT);
}
/**
 *
 */
#define TIMER_ID 3
#define TIMER_CHANNEL 3
#define FREQUENCY (50 * 1000)
#define TIMER_PIN PB9
#define TYPE uint16_t

int val;
void loop()
{
    bool onoff = true;
    lnDigitalWrite(LED, true);
    int roundup = 0;

    lnPinMode(PA0, lnADC_MODE);
    lnPinMode(PA1, lnADC_MODE);
    lnPinMode(PA4, lnDAC_MODE);
    lnPinMode(PA5, lnDAC_MODE);

    lnTimingAdc *adc = new lnTimingAdc(0);
    lnDAC *dac0 = new lnDAC(0);
    lnDAC *dac1 = new lnDAC(1);

    dac0->simpleMode();
    dac1->simpleMode();

    dac0->enable();
    dac1->enable();

    Logger("Connect PA4 and PA0\n");
    Logger("Connect PA5 and PA1\n");
#define SAMPLE_PER_CHANNEL 10
    TYPE output[2 * SAMPLE_PER_CHANNEL];
    lnPin pins[2] = {PA0, PA1};

    lnPinMode(TIMER_PIN, lnPWM);

#if 0
    {    
    lnAdcTimer pwm(TIMER_ID,TIMER_CHANNEL);
    pwm.setTimerFrequency(1000);
    pwm.enable();
    while(1)
    {
        xDelay(10);
    }
    }
#endif

#if 0
    {
    lnTimer pwm(TIMER_ID,TIMER_CHANNEL);
    pwm.setTimerFrequency(1000);
    pwm.setPwmMode(512);
    pwm.enable();
    while(1)
    {
        xDelay(10);
    }
    }
#endif
    adc->setSource(TIMER_ID, TIMER_CHANNEL, FREQUENCY, 2, pins);
    while (1)
    {
        dac0->setValue(500);
        dac1->setValue(3500);
        lnDelayMs(10);
        memset(output, 0, 2 * SAMPLE_PER_CHANNEL * sizeof(TYPE));

        adc->multiRead(SAMPLE_PER_CHANNEL, (uint16_t *)output);
        Logger("500:3500 PA0: %d PA1 :%d \n", output[0], output[1]);

        for (int i = 0; i < SAMPLE_PER_CHANNEL; i++)
        {
            if (output[0 + 2 * i] > 1000)
            {
                Logger(" PA0 %d : too big : %d \n", i, output[0 + 2 * i]);
            }
            if (output[1 + 2 * i] < 288)
            {
                Logger(" PA1 %d : too small : %d \n", i, output[1 + 2 * i]);
            }
        }
        lnDelayMs(10);
        dac0->setValue(3500);
        dac1->setValue(500);
        lnDelayMs(10);
        memset(output, 0, 2 * SAMPLE_PER_CHANNEL * sizeof(TYPE));
        adc->multiRead(SAMPLE_PER_CHANNEL, (uint16_t *)output);

        Logger("3500:500 PA0: %d PA1 :%d \n", output[0], output[1]);
        for (int i = 0; i < SAMPLE_PER_CHANNEL; i++)
        {
            if (output[1 + 2 * i] > 1000)
            {
                Logger("PA1 %d : too big : %d \n", i, output[1 + 2 * i]);
            }
            if (output[0 + 2 * i] < 288)
            {
                Logger("PA0 %d : too small : %d \n", i, output[0 + 2 * i]);
            }
        }
        lnDelayMs(20);
    }
}
