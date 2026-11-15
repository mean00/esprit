/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */
#include "lnDAC.h"
#include "esprit.h"
#include "lnBasicTimer.h"
#include "lnDAC_priv.h"
#include "lnDma.h"
#include "lnPeripheral_priv.h"
#include "math.h"
/**
 */
struct LN_DAC_DESCR
{
    lnPin pin;
    int dmaEngine;
    int dmaChannel;
    int basicTimer;
    int source;
};
// Timer 5 and 6 are a good fit
const LN_DAC_DESCR lnDacDesc[2] = {
    {PA4, 1, 2, 0, LN_DAC_CTL_DTSEL_SOURCE_T5},
    {PA5, 1, 3, 1, LN_DAC_CTL_DTSEL_SOURCE_T6},
};

LN_DAC_Registers *adac = (LN_DAC_Registers *)LN_DAC_ADR;

/**
 *
 * @return
 */
lnPin lnDAC::pin()
{
    return lnDacDesc[_instance].pin;
}
/**
 *
 * @param instance
 */
lnDAC::lnDAC(int instance)
    : _dma(lnDMA::DMA_MEMORY_TO_PERIPH, lnDacDesc[instance].dmaEngine, lnDacDesc[instance].dmaChannel, 16, 32)
{
    lnPeripherals::enable((Peripherals)(pDAC + instance));
    _instance = instance;
    disable();
}
/**
 *
 */
lnDAC::~lnDAC()
{
    disable();
}
/**
 *
 */
void lnDAC::enable()
{
    adac->CTL |= LN_DAC_CTL_ENABLE(_instance);
}
/**
 *
 */
void lnDAC::disable()
{
    adac->CTL &= ~LN_DAC_CTL_ENABLE(_instance);
}
/**
 *
 */
void lnDAC::simpleMode()
{
    uint32_t ctl = adac->CTL;
    ctl &= LN_DAC_CTL_DTSEL_MASK(_instance);
    ctl |= LN_DAC_CTL_DTSEL(_instance, LN_DAC_CTL_DTSEL_SOURCE_SW);
    ctl |= LN_DAC_CTL_DTEN(_instance);
    // ctl|=LN_DAC_CTL_DBOFF(_instance);
    adac->CTL = ctl;
}
/**
 *
 */
void lnDAC::startDmaMode(int fq)
{
    _timer = new lnBasicTimer(_instance);
    uint32_t ctl = adac->CTL;
    ctl &= LN_DAC_CTL_DTSEL_MASK(_instance);
    ctl |= LN_DAC_CTL_DTSEL(_instance, lnDacDesc[_instance].source);
    ctl |= LN_DAC_CTL_DTEN(_instance);
    // ctl|=LN_DAC_CTL_DBOFF(_instance);
    ctl |= LN_DAC_CTL_DDMAEN0(_instance);
    adac->CTL = ctl;
    _dma.beginTransfer();
    _timer->setTimerFrequency(fq);
}
/**
 *
 * @return
 */
int lnDAC::getDmaFrequency()
{
    return _timer->getTimerFrequency();
}
/**
 *
 */
void lnDAC::stopDmaMode()
{

    adac->CTL &= ~LN_DAC_CTL_DDMAEN0(_instance);
    _dma.endTransfer();
    delete _timer;
    _timer = NULL;
}

/**
 *
 * @param value
 */
void lnDAC::setValue(int value)
{
    adac->DATAS[_instance].DAC_R12DH = value;
    adac->SWT |= LN_DAC_SWT_ENABLE(_instance);
}
/**
 *
 * @param t
 */
void lnDAC::dmaDone_(void *t, lnDMA::DmaInterruptType typ)
{
    lnDAC *me = (lnDAC *)t;
    me->dmaDone();
}
/**
 *
 */
void lnDAC::dmaDone()
{
    _dmaSem.give();
}
/**
 *
 * @param nb
 * @param data
 * @param loop
 */
void lnDAC::dmaWriteInternal(int nb, uint16_t *data, bool loop)
{

    if (!loop)
    {
        _dma.attachCallback(lnDAC::dmaDone_, this);
    }

    _dma.doMemoryToPeripheralTransferNoLock(nb, (uint16_t *)data, (uint16_t *)&(adac->DATAS[_instance].DAC_R12DH),
                                            false, loop);
    enable();
    _timer->enable();
    if (loop)
        return;
    _dmaSem.take();
}

/**
 *
 */

void lnDAC::dmaWrite(int nbn, uint16_t *data, bool loop)
{
    dmaWriteInternal(nbn, data, loop);
}

// EOF
