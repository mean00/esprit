/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 *
 *
 * In simpleMode the setup is as follows :
 * simpleMode();
 * enable();
 * setValue(xxx);
 * setValue(xxx);
 *
 * InDmaMode, the setup is :
 *
 * startDmaMode(int fq) where fq is the sample frequency. The one you get can be
 *    different !
 * dmaWrite(nb,data,loop);
 * dmaWrite(nb,data,loop);
 * stopDmaMode();
 */

#pragma once
#include "esprit.h"
#include "lnDma.h"
class lnBasicTimer;
/**
 *
 * @param instance
 */
class lnDAC
{
  public:
    lnDAC(int instance);
    virtual ~lnDAC();
    lnPin pin();
    void enable();
    void disable();
    void simpleMode();        // call this if you just want to manually set the dac value
    void setValue(int value); // 12 bits value 00..495

    void startDmaMode(int fq);
    void stopDmaMode();
    int getDmaFrequency();
    void dmaWrite(int nbn, uint16_t *data, bool loop); // if loop is true it is a non blocking call, else it blocks

  public:
    static void dmaDone_(void *t, lnDMA::DmaInterruptType typ); // callback from interrupt
  protected:
    void dmaDone();

  protected:
    void dmaWriteInternal(int nb, uint16_t *data, bool loop);
    int _instance;
    lnDMA _dma;
    lnBasicTimer *_timer;
    lnBinarySemaphore _dmaSem;
};
