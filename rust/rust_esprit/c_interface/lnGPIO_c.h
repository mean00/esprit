#pragma once
#include "lnGPIO.h"

#ifdef __cplusplus
extern "C"
{
#endif

    volatile uint32_t *lnGetGpioOnRegister(uint32_t port);  // Bop register for port "port" with port A:0, B:1, ...
    volatile uint32_t *lnGetGpioOffRegister(uint32_t port); //
    void lnPinMode_c(const lnPin pin, const lnGpioMode mode, const uint32_t speedInMhz);

#ifdef __cplusplus
}
#endif