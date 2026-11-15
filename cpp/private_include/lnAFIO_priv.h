/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */

#pragma once
#include "esprit.h"

struct LN_AFIOx
{
    uint32_t EC;
    uint32_t PCF0;
    uint32_t EXTISS[4];
    uint32_t dummy;
    uint32_t PCF1;
};

typedef volatile LN_AFIOx LN_AFIO;

#define LN_AFIO_PCF0_SWJ_SET(x) (x << 24)
#define LN_AFIO_PCF0_SWJ_MASK (~(7 << 24))

#define LN_AFIO_PCF0_REMAP_TIMER(t, x) (x << (t * 2 + 6))
#define LN_AFIO_PCF0_REMAP_TIMER_MASK(t, x) (~(3 << (t * 2 + 6)))

#define LN_AFIO_PCF0_REMAP_SPI0(x) ((x << 0))
#define LN_AFIO_PCF0_REMAP_SPI0_MASK (~(1 << 0))
