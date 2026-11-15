/**
 * Copyright (c) 2020 Raspberry Pi (Trading) Ltd.
 *
 * SPDX-License-Identifier: BSD-3-Clause
 */

#include <stdio.h>
#include <stdlib.h>

#include "hardware/clocks.h"
#include "hardware/pio.h"
#include "esprit.h"
#include "lnWS2812_rp_single_pio.h"
#include "pico/stdlib.h"
#include "ws2812.pio.h"
/**
 * @brief Construct a new ws2812 rp2040 pio single::ws2812 rp2040 pio single object
 *
 * @param pio_instance
 * @param pin
 */
WS2812_rp2040_pio_single::WS2812_rp2040_pio_single(int pio_instance, int sm, lnPin pin)
{
    _sm = sm;
    switch (pio_instance)
    {
    case 0:
        _pio = pio0;
        break;
    case 1:
        _pio = pio1;
        break;
    default:
        xAssert(0);
    }

    uint offset = pio_add_program(_pio, &ws2812_program);
    ws2812_program_init(_pio, _sm, offset, pin, 800000, true);
}
/**
 * @brief Destroy the ws2812 rp2040 pio single::ws2812 rp2040 pio single object
 *
 */
WS2812_rp2040_pio_single::~WS2812_rp2040_pio_single()
{
}
/**
 * @brief
 *
 * @param r
 * @param g
 * @param b
 */
void WS2812_rp2040_pio_single::setColor(uint8_t r, uint8_t g, uint8_t b)
{
    uint32_t rgb = ((uint32_t)(r) << 16) | ((uint32_t)(g) << 24) | (uint32_t)(b << 8);
    pio_sm_put_blocking(_pio, _sm, rgb);
}

// EOF
