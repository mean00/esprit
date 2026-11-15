#include "lnI2C.h"
#include "mcp23017.h"
/**
 *
 * @param i2c
 * @param irqPin
 */
class lnMcp23017 : public myMcp23017
{
  public:
    lnMcp23017(lnI2C *i2c, int irqPin, int address) : myMcp23017(address)
    {
        _i2c = i2c;
        _irqPin = irqPin;
    }
    uint8_t readRegister(int addr);
    void writeRegister(int addr, int value);
    virtual bool begin();

  protected:
    lnI2C *_i2c;
    int _irqPin;
};