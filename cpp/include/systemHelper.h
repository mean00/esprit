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
// On RISC-V Espressif chips (ESP32-C3/C6), the IDF 6.x portmacro.h declares the
// single-core vPortEnterCritical(void)/vPortExitCritical(void) and
// portENTER_CRITICAL(mux) requires a (discarded) mux argument, so the no-argument
// portENTER_CRITICAL() does not compile there. Narrowed to Espressif (LN_ESPRESSIF
// set by mcus/riscv_esp32/toolchain.cmake) so other RISC-V targets (CH32V3x, GD32)
// keep using their plain portENTER_CRITICAL().
#if defined(__riscv) && defined(LN_ESPRESSIF)
#define ENTER_CRITICAL() vPortEnterCritical()
#define EXIT_CRITICAL() vPortExitCritical()
#else
#define ENTER_CRITICAL() portENTER_CRITICAL()
#define EXIT_CRITICAL() portEXIT_CRITICAL()
#endif
#endif

// extern "C" void ENTER_CRITICAL(void);
// extern "C" void EXIT_CRITICAL(void);
