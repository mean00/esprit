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

LN_ADC_Registers *aadc0 = (LN_ADC_Registers *)LN_ADC0_ADR;
LN_ADC_Registers *aadc1 = (LN_ADC_Registers *)LN_ADC1_ADR;

static int adc_vcc = 0;
static float adc_volt = 0.;

// Timer 5 and 6 are a good fit
const LN_ADC_DESCR lnAdcDesc[2] = {
    {0, 0, (LN_ADC_Registers *)LN_ADC0_ADR}, {-1, -1, (LN_ADC_Registers *)LN_ADC1_ADR}, // ADC1 cannot do DMA !
};

extern const LN_PIN_MAPPING pinMappings[];
/**
 */
int lnBaseAdc::adcChannel(lnPin pin)
{
    const LN_PIN_MAPPING *p = pinMappings;
    while (1)
    {
        xAssert(p->pin != -1);
        if (p->pin == pin)
            return p->adc;
        p++;
    }
}
/**
 *
 */
void lnBaseAdc::readVcc()
{
    LN_ADC_Registers *adc = lnAdcDesc[_instance].registers;
    adc->CTL1 &= ~LN_ADC_CTL1_ADCON;
    if (_instance)
        xAssert(0); // only ADC0 has channel 16 & 17

    adc->CTL1 |= LN_ADC_CTL1_TSVREN;
    adc->RSQS[2] = 17; // VREF
    adc->RSQS[0] = 0;  // VREF
    adc->SAMPT[1] = LN_ADC_SAMPT_239_5;

    adc->CTL1 |= LN_ADC_CTL1_ADCON;
    lnDelayUs(10);
    //
    adc_vcc = 0;
    adc_volt = 0.;
    for (int i = 0; i < NB_VCC_AVERAGE; i++)
    {
        adc->CTL1 |= LN_ADC_CTL1_SWRCST;
        while (!((adc->STAT & LN_ADC_STAT_EOC)))
        {
            __asm__("nop");
        }
        //
        adc_vcc += adc->RDATA & 0xfff;
    }
    if (!adc_vcc)
        adc_vcc = 1;
    // compute voltage
    adc_volt = 1200. * 4095.;
    adc_volt *= (float)NB_VCC_AVERAGE;
    adc_volt /= (float)adc_vcc;

    //
    adc_vcc /= NB_VCC_AVERAGE;
    //
    adc->CTL1 &= ~LN_ADC_CTL1_ADCON;
    adc->CTL1 &= ~LN_ADC_CTL1_TSVREN;
    // adc->CTL1 |=LN_ADC_CTL1_ADCON;
}

/**
 *
 */
void lnBaseAdc::setup()
{
    // Setup default value
    LN_ADC_Registers *adc = lnAdcDesc[_instance].registers;

    adc->CTL1 = 0;
    adc->CTL0 = 0;
    // sw mode by default
    adc->CTL1 |= LN_ADC_CTL1_ETSRC_SET(LN_ADC_CTL1_ETSRC_SOURCE_SW);
    adc->CTL1 |= LN_ADC_CTL1_ETERC;
    ;

    adc->SAMPT[0] = 0;
    adc->SAMPT[1] = LN_ADC_SAMPT_239_5;

    for (int i = 0; i < 4; i++)
        adc->IOFF[i] = 0;

    adc->WDHT = 0;
    adc->WDLT = 0;

    for (int i = 0; i < 3; i++)
        adc->RSQS[i] = 0;

    adc->RSQS[2] = 1; // put a random pin

    adc->ISQ = 0;
    adc->OVRS = 0;

    // Set default clock divider
    // no we dont
    // lnPeripherals::setAdcDivider(lnADC_CLOCK_DIV_BY_8);

    adc->CTL1 |= LN_ADC_CTL1_ADCON;
    //  calibration
    lnDelayUs(10);
    // reset calibration
    adc->CTL1 |= LN_ADC_CTL1_RSTCLB;
    while (adc->CTL1 & LN_ADC_CTL1_RSTCLB)
    {
        __asm__("nop");
    }
    adc->CTL1 |= LN_ADC_CTL1_CLB;
    while (adc->CTL1 & LN_ADC_CTL1_CLB)
    {
        __asm__("nop");
    }

    // read vcc
    if (!_instance)
    {
        readVcc();
    }
}
/**
 *
 * @return
 */
float lnBaseAdc::getVcc()
{
    xAssert(adc_volt);
    return adc_volt;
}

/**
 *
 * @return
 */
int lnBaseAdc::getVref()
{
    return adc_vcc;
}
/**
 *
 * @param cycles
 */
void lnBaseAdc::setSmpt(lnADC_CYCLES cycles)
{
    LN_ADC_Registers *adc = lnAdcDesc[_instance].registers;
    uint32_t smpt = adc->SAMPT[1];
    smpt &= ~7;
    smpt |= cycles & 7;
    adc->SAMPT[1] = smpt;
}

// EOF
