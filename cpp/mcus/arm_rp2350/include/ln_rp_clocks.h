/*
 *
 */
#pragma once
extern "C"
{
#include "hardware/structs/clocks.h"
    typedef clock_num_t clock_handle_t;
    uint32_t clock_get_hz(clock_handle_t clk_index);
}
