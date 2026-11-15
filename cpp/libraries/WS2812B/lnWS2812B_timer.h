/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */
#pragma once
#include "lnSPI.h"
#include "lnTimer.h"
#include "lnWS2812B.h"
/**
 *
 * @param nbLeds
 * @param pin
 */
class WS2812B_timer : public WS2812B_base, public lnDmaTimerCallback
{
  public:
    WS2812B_timer(uint32_t nbLeds, lnPin pin);
    virtual ~WS2812B_timer();

    void begin();  // call this first
    void update(); // call this to have the changes committed
    bool timerCallback(bool half);

  protected:
    void convertRgb(int hilow, uint8_t *rgb);
    lnPin _pin;
    lnDmaTimer *_timer;
    uint8_t _timerPwmValue[48];
    int _one, _zero;
    int _nextLed;
    lnBinarySemaphore _sem;
    uint32_t lookup[16];
};
