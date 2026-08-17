/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */

#include "stdint.h"

// #pragma clang diagnostic ignored "-Wextra"

extern "C"
{
// #include "riscv_encoding.h"
#include "lnFreeRTOS.h"
}
#include "systemHelper.h"

#include "lnSystemTime.h"
extern "C" void do_assert(const char *a);

/**
 */

extern "C"
{
    void deadEnd(int code);
    // extern void taskENTER_CRITICAL(void);
    // extern void taskEXIT_CRITICAL(void);

    uintptr_t handle_trap(uintptr_t mcause, uintptr_t sp)
    {
        deadEnd(0xffff);
        return 0;
    }

    __attribute__((interrupt)) void unhandledException(void)
    {
        deadEnd(0x1000);
    }
    void lnInterrupts()
    {
        EXIT_CRITICAL();
    }
    void lnNoInterrupt()
    {
        ENTER_CRITICAL();
    }
#ifndef LN_CUSTOM_FREERTOS
    void vApplicationStackOverflowHook(TaskHandle_t xTask, char *pcTaskName)
    {
        deadEnd(0x1001);
    }
#endif
}

/**

*/
void lnDelayUs(uint32_t wait)
{
    uint64_t target = lnGetUs() + wait;
    while (1)
    {
        uint64_t vw = lnGetUs();
        if (vw > target)
            return;
        __asm__("nop" ::);
    }
}

// The millisecond timebase (lnGetMs / lnGetMs_c) and the FreeRTOS tick hook
// that feeds it are NOT common code: under ESP-IDF (LN_ESPRESSIF) the hook is
// never wired up (xPortSysTickHandler calls esp_vApplicationTickHook(), which
// only runs callbacks registered via esp_register_freertos_tick_hook()), so a
// tick-counter lnGetMs() would stay 0 forever - platform_timeout_is_expired()
// would never return true and the SWD WAIT/FAULT retry loop in sendHeader()
// would spin indefinitely (Task WDT crash).
// The build selects one implementation via LN_EXTERNAL_SYSTEM_HELPER:
//   - default (tick-based):        esprit/src/lnSystemTime.cpp
//   - ESP-IDF (esp_timer-based):   esprit/mcus/riscv_esp32/src/lnSystemTime_esp32.cpp

void lnDelay(uint32_t a);
/**
 */
void xDelay(uint32_t wait)
{
    lnDelay(wait);
}

// EOF
