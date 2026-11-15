/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */
#pragma once
#include "stdint.h"
/**
 *
 * @param p
 */
class lnFastIO
{
  public:
    lnFastIO(lnPin p);
    void on() volatile
    {
        *_on = _bit;
    }
    void off() volatile
    {
        *_off = _bit;
    }
    void pulseLow() volatile __attribute__((always_inline))
    {
        *_off = _bit;
        *_on = _bit;
    }
    void pulseHigh() volatile __attribute__((always_inline))
    {
        *_off = _bit;
        *_on = _bit;
    }

  protected:
    volatile uint32_t *_on;
    volatile uint32_t *_off;
    volatile uint32_t _bit;
};
// EOF
