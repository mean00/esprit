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

#ifdef LN_RUST_BLUEPILL
// The clock/reset actions, exported by rs_rcu_bluepill (see the class below).
extern "C"
{
    void lnPeripherals_reset(int periph);
    void lnPeripherals_enable(int periph);
    void lnPeripherals_disable(int periph);
}
#endif

class lnPeripherals
{
  public:
  public:
#ifdef LN_RUST_BLUEPILL
    // Rust build of the bluepill family (drop_C.md, P6): mcus/common_bluepill/lnRCU.cpp
    // is not compiled - rs_rcu_bluepill owns the APB/AHB tables - so the three actions
    // that lived there forward to the crate's C entries (rs_rcu_bluepill/src/c_api.rs).
    // The rest of this class is implemented by the family files
    // (lnRCU_armgd32.cpp / lnRCU_ch32v3x_usb.cpp / lnRCU_systemClock_*), which call the
    // very same inline forwarders.
    static void reset(const Peripherals periph) { lnPeripherals_reset((int)periph); }
    static void enable(const Peripherals periph) { lnPeripherals_enable((int)periph); }
    static void disable(const Peripherals periph) { lnPeripherals_disable((int)periph); }
    static uint32_t getClock(const Peripherals periph);
    static void setAdcDivider(lnADC_DIVIDER divider);
    static void enableUsb48Mhz();
    static void enableUsbHS48Mhz_ch32v3x();
    static void initEthClock();
#else
    static void reset(const Peripherals periph);
    static void enable(const Peripherals periph);
    static void disable(const Peripherals periph);
    static uint32_t getClock(const Peripherals periph); // Call this once so that the usb clock is 48 Mhz, you still have to enable/disable
    static void setAdcDivider(lnADC_DIVIDER divider);
    static void enableUsb48Mhz(); // Call this once so that the usb clock is 48 Mhz, you still have to enable/disable
                                  // the usbd clock
    static void enableUsbHS48Mhz_ch32v3x(); //  same as above for the USB HS on CH32V3x chips
                                            //
    static void initEthClock();
#endif
};
