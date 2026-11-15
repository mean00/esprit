/**
 * @file lnSystemTimer_riscv_esp32.cpp
 * @brief System timer for ESP32 (C3/C6/S3): microsecond timebase via the IDF
 *        esp_timer component.
 *
 * The swindle firmware (bmp_interface_c.cpp instrumentation, lnDelayUs,
 * rust_esprit task timing) expects lnGetUs()/lnGetUs64()/lnGetMs() to exist on
 * every MCU. ESP32 is the only supported MCU without a dedicated system-timer
 * file; esp_timer_get_time() provides a uniform monotonic microsecond counter.
 */
#include "esp_timer.h"
#include "lnSystemTime.h"

/**
 * @brief Return the number of microseconds elapsed since boot.
 * @return uint32_t
 */
uint32_t lnGetUs()
{
    return (uint32_t)esp_timer_get_time();
}
/**
 * @brief Return the number of microseconds elapsed since boot (64-bit).
 * @return uint64_t
 */
uint64_t lnGetUs64()
{
    return (uint64_t)esp_timer_get_time();
}
// lnGetMs()/lnGetMs_c() are provided by
// esprit/mcus/riscv_esp32/src/lnSystemTime_esp32.cpp (esp_timer-based).
// The tick-based lnSystemTime.cpp is NOT used on ESP-IDF: the FreeRTOS tick
// hook is never wired up there, so a tick counter would stay 0 forever.
// EOF
