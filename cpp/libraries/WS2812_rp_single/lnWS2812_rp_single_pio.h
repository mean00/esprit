#pragma once

#include "hardware/pio.h"
#include "lnGPIO.h"
/**
 * @brief
 *
 */
class WS2812_rp2040_pio_single
{
  public:
    WS2812_rp2040_pio_single(int pio_instance, int sm, lnPin pin);
    virtual ~WS2812_rp2040_pio_single();
    void setColor(uint8_t r, uint8_t g, uint8_t b);

  protected:
    PIO _pio;
    int _sm;
};