/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */

#pragma once

struct LN_EXTI_Registerx
{
    uint32_t INTEN; // 0 Interrupt enable
    uint32_t EVEN;  //  4 Event enable
    uint32_t RTEN;  //  8  Rising
    uint32_t FTEN;  //  C  Falling
    uint32_t SWIEV; // 10  Sw Event
    uint32_t PD;    // 14  Pending
};

typedef volatile LN_EXTI_Registerx LN_EXTI_Register;
