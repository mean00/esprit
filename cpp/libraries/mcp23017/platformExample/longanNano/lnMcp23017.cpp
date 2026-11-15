#include "lnMcp23017.h"
#include "lnExti.h"
/**
 *
 * @param addr
 * @return
 */
uint8_t lnMcp23017::readRegister(int addr)
{

    uint8_t a = addr, b;
    if (!_i2c->write(1, &a))
        return 0;
    if (!_i2c->read(1, &b))
        return 0;
    return b;
}
/**
 *
 * @param addr
 * @param value
 */
void lnMcp23017::writeRegister(int addr, int value)
{
    uint8_t a[2] = {(uint8_t)addr, (uint8_t)value};
    if (!_i2c->write(2, a))
    {
        return;
    }
}

/**
 *
 * @return
 */
bool lnMcp23017::begin()
{
    // step 1, init MCP itself
    if (!myMcp23017::begin())
        return false;
    return true;
}
// Fail