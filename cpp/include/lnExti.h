/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */

#pragma once

enum lnPin : int;
enum lnEdge
{
    LN_EDGE_NONE = 0,
    LN_EDGE_RISING = 1,
    LN_EDGE_FALLING = 2,
    LN_EDGE_BOTH = 3,
};

typedef void lnExtiCallback(lnPin pin, void *cookie);

void lnExtiAttachInterrupt(const lnPin pin, const lnEdge edge, lnExtiCallback *cb, void *cookie);
void lnExtiDetachInterrupt(const lnPin pin);
void lnExtiEnableInterrupt(const lnPin pin);
void lnExtiDisableInterrupt(const lnPin pin);

// EOF
