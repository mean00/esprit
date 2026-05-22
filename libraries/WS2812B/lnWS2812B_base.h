/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */
#pragma once
#include "esprit.h"
class WS2812B_base
{
  public:
    WS2812B_base(uint32_t nbLeds);
    virtual ~WS2812B_base();

    void setGlobalBrightness(uint8_t value);                         // between 0 & 255
    void setColor(uint8_t r, uint8_t g, uint8_t b);                  // set all the same color
    void setLedColor(uint32_t led, uint8_t r, uint8_t g, uint8_t b); // set only one led
    void setLedColors(uint32_t start, uint32_t nb_led, const uint8_t *data);
    void setLedBrightness(uint32_t led, uint8_t brightness); // set only one led
    virtual void update() = 0;                               // call this to have the changes committed

  protected:
    uint32_t _nbLeds;
    uint8_t *_ledsColor;
    uint8_t *_ledsBrightness;
    uint8_t _brightness;
};
