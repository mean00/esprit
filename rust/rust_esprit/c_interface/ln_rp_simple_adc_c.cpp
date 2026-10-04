/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 *
 *  RP2040 / RP2350 <-> Rust bridge for the simple (single shot) ADC.
 *
 *  esprit only provides the C++ `lnSimpleADC` driver on RP2xxx: there is no
 *  `lnTimingAdc` equivalent there (that class only exists for the Bluepill
 *  targets, see mcus/common_bluepill/lnADC_timing.cpp), and the `ln_simple_adc_*`
 *  C ABI is owned by the Rust backends.  These `extern "C"` helpers are what
 *  `rs_adc_rp2xxx` - the backend rust_esprit uses on RP2040/RP2350 - calls, so
 *  that `ln_simple_adc_*` behaves the same on RP2xxx as it does on GD32/CH32.
 */
#include "ln_rp_simple_adc_c.h"
#include "lnADC.h"

/**
 * @brief Create a simple ADC on the given instance / pin.
 *
 * `pin` is an `lnPin` value (GPIO26..GPIO29 are the RP2xxx ADC channels); the
 * driver asserts on anything else.  Allocates with the C++ heap, as
 * swindle/src/platform/rp2040/bmp_adc_rp2040.cpp does.
 */
void *ln_rp_simple_adc_create(int instance, uint32_t pin)
{
    return (void *)new lnSimpleADC(instance, (lnPin)pin);
}

/**
 * @brief Destroy an ADC created by ln_rp_simple_adc_create().
 */
void ln_rp_simple_adc_destroy(void *adc)
{
    delete (lnSimpleADC *)adc;
}

/**
 * @brief Do `averaging` conversions and return the mean value.
 */
uint32_t ln_rp_simple_adc_read(void *adc, uint32_t averaging)
{
    if (!adc)
        return 0;
    if (!averaging) // simpleRead() divides by it
        averaging = 1;
    return ((lnSimpleADC *)adc)->simpleRead(averaging);
}
// -- EOF --
