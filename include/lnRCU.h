/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */

#pragma once
#include "esprit.h"
#include "lnPeripherals.h"

enum lnADC_DIVIDER
{
    lnADC_CLOCK_DIV_BY_2 = 0,
    lnADC_CLOCK_DIV_BY_4 = 1,
    lnADC_CLOCK_DIV_BY_6 = 2,
    lnADC_CLOCK_DIV_BY_8 = 3,
    // lnADC_CLOCK_DIV_BY_2=4,
    lnADC_CLOCK_DIV_BY_12 = 5,
    // lnADC_CLOCK_DIV_BY_8=6,
    lnADC_CLOCK_DIV_BY_16 = 7,
};

class lnPeripherals
{
  public:
  public:
    static void reset(const Peripherals periph);
    static void enable(const Peripherals periph);
    static void disable(const Peripherals periph);
    static uint32_t getClock(const Peripherals periph);
    static void setAdcDivider(lnADC_DIVIDER divider);
    static void enableUsb48Mhz(); // Call this once so that the usb clock is 48 Mhz, you still have to enable/disable
                                  // the usbd clock
    static void enableUsbHS48Mhz_ch32v3x(); //  same as above for the USB HS on CH32V3x chips
                                            //
    static void initEthClock();
};
