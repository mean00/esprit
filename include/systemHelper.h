/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */

#pragma once
extern "C"
{
    void deadEnd(int code);
}
#ifdef CRITICAL_SECTION_EXTRA_ARG
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
extern portMUX_TYPE my_spinlock;
#define ENTER_CRITICAL() vPortEnterCritical(&my_spinlock)
#define EXIT_CRITICAL() vPortExitCritical(&my_spinlock)
#else
#define ENTER_CRITICAL() vPortEnterCritical()
#define EXIT_CRITICAL() vPortExitCritical()
#endif

// extern "C" void ENTER_CRITICAL(void);
// extern "C" void EXIT_CRITICAL(void);
