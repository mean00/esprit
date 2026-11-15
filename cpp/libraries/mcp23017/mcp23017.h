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
#include "vector"
#include "esprit.h"
class myMcpClient;
/**
 *
 * @param pinInterrupt
 * @param addr
 * @param w
 */
class myMcp23017
{
  public:
    /**
     * \fn digitalWrite
     * @param pin 0..7 matching b0...b7
     * @param value
     */
    virtual void digitalWrite(int pin, bool onoff);
    bool readOutput(int pin);
    bool readInput(int pin);
    void registerClient(int mask, myMcpClient *client);
    virtual bool begin();
    bool dumpRegisters();

  protected:
    virtual uint8_t readRegister(int addr) = 0;
    virtual void writeRegister(int addr, int value) = 0;

    myMcp23017(int address)
    {
        _address = address;
    }

    int _address;
    int inputMask;
    int PortALatch;
    int PortBValue;
    bool changed;
    lnBinarySemaphore _sem;
};
