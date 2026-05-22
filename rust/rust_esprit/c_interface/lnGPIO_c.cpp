#include "lnGPIO.h"

extern "C"
{
#include "lnGPIO_c.h"
}

// lnGetGpioOnRegister / lnGetGpioOffRegister are already defined for RP2040
// in ln_rp_gpio.cpp.  For other platforms we provide them here.
#if !defined(USE_RP2040) && !defined(USE_RP2350)

volatile uint32_t *lnGetGpioOnRegister(uint32_t port)
{
    // On GD32/bluepill: BOP register, writing bit N sets the pin high
    return lnGetGpioToggleRegister((int)port);
}

volatile uint32_t *lnGetGpioOffRegister(uint32_t port)
{
    // On GD32/bluepill: BOP register, writing bit (16+N) sets the pin low
    return lnGetGpioToggleRegister((int)port);
}

#endif

void lnPinMode_c(const lnPin pin, const lnGpioMode mode, const uint32_t speedInMhz)
{
    lnPinMode(pin, mode, speedInMhz);
}