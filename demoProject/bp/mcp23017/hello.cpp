//
#include "esprit.h"
#include "lnI2C.h"
#include "lnMcp23017.h"
// green = PA1, blue = PA2, RED PC13
#define LED PA2
#define LEDRED PC13
#define SPEED (400 * 1000)

/**
 *
 */
void setup()
{
}

/**
 *
 */
void loop()
{
    Logger("MCP23017 I2C demo...\n");

    lnI2C *i2c = lnI2C::create(1, 100000); // USING I2C2!!
    i2c->begin(0x20);

    lnMcp23017 *mcp = new lnMcp23017(i2c, PB5, 0x20);
    mcp->begin();
    bool onoff = false;
    lnPinMode(LEDRED, lnOUTPUT);
    while (1)
    {

        mcp->digitalWrite(0, onoff);
        lnDigitalWrite(LEDRED, onoff);
        bool readBack = mcp->readOutput(0);
        bool readBack5 = mcp->readOutput(5);
        bool readIn = mcp->readInput(0);
        Logger("Set %d, read %d,read 5:%d, input %d\n", onoff, readBack, readBack5, readIn);
        onoff = !onoff;
        //  mcp->dumpRegisters();
        xDelay(500);
    }
}

// EOF
