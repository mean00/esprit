/*
 *  Millisecond timebase for ESP-IDF (ESP32-C3/C6/S3).
 *
 *  Selected when LN_EXTERNAL_SYSTEM_HELPER is set (see
 *  esprit/mcus/riscv_esp32/toolchain.cmake and esprit/CMakeLists.txt).
 *
 *  Uses the monotonic esp_timer timebase instead of the FreeRTOS tick hook:
 *  under ESP-IDF xPortSysTickHandler calls esp_vApplicationTickHook(), which
 *  only runs callbacks registered via esp_register_freertos_tick_hook(), so a
 *  plain vApplicationTickHook() would never be called and lnGetMs() would
 *  stay 0 forever - platform_timeout_is_expired() would never return true and
 *  the SWD WAIT/FAULT retry loop in sendHeader() would spin indefinitely
 *  (Task WDT crash). Does not depend on the tick hook being registered.
 *
 *  Wraps after ~49.7 days; the signed-arithmetic check in
 *  platform_timeout_is_expired() handles 32-bit wrap-around correctly.
 */

#include "stdint.h"
#include "lnSystemTime.h"
#include "esp_timer.h"

uint32_t lnGetMs()
{
    return (uint32_t)(esp_timer_get_time() / 1000ULL);
}

extern "C" uint32_t lnGetMs_c()
{
    return (uint32_t)(esp_timer_get_time() / 1000ULL);
}
