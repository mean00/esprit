/**
 * @file    rp_serial.cpp
 * @author  mean00
 * @brief
 * @version 0.1
 * @date 2023-11-01
 *
 * @copyright Copyright (c) 2023
 *
 */
// clang-format off
#include "ln_rp.h" 
#include "esprit.h"
#include "lnRCU.h"
#include "hardware/clocks.h"
#include "pico/time.h"

// clang-format on
/**

 */
uint32_t lnPeripherals::getClock(const Peripherals periph)
{
    switch (periph)
    {
    case pSYSCLOCK:
        return clock_get_hz(clk_sys);
        break;
    default:
        xAssert(0);
        break;
    }
}
/**
 *
 *
 */
uint64_t lnGetUs64()
{
    return to_us_since_boot(get_absolute_time());
}
/**
 *
 *
 */
uint32_t lnGetUs()
{
    return lnGetUs64() & 0xFFFFFFFFUL;
}

//- EOF
