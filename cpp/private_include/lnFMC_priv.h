/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */

#pragma once

struct LN_FMCx
{
    uint32_t WS;
    uint32_t KEY;
    uint32_t OBKEY;
    uint32_t STAT;
    uint32_t CTL;
    uint32_t ADDR;   // 0x14
    uint32_t dummy1; // 0x18
    uint32_t OBSTAT; // 0x1c
    uint32_t WP;     // 0x20
};

typedef volatile LN_FMCx LN_FMC;

// STAT
#define LN_FMC_STAT_BUSY (1 << 0)
#define LN_FMC_STAT_PG_ERR (1 << 3)  // program error
#define LN_FMC_STAT_WP_ERR (1 << 4)  // erase / program erro
#define LN_FMC_STAT_WP_ENDF (1 << 5) // end of operation

// CTL
#define LN_FMC_CTL_PG (1 << 0)     // program command
#define LN_FMC_CTL_PER (1 << 1)    // page erase command
#define LN_FMC_CTL_MER (1 << 2)    // mass erase command
#define LN_FMC_CTL_OBPG (1 << 4)   // option byte
#define LN_FMC_CTL_OBER (1 << 5)   // option byte
#define LN_FMC_CTL_START (1 << 6)  // erase command
#define LN_FMC_CTL_LK (1 << 7)     // lock bit
#define LN_FMC_CTL_OBWEN (1 << 9)  // Option erase
#define LN_FMC_CTL_ERRIE (1 << 10) // Erase Interrupt enable
#define LN_FMC_CTL_ENDIE (1 << 12) // End of operation interrupt enable
