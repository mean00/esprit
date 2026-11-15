/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 *
 *
 */

#pragma once
#include "esprit.h"

/**
 *
 * @param instance
 */
class lnSimpleADC 
{
  public:
    lnSimpleADC(int instance, lnPin pin);
    virtual ~lnSimpleADC();
    lnPin pin();
    bool setPin(lnPin pin);
    uint32_t simpleRead(uint32_t averaging);                                                 // do a basic sampling
  protected:
    void setup();
    void shutdown();
    lnPin _pin;
};
// EOF

