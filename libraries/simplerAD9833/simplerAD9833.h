/**
    This is a library for the MCP23017 i2c port expander
    It is derived from the adafruit one with the following change :
        * All Axxx port are inputs
        * All Bxxx ports are outputs
        * Additional methods to easily manage rotarty encoder and input button
 *
 * It is using ARM friendly convention i.e. using "int" as much as possible
 * (c) mean00, BSD License
 * Datasheet :
 * http://ww1.microchip.com/downloads/en/DeviceDoc/20001952C.pdf
 *
 */

/***************************************************
  This is a library for the MCP23017 i2c port expander

  These displays use I2C to communicate, 2 pins are required to
  interface
  Adafruit invests time and resources providing this open source code,
  please support Adafruit and open-source hardware by purchasing
  products from Adafruit!

  Written by Limor Fried/Ladyada for Adafruit Industries.
  BSD license, all text above must be included in any redistribution
 ****************************************************/
#pragma once
#include "lnGPIO.h"
#include "lnSPI.h"
#include "vector"

/**
 *
 * @param pinInterrupt
 * @param addr
 * @param w
 */
class simplerAD9833
{
  public:
    enum WaveForm
    {
        Square,
        Triangle,
        Sine
    };

    simplerAD9833(lnSPI *spi, lnPin cs, uint32_t baseClock = 25000000);
    virtual ~simplerAD9833();
    void enable();
    void disable();
    void setWaveForm(WaveForm wf);
    void setFrequency(uint32_t frequency);

  protected:
    uint32_t readRegister(uint32_t addr);
    void writeRegister(uint32_t addr, uint32_t value);
    lnSPI *_spi;
    lnPin _cs;
    WaveForm _waveform;
    int _frequency;
    int _state;
    lnSPISettings *_spiSettings;
    int _baseClock;
    int _factor; // fixed decimal multiplier
};
