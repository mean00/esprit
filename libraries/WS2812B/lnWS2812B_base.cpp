/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 *  Each bit in the WS2812B stream becomes a byte
 *
 */
#include "lnWS2812B_base.h"
#include "esprit.h"

/**
 *
 * @param nbLeds
 * @param s
 */
WS2812B_base::WS2812B_base(uint32_t nbLeds)
{
    _nbLeds = nbLeds;
    _brightness = 0xff;
    _ledsColor = new uint8_t[3 * nbLeds];
    _ledsBrightness = new uint8_t[nbLeds];
    for (int i = 0; i < nbLeds; i++)
    {
        _ledsColor[3 * i + 0] = 0;
        _ledsColor[3 * i + 1] = 0;
        _ledsColor[3 * i + 2] = 0;
        _ledsBrightness[i] = 255;
    }
}

/**
 */
WS2812B_base::~WS2812B_base()
{
#define XCLR(x)                                                                                                        \
    delete[] x;                                                                                                        \
    x = NULL;
    XCLR(_ledsBrightness);
    XCLR(_ledsColor);
}
/**
 *
 * @param value
 */
void WS2812B_base::setGlobalBrightness(uint8_t value)
{
    _brightness = value;
}
/**
 *
 * @param r
 * @param g
 * @param b
 */
void WS2812B_base::setColor(uint8_t r, uint8_t g, uint8_t b)
{
    uint8_t *pr = _ledsColor;
    uint8_t *pg = _ledsColor + 1;
    uint8_t *pb = _ledsColor + 2;
    for (int i = 0; i < _nbLeds; i++)
    {
        *pr = r;
        *pg = g;
        *pb = b;
        pr += 3;
        pb += 3;
        pg += 3;
    }
}
/**
 *
 * @param led
 * @param r
 * @param g
 * @param b
 */
void WS2812B_base::setLedColor(uint32_t led, uint8_t r, uint8_t g, uint8_t b)
{
    uint8_t *p = _ledsColor + led * 3;
    p[0] = g;
    p[1] = r;
    p[2] = b;
}
/**
 *
 * @param led
 * @param brightness
 */
void WS2812B_base::setLedColors(uint32_t start, uint32_t nb_led, const uint8_t *data)
{
    xAssert(start + nb_led <= this->_nbLeds);
    uint8_t *p = _ledsColor + start * 3;
    const uint8_t *src = data;
    for (int i = 0; i < nb_led; i++)
    {
        p[0] = src[1];
        p[1] = src[0];
        p[2] = src[2];
        p += 3;
        src += 3;
    }
}
/**
 *
 * @param led
 * @param brightness
 */
void WS2812B_base::setLedBrightness(uint32_t led, uint8_t brightness)
{
    _ledsBrightness[led] = brightness;
}

// EOF
