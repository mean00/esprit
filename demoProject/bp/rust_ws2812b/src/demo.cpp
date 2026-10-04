#include "esprit.h"

extern "C" void user_init();

void setup()
{
}

void loop()
{
    Logger("Starting Rust WS2812B glowing blue demo on PB4\n");
    user_init();
    while (1)
    {
        lnDelayMs(1000);
    }
}
