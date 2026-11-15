/*
 *
 */
#pragma once
extern "C"
{
#include "hardware/structs/clocks.h"
#if RP_SDK_VERSION == 1
    uint32_t clock_get_hz(enum clock_index clk_index);
#elif RP_SDK_VERSION == 2
    typedef clock_num_t clock_handle_t;
    uint32_t clock_get_hz(clock_handle_t clk_index);
#else
#error RP_SDK_VERSION undefined
#endif
}
