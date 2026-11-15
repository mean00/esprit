/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */
#include "lnADC.h"
#include "lnADC_priv.h"
#include "esprit.h"
#include "lnBasicTimer.h"
#include "lnDma.h"
#include "lnPeripheral_priv.h"
#include "lnPinMapping.h"
/**
 */
#define NB_VCC_AVERAGE 8
/**
 *
 * @param instance
 * @param pin
 */
lnSimpleADC::lnSimpleADC(int instance, lnPin pin) : lnBaseAdc(instance)
{
    _pin = pin;
    setPin(pin);
    setup();
}
/**
 *
 */
lnSimpleADC::~lnSimpleADC()
{
}
/**
 *
 * @param pin
 * @return
 */

/**
 *
 */
void lnSimpleADC::setup()
{
    LN_ADC_Registers *adc = lnAdcDesc[_instance].registers;
    lnBaseAdc::setup();
    //
    adc->CTL1 &= ~LN_ADC_CTL1_ADCON;
    adc->RSQS[2] = adcChannel(_pin);
    adc->RSQS[0] = 0 << 20; // one channel
    adc->CTL1 |= LN_ADC_CTL1_ADCON;
}
/**
 *
 * @return
 */
lnPin lnSimpleADC::pin()
{
    return _pin;
}
/**
 *
 * @param pin
 * @return
 */
bool lnSimpleADC::setPin(lnPin pin)
{
    _pin = pin;
    setup();
    return true;
}
/**
 *
 * @return
 */
int lnSimpleADC::simpleRead()
{
    uint16_t value;
    if (!pollingMultiRead(1, &_pin, &value))
        return -1;
    return (int)value;
}
/**
 *
 * @param nbPins
 * @param pins
 * @param output
 * @return
 */
bool lnSimpleADC::pollingMultiRead(int nbPins, lnPin *pins, uint16_t *output)
{
    xAssert(nbPins <= 5);
    LN_ADC_Registers *adc = lnAdcDesc[_instance].registers;
    xAssert(nbPins);
    // adc Off
    // 1 sample => 0
    adc->RSQS[0] = 0;
    for (int i = 0; i < nbPins; i++)
    {
        uint32_t rsq2 = adcChannel(pins[i]);
        adc->RSQS[2] = rsq2;
        adc->CTL1 |= LN_ADC_CTL1_SWRCST;
        while (!((adc->STAT & LN_ADC_STAT_EOC)))
        {
            __asm__("nop");
        }
        int data = adc->RDATA;
        adc->STAT &= ~LN_ADC_STAT_STRC;
        // Retrieve data
        output[i] = data & 0xfff;
    }
    return true;
}

// EOF