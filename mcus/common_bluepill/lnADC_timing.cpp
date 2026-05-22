/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */
#include "lnADC.h"
#include "lnADC_priv.h"
#include "esprit.h"
#include "lnDma.h"
#include "lnPeripheral_priv.h"
#include "lnPinMapping.h"
#include "lnTimer.h"

//------------------------------------------------------------------

/**
 *
 * @param instance
 */
lnTimingAdc::lnTimingAdc(int instance)
    : lnBaseAdc(instance),
      _dma(lnDMA::DMA_PERIPH_TO_MEMORY, lnAdcDesc[_instance].dmaEngine, lnAdcDesc[_instance].dmaChannel, 32, 16)
{
    xAssert(instance == 0); // we need DMA so only ADC0
    _timer = -1;
    _channel = -1;
    _fq = -1;
    _adcTimer = NULL;
    _nbPins = -1;
    _asyncCallback = NULL;
    _asyncCtx = NULL;
}
/**
 *
 */
lnTimingAdc::~lnTimingAdc()
{
    if (_adcTimer)
        delete _adcTimer;
    _adcTimer = NULL;
}
/**
 *
 * @param timer
 * @param channel
 * @param fq
 * @return
 */
bool lnTimingAdc::setSource(int timer, int channel, int fq, int nbPins, const lnPin *pins)
{
    LN_ADC_Registers *adc = lnAdcDesc[_instance].registers;
    _fq = fq;
    _nbPins = nbPins;
    _channel = channel;
    _timer = timer;
    int source = -1;
    int timerId = -1, timerChannel = -1;

    lnBaseAdc::setup();

#define SETTIM(c, a, b)                                                                                                \
    case a * 10 + b:                                                                                                   \
        source = LN_ADC_CTL1_ETSRC_SOURCE_##c;                                                                         \
        timerId = a;                                                                                                   \
        timerChannel = b;                                                                                              \
        break;
    switch (_timer * 10 + _channel)
    {
        SETTIM(T0CH0, 0, 0)
        SETTIM(T0CH1, 0, 1)
        SETTIM(T0CH2, 0, 2)
        SETTIM(T1CH1, 1, 1)
        SETTIM(T2TRGO, 2, 0)
        SETTIM(T3CH3, 3, 3)
    default:
        xAssert(0);
        break;
    }
    // set source
    uint32_t src = adc->CTL1 & LN_ADC_CTL1_ETSRC_MASK;
    src |= LN_ADC_CTL1_ETSRC_SET(source);
    adc->CTL1 = src | LN_ADC_CTL1_ETERC;
    //
    if (_adcTimer)
        delete _adcTimer;
    _adcTimer = new lnAdcTimer(timerId, timerChannel);
    _adcTimer->setPwmFrequency(fq);

    // add our channel(s)
    adc->RSQS[0] = ((uint32_t)(nbPins - 1)) << 20;
    xAssert(_nbPins < 6);
    uint32_t rsq2 = 0;
    for (int i = 0; i < _nbPins; i++)
    {
        rsq2 |= (adcChannel(pins[i]) << (5 * i));
    }
    adc->RSQS[2] = rsq2;
    //
    adc->CTL0 |= LN_ADC_CTL0_SM; // scan mode

    // set clock divider as needed
    uint32_t adcClock = lnPeripherals::getClock(pADC0); // ADC1 does not support DMA/Timing
    int divider = adcClock / fq;                        // nb of clock ticks per conversion
    divider = divider >> 8;                             // divide by 256, max conversion cycles +239.5+12.5 ~ 256

    lnADC_DIVIDER clockDivider = lnADC_CLOCK_DIV_BY_2;

#define SET_DIVIDER(x)                                                                                                 \
    if (divider > x)                                                                                                   \
    {                                                                                                                  \
        clockDivider = lnADC_CLOCK_DIV_BY_##x;                                                                         \
        divider /= x;                                                                                                  \
    }

    SET_DIVIDER(16)
    else SET_DIVIDER(8) else SET_DIVIDER(4) lnPeripherals::setAdcDivider(clockDivider);

    uint32_t conversionCycle = 0;
    // now # of cycles
    if (divider > 239)
        conversionCycle = 7;
    else if (divider > 70)
        conversionCycle = 6;
    else if (divider > 28)
        conversionCycle = 3;
    else if (divider > 7)
        conversionCycle = 1;

    uint32_t smtp = 0;
    for (int i = 0; i < _nbPins; i++)
    {
        smtp |= conversionCycle << (3 * i);
    }
    adc->SAMPT[1] = smtp;
    // go !
    adc->CTL1 |= LN_ADC_CTL1_ADCON;
    return true;
}
/**
 *
 * @param t
 */
