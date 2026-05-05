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

#include "simplerAD9833.h"
#include "esprit.h"
#include "simplerAD9833_internal.h"
/**
 *
 * @param spi
 * @param cs
 */
simplerAD9833::simplerAD9833(lnSPI *spi, lnPin cs, int baseClock)
{
    _spi = spi;
    _cs = cs;
    _baseClock = baseClock;

    float m = (1024.f * (float)(1 << (28))) / (float)baseClock;
    _factor = (int)(m + 0.49f);
    if (_cs != -1)
    {
        lnDigitalWrite(_cs, true);
        lnPinMode(_cs, lnOUTPUT);
    }
    _waveform = Sine;
    _frequency = 1000;
    _state = 0;
    _spiSettings = new lnSPISettings(5 * 1000 * 1000, SPI_MSBFIRST, SPI_MODE2, -1);

    writeRegister(0, LN_AD9833_RESET);
    xDelay(1);
    writeRegister(0, 0);
}
/**
 *
 */
simplerAD9833::~simplerAD9833()
{
    _spi = NULL;
    delete _spiSettings;
    _spiSettings = NULL;
}
/**
 *
 * @param addr
 * @return
 */
int simplerAD9833::readRegister(int addr)
{
    return 0;
}
/**
 *
 * @param wf
 */
void simplerAD9833::setWaveForm(WaveForm wf)
{
    _state &= ~(LN_AD9833_OPBITEN + LN_AD9833_DIV2 + LN_AD9833_MODE);
    _waveform = wf;
#define TTT(op, mode, div2)                                                                                            \
    {                                                                                                                  \
        _state += op * LN_AD9833_OPBITEN + mode * LN_AD9833_MODE + LN_AD9833_DIV2 * div2;                              \
    }
    switch (_waveform)
    {
    case Square:
        TTT(1, 0, 1);
        break;
    case Triangle:
        TTT(0, 1, 0);
        break;
    case Sine:
        TTT(0, 0, 0);
        break;
    }
    writeRegister(0, _state);
}

/**
 *
 */
void simplerAD9833::enable()
{
    _state &= ~(LN_AD9833_SLEEP1 + LN_AD9833_SLEEP12);
    writeRegister(0, _state);
}
/**
 *
 */
void simplerAD9833::disable()
{
    _state |= (LN_AD9833_SLEEP1 + LN_AD9833_SLEEP12);
    writeRegister(0, _state);
}

/**
 *
 * @param fq
 */
void simplerAD9833::setFrequency(int fq)
{
    _frequency = fq;
    if (fq > _baseClock)
        fq = _baseClock;
    if (!fq)
        fq = 1;

    uint64_t m = (uint64_t)fq * (uint64_t)_factor; // ~ 10*256
    uint32_t n = (uint32_t)(m >> 10);

    int High14 = n >> 14;
    int Low14 = n & 0x3FFF;
    // this is inefficient, but the amount of data transfered is very low
    // so it does not really matter

    writeRegister(0, LN_AD9833_B28 + _state);  // Wrote
    writeRegister(0, LN_AD9833_FREQ + Low14);  // Wrote
    writeRegister(0, LN_AD9833_FREQ + High14); // Wrote
}

/**
 * \brief It seems the AD9833 does not like the CS to go up quickly, so manually
 * drive it and add a small delay at the end of the transmittion
 * @param addr
 * @param value
 */
void simplerAD9833::writeRegister(int addr, int value)
{
    _spi->begin(16);
    lnDigitalWrite(_cs, false);
    _spi->write16(value);
    lnDelayUs(1);
    lnDigitalWrite(_cs, true);
    _spi->end();
}

// EOF
