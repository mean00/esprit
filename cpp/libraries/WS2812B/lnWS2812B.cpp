/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 *  Each bit in the WS2812B stream becomes a byte
 *
 */
#include "lnWS2812B.h"
#include "esprit.h"

#define WS_PREAMBLE 4 * 24 // number of zero byte preamble

/**
 *
 * @param nbLeds
 * @param s
 */
WS2812B::WS2812B(uint32_t nbLeds, lnSPI *s) : WS2812B_base(nbLeds)
{
    _spi = s;
    int up = ((nbLeds + 3) & (~3)) + (WS_PREAMBLE + 23) / 24; // next multiple of 4 + add one before / one after
    _ledsColorSPI = new uint8_t[3 * up * 8];                  // spi expansion
    memset(_ledsColorSPI, 0, 3 * up * 8);
}
/**
 *
 */
void WS2812B::begin()
{
    int div;
    xAssert(_spi);
    _spi->begin();
    // Grab the SPI speed
    int clock = lnPeripherals::getClock(Peripherals::pSPI0); // assume all spi have the same clock
    switch (clock)
    {
    case 128000000:
        div = 19;
        break;
    case 108000000:
        div = 16;
        break;
    case 96000000:
        div = 14;
        break;
    case 72000000:
    default:
        div = 8;
        break;
    }
    _spi->setSpeed(clock / div);
    _spi->setDataMode(SPI_MODE1);
}

/**
 */
WS2812B::~WS2812B()
{
#define XCLR(x)                                                                                                        \
    delete[] x;                                                                                                        \
    x = NULL;
    XCLR(_ledsColorSPI);
}
/**
 *
 * @param value
 */
void WS2812B::setGlobalBrightness(uint32_t value)
{
    WS2812B_base::setGlobalBrightness(value);
    convertAll();
}
/**
 *
 * @param r
 * @param g
 * @param b
 */
void WS2812B::setColor(uint32_t r, uint32_t g, uint32_t b)
{
    WS2812B_base::setColor(r, g, b);
    convertAll();
}
/**
 *
 * @param led
 * @param r
 * @param g
 * @param b
 */
void WS2812B::setLedColor(uint32_t led, uint32_t r, uint32_t g, uint32_t b)
{
    WS2812B_base::setLedColor(led, r, g, b);
    convert(led);
}
/**
 *
 * @param led
 * @param brightness
 */
void WS2812B::setLedBrightness(uint32_t led, uint32_t brightness)
{
    WS2812B_base::setLedBrightness(led, brightness);
    convert(led);
}
/**
 *
 */
void WS2812B::update()
{
    _spi->blockWrite8(_nbLeds * 3 * 8 + WS_PREAMBLE, _ledsColorSPI); // add a couple of zeros at the end
}
/**
 *
 * @param led
 */
#define ZRANK(x, y) ((((x * 0x1fUL) << 3UL) + (((1UL - x) * 0x7UL) << 5UL)) << y)
// #define ZZ(a,b,c,d)   ZRANK(a,24)+ZRANK(b,16)+ZRANK(c,8)+ZRANK(d,0)
#define ZZ(a, b, c, d) ZRANK(a, 0) + ZRANK(b, 8) + ZRANK(c, 16) + ZRANK(d, 24)

static const uint32_t lookupTable[16] = {ZZ(0, 0, 0, 0), ZZ(0, 0, 0, 1), ZZ(0, 0, 1, 0), ZZ(0, 0, 1, 1),
                                         ZZ(0, 1, 0, 0), ZZ(0, 1, 0, 1), ZZ(0, 1, 1, 0), ZZ(0, 1, 1, 1),

                                         ZZ(1, 0, 0, 0), ZZ(1, 0, 0, 1), ZZ(1, 0, 1, 0), ZZ(1, 0, 1, 1),
                                         ZZ(1, 1, 0, 0), ZZ(1, 1, 0, 1), ZZ(1, 1, 1, 0), ZZ(1, 1, 1, 1)};

static void convertOne(int color, int b16, uint8_t *target)
{
    uint32_t *p = (uint32_t *)target;

    color = (color * b16) >> 16;

    p[0] = lookupTable[(color >> 4) & 0xf];
    p[1] = lookupTable[(color) & 0xf];
}
/**
 *
 * @param led
 */
void WS2812B::convert(uint32_t led)
{
    uint8_t *p = _ledsColorSPI + 3 * 8 * led + WS_PREAMBLE;
    uint8_t *c = _ledsColor + led * 3;

    int b = (int)(_ledsBrightness[led]) * (int)(_brightness);

    convertOne(c[0], b, p);
    convertOne(c[1], b, p + 8);
    convertOne(c[2], b, p + 16);
}
/**
 *
 */
void WS2812B::convertAll()
{
    for (int i = 0; i < _nbLeds; i++)
        convert(i);
}

// EOF
