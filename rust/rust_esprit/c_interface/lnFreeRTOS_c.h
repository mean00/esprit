#include "lnFreeRTOS.h"
#include "include/timers.h"
#include "stdbool.h"
#include "stdint.h"

// C API that the Rust half calls.
//
// The numbers Rust needs from the kernel are deliberately NOT exported here:
// sizeof(StaticQueue_t) and configTICK_RATE_HZ describe the kernel instance we
// link (esprit/FreeRTOS + FreeRTOSConfig.h, or ESP-IDF's copy of FreeRTOS, which
// differ), so they live in esprit/rust/rs_rtos/src/kernel/*.rs, one file per
// kernel.  Keeping them out of this header keeps bindgen a pure function of the
// sources: it no longer needs a build directory (freertos_struct.h) to run, and
// the generated file no longer bakes in one target's struct sizes.
extern "C" void Logger_chars(int n, const char *data);
