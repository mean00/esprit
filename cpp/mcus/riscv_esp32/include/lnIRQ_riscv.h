/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */
#pragma once
#include "esprit.h"
#include "lnPeripherals.h"

#define LN_INTERRUPT_TYPE __attribute__((used)) __attribute__((interrupt("IRQ")));

enum LnIRQ : int
{
    LN_IRQ_NONE = 0,
};
#define underInterrupt (xPortInIsrContext())

