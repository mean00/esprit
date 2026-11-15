/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */
#pragma once
#include "lnSPI.h"
#include "lnWS2812B_base.h"
/**
 *
 * @param nbLeds
 * @param s
 */
class WS2812B : public WS2812B_base
{
  public:
    WS2812B(uint32_t nbLeds, lnSPI *s);
    virtual ~WS2812B();

    void begin();                                   // call this first
    void setGlobalBrightness(uint32_t value);            // between 0 & 255
    void setColor(uint32_t r, uint32_t g, uint32_t b);             // set all the same color
    void setLedColor(uint32_t led, uint32_t r, uint32_t g, uint32_t b); // set only one led
    void setLedBrightness(uint32_t led, uint32_t brightness); // set only one led
    void update();                                  // call this to have the changes committed

  protected:
    lnSPI *_spi;
    uint8_t *_ledsColorSPI;

  protected:
    void convert(uint32_t led);
    void convertAll();
};
