/*
 *  Millisecond timebase, default (tick-based) implementation.
 *
 *  Selected when LN_EXTERNAL_SYSTEM_HELPER is NOT set (non-ESP-IDF MCUs:
 *  CH32V3x, GD32, RP2040, ...).
 *
 *  ESP-IDF does NOT use this file: there the FreeRTOS tick hook
 *  vApplicationTickHook() is never wired up (xPortSysTickHandler calls
 *  esp_vApplicationTickHook(), which only runs callbacks registered via
 *  esp_register_freertos_tick_hook()), so the counter below would stay 0
 *  forever and lnGetMs() would never advance - which made
 *  platform_timeout_is_expired() never return true and the SWD WAIT/FAULT
 *  retry loop in sendHeader() spin indefinitely (Task WDT crash). ESP32 uses
 *  the esp_timer-based implementation instead:
 *  esprit/mcus/riscv_esp32/src/lnSystemTime_esp32.cpp
 */

#include "stdint.h"
#include "lnSystemTime.h"

static uint32_t myTick;

extern "C" void vApplicationTickHook()
{
    // this is not atomic, ok but it is called under interrupt
    myTick++;
}

/**
 * @return The number of milliseconds elapsed since boot.
 *
 * Incremented by the FreeRTOS tick hook. Wraps after ~49.7 days; the
 * signed-arithmetic check in platform_timeout_is_expired() handles 32-bit
 * wrap-around correctly.
 */
uint32_t lnGetMs()
{
    return myTick;
}

extern "C" uint32_t lnGetMs_c()
{
    return myTick;
}
