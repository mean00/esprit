#pragma once
#include "lnExti.h"

#ifdef __cplusplus
extern "C"
{
#endif

    void lnExtiAttachInterrupt_c(const lnPin pin, const lnEdge edge, lnExtiCallback *cb, void *cookie);
    void lnExtiDetachInterrupt_c(const lnPin pin);
    void lnExtiEnableInterrupt_c(const lnPin pin);
    void lnExtiDisableInterrupt_c(const lnPin pin);

#ifdef __cplusplus
}
#endif
