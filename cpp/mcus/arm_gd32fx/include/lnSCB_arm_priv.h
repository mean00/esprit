/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */
#pragma once

struct LN_SCB_Registersx
{
    uint32_t CPUID;
    uint32_t ICSR;
    uint32_t VTOR;
    uint32_t AIRCR;
    uint32_t SCR;
    uint32_t CR;
    uint8_t SHP[12]; // system exception priority
    uint32_t SHCSR;
};

typedef volatile LN_SCB_Registersx LN_SCB_Registers;
