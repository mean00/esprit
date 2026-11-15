/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */

#pragma once

struct LN_GPIOx
{
    uint32_t CTL0;
    uint32_t CTL1;
    uint32_t ISTAT;
    uint32_t OCTL;
    uint32_t BOP;
    uint32_t BC;
    uint32_t LOCK;
};

typedef volatile LN_GPIOx LN_GPIO;

#define LN_CTL_MD_INPUT 0

#define LN_CTL_INPUT_ANALOG 0
#define LN_CTL_INPUT_FLOATING 1
#define LN_CTL_INPUT_PULLUPPULLDOWN 2
#define LN_OCTL_INPUT_PULLUP 1
#define LN_OCTL_INPUT_PULLDOWN 0

#define LN_CTL_OUTPUT_PP 0
#define LN_CTL_OUTPUT_OD 1
#define LN_CTL_OUTPUT_ALTERNAT_PP 2
#define LN_CTL_OUTPUT_ALTERNAT_OD 3

#define LNGPIOSET(mod, ctl) ((ctl << 2) + mod)

#define LN_GPIO_TIMER0_REMAP_POS 6
#define LN_GPIO_TIMER1_REMAP_POS 8
#define LN_GPIO_TIMER2_REMAP_POS 10
//
