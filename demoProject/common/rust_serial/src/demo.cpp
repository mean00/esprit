#include "esprit.h"

extern "C" void user_init();

void setup() {
    Logger("Setup C++ harness for Serial...\n");
    user_init();
}

void loop() {
    lnDelayMs(1000);
}

