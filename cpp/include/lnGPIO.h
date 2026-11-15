/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */
#pragma once
// #include "esprit.h"
#include "esprit_macro.h"
// #include "lnTimer_priv.h"
#include "stdint.h"

/**
 */
#include "lnGPIO_pins.h"
/**
 */
enum lnGpioMode
{
    lnFLOATING = 0,
    lnINPUT_FLOATING = 1,
    lnINPUT_PULLUP = 2,
    lnINPUT_PULLDOWN = 3,
    lnOUTPUT = 4,
    lnOUTPUT_OPEN_DRAIN = 5,
    lnALTERNATE_PP,
    lnALTERNATE_OD,
    lnPWM,
    lnADC_MODE,
    lnDAC_MODE,
    lnUART,
    lnSPI_MODE,
    lnUART_Alt,
};
#define GpioMode lnGpioMode
// typedef int lnPin;
#ifdef LN_RUST_BLUEPILL
// Rust build of the bluepill family: the GPIO driver is `rs_gpio_bluepill`, and
// `esprit/mcus/common_bluepill/lnGPIO.cpp` is not compiled (see drop_C.md, P5), so the
// driver's `#[unsafe(no_mangle)]` exports need C linkage here. Every other build
// (RP2040/RP2350, ESP32, the non-Rust esprit demos) keeps the C++ declarations and the
// C++ definitions unchanged.
extern "C"
{
#endif
    void lnPinMode(const lnPin pin, const lnGpioMode mode, const uint32_t speedInMhz = 0);
    void lnDigitalWrite(const lnPin pin, bool value);
    bool lnDigitalRead(const lnPin pin);
    void lnDigitalToggle(const lnPin pin);
    void lnOpenDrainClose(const lnPin pin, const bool close); // if true, the open drain is passing, else it is hiz

    volatile uint32_t *lnGetGpioToggleRegister(uint32_t port); // Bop register for port "port" with port A:0, B:1, ...
    volatile uint32_t *lnGetGpioDirectionRegister(
        uint32_t port); // Direction register for the bit 0..7 of port "port" , A=0, B=1, ...
    volatile uint32_t *lnGetGpioValueRegister(uint32_t port); // Bit value for LOW bits of port "port"
    uint32_t lnReadPort(uint32_t port);
#ifdef LN_RUST_BLUEPILL
    // Input status register of `port` (the C++ driver reached it through its private
    // gpio[] table). Used by the inline lnFastIO constructor below.
    volatile uint32_t *lnGetGpioStatusRegister(uint32_t port);
#endif

    enum LnRemapTimer
    {
        NoRemap = 0,
        PartialRemap = 1,
        FullRemap = 3,
    };

    void lnRemapTimerPin(uint32_t timer, LnRemapTimer map);
#ifdef LN_RUST_BLUEPILL
}
#endif

#include "lnFastGpio.h"

// #include "lnExti.h"
//  EOF
