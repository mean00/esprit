/**
 * @file lnWS2812_rp_single.cpp
 * @author MEAN00 (you@domain.com)
 * @brief
 * @version 0.1
 * @date 2023-12-27
 *
 * @copyright Copyright (c) 2023
 *
 */
#include "lnWS2812_rp_single_spi.h"
#include "../private_include/ln_rp_spi_priv.h"
#include "esprit.h"
#include "lnGPIO.h"
#include "lnSerial.h"

#define P1 0xCU
#define P0 0x8U

// The RP send data MSB first
// #define XXX(a, b, c, d) (a + (b << 4) + (c << 8) + (d << 12))
// #define XXX(b, a, d, c) (a + (b << 4) + (c << 8) + (d << 12))
#define XXX(d, c, b, a) (a + (b << 4) + (c << 8) + (d << 12))

// expand 4 bits to 16 (*4)

const uint16_t lookupTable[16] = {
    XXX(P0, P0, P0, P0), // 0
    XXX(P0, P0, P0, P1), XXX(P0, P0, P1, P0), XXX(P0, P0, P1, P1),

    XXX(P0, P1, P0, P0), // 4
    XXX(P0, P1, P0, P1), XXX(P0, P1, P1, P0), XXX(P0, P1, P1, P1),

    XXX(P1, P0, P0, P0), // 0
    XXX(P1, P0, P0, P1), XXX(P1, P0, P1, P0), XXX(P1, P0, P1, P1),

    XXX(P1, P1, P0, P0), // 0
    XXX(P1, P1, P0, P1), XXX(P1, P1, P1, P0), XXX(P1, P1, P1, P1),
};

/**
 * @brief
 *
 * @param r
 * @param g
 * @param b
 */
void WS2812_rp2040_single::setColor(uint8_t r, uint8_t g, uint8_t b)
{
    _buffer[0] = lookupTable[g >> 4];
    _buffer[1] = lookupTable[g & 0xf];
    _buffer[2] = lookupTable[r >> 4];
    _buffer[3] = lookupTable[r & 0xf];
    _buffer[4] = lookupTable[b >> 4];
    _buffer[5] = lookupTable[b & 0xf];
    _spi->blockWrite16(6, _buffer);
}
// EOF
