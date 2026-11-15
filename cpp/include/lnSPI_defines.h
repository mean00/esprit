/*
 *  (C) 2021/2022 MEAN00 fixounet@free.fr
 *  See license file
 *
 * Normal usage :
 * --------------
 *    setup()
 *    begin()
 *    ...
 *    ...
 *    beginTransaction() <= Lock on a specific device till endTransaction
 *    beginSession(8)  <= while in beginSession the SPI stays enabled and the CS stays on (if defined),
 *                             also the bus width stays the same (8/16 bits)
 *    write8
 *    write8
 *    read8
 *    endSession()
 *    beginSession(16)
 *    write16
 *    write16
 *    read16
 *    endSession()
 * endTransaction() <= Other devices can be used
 *
 *
 *
 */

#pragma once
#include "stdint.h"
// Mode          Clock Polarity (CPOL)   Clock Phase (CPHA)
// SPI_MODE0             0                     0
// SPI_MODE1             0                     1
// SPI_MODE2             1                     0
// SPI_MODE3             1                     1
enum spiDataMode
{
    SPI_MODE0 = 0,
    SPI_MODE1 = 1,
    SPI_MODE2 = 2,
    SPI_MODE3 = 3
};
/**
 */
enum spiBitOrder
{
    SPI_LSBFIRST = 0,
    SPI_MSBFIRST = 1
};
/**
 * This is not really used
 */
class lnSPISettings
{
  public:
    lnSPISettings(uint32_t ispeed, spiBitOrder bitOrder, spiDataMode dataMode, int cs)
    {
        speed = ispeed;
        bOrder = bitOrder;
        dMode = dataMode;
        pinCS = cs;
    }

    int pinCS;          // CS pin associated to the configuration
    uint32_t speed;     // specifies the spi bus maximum clock speed
    spiBitOrder bOrder; // bit order (MSBFirst or LSBFirst)
    spiDataMode dMode;  // one of the data mode
};

typedef void lnSpiCallback(void *cookie);
