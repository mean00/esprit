//
#include "esprit.h"
#include "lnI2C.h"

// green = PA1, blue = PA2, RED PC13
#define LED PA2
#define LEDRED PC13
#define SPEED (200 * 1000)

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
    Logger("ISC Scannner demo...\n");

    lnI2C *i2c = lnI2C::create(1, SPEED); // USING I2C2!!
    i2c->begin();
    while (1)
    {
        for (int i = 01; i < 127; i++)
        {
            xDelay(100);
            Logger("Scanning %d\n", i);

            i2c->setAddress(i);
            //   while(1)
            {
                if (!i2c->write(0, NULL))
                {
                    xDelay(100);
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
// EOF
