#include "lnArduino.h"

extern "C" void user_init();

void setup() {
    Logger("Setup C++ harness...\n");
    user_init();
}

void loop() {
    lnDelay(1000);
}
