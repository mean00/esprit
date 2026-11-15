/**************************************************************************/
/*!
    @file     Adafruit_MCP4725.cpp
    @author   K.Townsend (Adafruit Industries)
    @license  BSD (see license.txt)

    I2C Driver for Microchip's MCP4725 I2C DAC

    This is a library for the Adafruit MCP4725 breakout
    ----> http://www.adafruit.com/products/935

    Adafruit invests time and resources providing this open source code,
    please support Adafruit and open-source hardware by purchasing
    products from Adafruit!

    @section  HISTORY

    v1.0 - First release
*/
/**************************************************************************/

#include "simplerMCP4725.h"

#define MCP4726_CMD_WRITEDAC (2 << 5) // Writes data to the DAC
#define MCP4726_CMD_WRITEDACEEPROM                                                                                     \
    (3 << 5) // Writes data to the DAC and the EEPROM (persisting the assigned value after reset)

/**************************************************************************/
/*!
    @brief  Instantiates a new MCP4725 class
*/
/**************************************************************************/
myMCP4725::myMCP4725(lnI2C &wire, uint32_t address) : _i2c(wire)
{
    _i2caddr = address;
}

/**************************************************************************/
/*!
    @brief  Sets the output voltage to a fraction of source vref.  (Value
            can be 0..4095)

    @param[in]  output
                The 12-bit value representing the relationship between
                the DAC's input voltage and its output voltage.
    @param[in]  writeEEPROM
                If this value is true, 'output' will also be written
                to the MCP4725's internal non-volatile memory, meaning
                that the DAC will retain the current voltage output
                after power-down or reset.
*/
/**************************************************************************/
void myMCP4725::setVoltage(uint32_t output)
{
#if 0
  // slow
  uint8_t datas[3]={MCP4726_CMD_WRITEDAC,(uint8_t)(output>>4),(output&0xf)<<4};
  _i2c.write(_i2caddr,3,datas);
#else // fast
    uint8_t datas[2] = {(uint8_t)((output >> 8) & 0xf), (uint8_t)(output & 0xff)};
    _i2c.write(_i2caddr, 2, datas);
#endif
}
/**
 *
 * @param output
 * @param writeEEPROM
 */
void myMCP4725::setDefaultValue(uint32_t output)
{
    uint8_t datas[3] = {MCP4726_CMD_WRITEDACEEPROM, (uint8_t)(output >> 4), (uint8_t)((output & 0xf) << 4)};
    _i2c.write(_i2caddr, 3, datas);
}
// EOF
