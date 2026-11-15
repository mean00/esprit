/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */

#pragma once

struct LN_SYSTIMERx
{
    uint32_t MTIME_LO;
    uint32_t MTIME_HI;
    uint32_t MTIMECMP_LO;
    uint32_t MTIMECMP_HI;
};

struct LN_SYSTIMER64x
{
    uint64_t MTIME64;
    uint64_t MTIMECMP64;
};

typedef volatile LN_SYSTIMERx LN_SYSTIMER;
typedef volatile LN_SYSTIMER64x LN_SYSTIMER64;
