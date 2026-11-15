/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 *
 *
 */

#pragma once
#include "esprit.h"
#include "lnDma.h"

enum lnADC_CYCLES
{
    LN_ADC_SMPT_1_5 = 0,
    LN_ADC_SMPT_7_5 = 1,
    LN_ADC_SMPT_13_5 = 2,
    LN_ADC_SMPT_28_5 = 3,
    LN_ADC_SMPT_41_5 = 4,
    LN_ADC_SMPT_55_5 = 5,
    LN_ADC_SMPT_71_5 = 6,
    LN_ADC_SMPT_239_5 = 7,
};

class lnBasicTimer;
/**
 */
class lnBaseAdc
{
  public:
    lnBaseAdc(int instance)
    {
        _instance = instance;
    }
    ~lnBaseAdc()
    {
    }
    int getVref();         // direct value
    static float getVcc(); // Vcc value in mv
    void setSmpt(lnADC_CYCLES cycles);

  protected:
    void readVcc();
    virtual void setup();
    static int adcChannel(lnPin pin);
    int _instance;
};

/**
 *
 * @param instance
 */
class lnSimpleADC : public lnBaseAdc
{
  public:
    lnSimpleADC(int instance, lnPin pin);
    virtual ~lnSimpleADC();
    lnPin pin();
    bool setPin(lnPin pin);
    int simpleRead();                                                 // do a basic sampling
    bool pollingMultiRead(int nbPins, lnPin *pins, uint16_t *output); // read N pins in one go POLLING

    int getVref();         // direct value
    static float getVcc(); // Vcc value in mv
  protected:
    void readVcc();
    void setup();
    lnPin _pin;
};
/**
 *
 * @param instance
 */
class lnAdcTimer;
class lnTimingAdc : public lnBaseAdc
{
  public:
    lnTimingAdc(int instance);
    virtual ~lnTimingAdc();
    bool setSource(int timer, int channel, int fq, int nbPins, const lnPin *pins);
    bool multiRead(int nbSamplePerChannel, uint16_t *output); // read N pins in one go POLLING
    bool asyncMultiRead(int nbSamplePerChannel, uint16_t *output,
                        void (*cb)(void *), void *ctx); // non-blocking DMA read
  public:
    static void dmaDone_(void *foo, lnDMA::DmaInterruptType typ);
    static void dmaDoneAsync_(void *foo, lnDMA::DmaInterruptType typ);

  protected:
    void dmaDone();
    void dmaDoneAsync();
    int _timer, _channel, _fq;
    lnDMA _dma;
    lnBinarySemaphore _dmaSem;
    lnAdcTimer *_adcTimer;
    int _nbPins;
    void (*_asyncCallback)(void *);
    void *_asyncCtx;
};
// EOF