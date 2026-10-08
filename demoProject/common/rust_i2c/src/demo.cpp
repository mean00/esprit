#include "lnArduino.h"

extern "C" void user_init();

void setup() {
    Logger("Setup C++ harness for I2C...\n");
    user_init();
}

void loop() {
    lnDelay(1000);
}
