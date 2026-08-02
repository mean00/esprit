#include "lnFreeRTOS.h"
#include "stdbool.h"
#include "stdint.h"

extern "C" void Logger_chars(int n, const char *data);

// Expose the FreeRTOS kernel tick rate to Rust.
// bindgen cannot evaluate the cast in `#define configTICK_RATE_HZ ((TickType_t)1000)`,
// so we hand it a plain constant folded from the config.
static const uint32_t configTICK_RATE_HZ_RUST = (uint32_t)configTICK_RATE_HZ;
