

#pragma once
#define LN_FAST_CODE __attribute__((section(".ln_copy_to_ram.")))
// #define LN_FAST_CODE
void lnPinModePIO(const lnPin pin, const int instance, const bool pullup=false);
//
