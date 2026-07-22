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

static uint32_t myTick;
extern "C" void vApplicationTickHook()
{
    // this is not atomic, ok but it is called under interrupt
    myTick++;
}

/**
 *
 * @return
 */
uint32_t lnGetMs()
{
    return myTick;
}
extern "C" uint32_t lnGetMs_c()
{
    return myTick;
}

void lnDelay(uint32_t a);
/**
 */
void xDelay(uint32_t wait)
{
    lnDelay(wait);
}

// EOF
