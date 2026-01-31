#include "esprit.h"
#include "lnGPIO.h"
#include "simplerMCP4725.h"

#define LED LN_SYSTEM_LED

myMCP4725 *currentLimiter;
lnI2C *i2c;

// void i2cScanner();

void setup()
{
    lnPinMode(LED, lnOUTPUT);
    lnPinMode(PB11, lnALTERNATE_OD); // SDA2
    lnPinMode(PB10, lnALTERNATE_PP); // SCLK2
}
/**
 *
 */

int val;
void loop()
{

    lnI2C *i2c = lnI2C::create(1, 100000); // USING I2C2!!

    i2c->begin();

    // i2cScanner();

    currentLimiter = new myMCP4725(*i2c, MCP4725_DEFAULT_I2C_ADDRESS);
    currentLimiter->setVoltage(64);
    currentLimiter->setDefaultValue(1024);
    while (1)
    {
        for (int i = 0; i < 4095; i++)
        {
            currentLimiter->setVoltage(i);
            Logger("%d \n", i);
        }
    }
}
void i2cScanner()
{
    while (1)
    {
        for (int i = 1; i < 127; i++)
        {
            xDelay(100);
            Logger("Scanning %d\n", i);

            i2c->setAddress(i);
            //   while(1)
            {
                if (!i2c->write(0, NULL))
                {
                    xDelay(5);
                    continue;
                }
                else
                {
                    Logger(">>>found something at address %d (0x%0x)\n", i, i);
                }
            }
        }
    }
}
