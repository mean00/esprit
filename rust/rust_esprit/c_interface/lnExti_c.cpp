#include "lnExti.h"
#include "lnGPIO.h"

extern "C"
{
#include "lnExti_c.h"
}

void lnExtiAttachInterrupt_c(const lnPin pin, const lnEdge edge, lnExtiCallback *cb, void *cookie)
{
    lnExtiAttachInterrupt(pin, edge, cb, cookie);
}

void lnExtiDetachInterrupt_c(const lnPin pin)
{
    lnExtiDetachInterrupt(pin);
}

void lnExtiEnableInterrupt_c(const lnPin pin)
{
    lnExtiEnableInterrupt(pin);
}

void lnExtiDisableInterrupt_c(const lnPin pin)
{
    lnExtiDisableInterrupt(pin);
}