void lnTimingAdc::dmaDone_(void *t, lnDMA::DmaInterruptType typ)
{
    lnTimingAdc *me = (lnTimingAdc *)t;
    me->dmaDone();
}
/**
 *
 */
void lnTimingAdc::dmaDone()
{
    _dmaSem.give();
}
/**
 *
 * @param fq
 * @param nbSamplePerChannel
 * @param nbPins
 * @param pins
 * @param output
 * @return
 */
bool lnTimingAdc::multiRead(int nbSamplePerChannel, uint16_t *output)
{
    LN_ADC_Registers *adc = lnAdcDesc[_instance].registers;
    xAssert(_fq > 0);
    // Program DMA
    _dma.beginTransfer();
    _dma.attachCallback(lnTimingAdc::dmaDone_, this);
    _dma.doPeripheralToMemoryTransferNoLock(nbSamplePerChannel * _nbPins, (uint16_t *)output, (uint16_t *)&(adc->RDATA),
                                            false);
    // go !
    adc->CTL1 &= ~LN_ADC_CTL1_CTN;
    adc->CTL1 |= LN_ADC_CTL1_DMA;
    _adcTimer->enable();
    _dmaSem.take();
    _adcTimer->disable();
    // cleanup
    adc->CTL1 &= ~LN_ADC_CTL1_DMA;
    adc->CTL1 &= ~LN_ADC_CTL1_CTN;
    _dma.endTransfer();
    return true;
}

/**
 * Async DMA completion trampoline (static).
 */
void lnTimingAdc::dmaDoneAsync_(void *t, lnDMA::DmaInterruptType typ)
{
    lnTimingAdc *me = (lnTimingAdc *)t;
    me->dmaDoneAsync();
}

/**
 * Async DMA completion handler.
 *
 * 1. Captures the callback/ctx
 * 2. Zeros them (prevents re-entrancy)
 * 3. Cleans up hardware (disables timer, clears DMA flags, ends DMA transfer)
 * 4. Fires the user callback (safe to restart another asyncMultiRead from here)
 */
void lnTimingAdc::dmaDoneAsync()
{
    // Capture and zero before cleanup
    auto cb = _asyncCallback;
    auto ctx = _asyncCtx;
    _asyncCallback = NULL;
    _asyncCtx = NULL;

    // Hardware cleanup (same as end of multiRead)
    LN_ADC_Registers *adc = lnAdcDesc[_instance].registers;
    _adcTimer->disable();
    adc->CTL1 &= ~LN_ADC_CTL1_DMA;
    adc->CTL1 &= ~LN_ADC_CTL1_CTN;
    _dma.endTransfer();

    // Fire user callback (HW is clean, cb/ctx are zeroed)
    if (cb)
        cb(ctx);
}

/**
 * Non-blocking DMA read.
 *
 * Starts a timer-triggered DMA transfer and returns immediately.
 * When the transfer completes, `cb(ctx)` is called from the DMA ISR
 * after hardware cleanup.
 */
bool lnTimingAdc::asyncMultiRead(int nbSamplePerChannel, uint16_t *output,
                                 void (*cb)(void *), void *ctx)
{
    LN_ADC_Registers *adc = lnAdcDesc[_instance].registers;
    xAssert(_fq > 0);

    _asyncCallback = cb;
    _asyncCtx = ctx;

    // Program DMA (same setup as multiRead)
    _dma.beginTransfer();
    _dma.attachCallback(lnTimingAdc::dmaDoneAsync_, this);
    _dma.doPeripheralToMemoryTransferNoLock(nbSamplePerChannel * _nbPins, (uint16_t *)output, (uint16_t *)&(adc->RDATA),
                                            false);
    // Start hardware
    adc->CTL1 &= ~LN_ADC_CTL1_CTN;
    adc->CTL1 |= LN_ADC_CTL1_DMA;
    _adcTimer->enable();

    return true; // non-blocking: returns immediately
}

// EOF