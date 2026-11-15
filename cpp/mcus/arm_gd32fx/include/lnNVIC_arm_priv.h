/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */

#pragma once

struct LN_NVIC_PACK
{
    uint32_t data[8];
    uint32_t dummy[24];
};

struct LN_NVICx
{
    volatile LN_NVIC_PACK ISER; // Set enable
    volatile LN_NVIC_PACK ICER; // Clear enable
    volatile LN_NVIC_PACK ISPR; // Set pending
    volatile LN_NVIC_PACK IPCR; // Clear pending
    volatile LN_NVIC_PACK IABR; // Active
    uint32_t dummy1[56 - 24];
    uint8_t IP[240]; // Priority
    uint32_t dummy2[644];
    uint32_t STIR; // Software trigger
};
typedef volatile LN_NVICx LN_NVIC;

// EOF
