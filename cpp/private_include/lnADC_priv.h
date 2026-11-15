/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */
#pragma once
#include "lnPeripheral_priv.h"
struct LN_ADC_Registersx
{
    uint32_t STAT;     // 0
    uint32_t CTL0;     // 4
    uint32_t CTL1;     // 8
    uint32_t SAMPT[2]; // 0c 10
    uint32_t IOFF[4];  // 14 x20
    uint32_t WDHT;     // 24
    uint32_t WDLT;     // 28
    uint32_t RSQS[3];  // 2c 30 34
    uint32_t ISQ;      // 38
    uint32_t IDATA[4]; // 3c 40 44 48
    uint32_t RDATA;    // 4c
    uint32_t dummy[3 * 4];
    uint32_t OVRS;
    ; // 50
};

// // 0x80

typedef volatile LN_ADC_Registersx LN_ADC_Registers;

#define LN_ADC_SAMPT_1_5 0
#define LN_ADC_SAMPT_7_5 1
#define LN_ADC_SAMPT_13_5 2
#define LN_ADC_SAMPT_28_5 3
#define LN_ADC_SAMPT_41_5 4
#define LN_ADC_SAMPT_55_5 5
#define LN_ADC_SAMPT_71_5 6
#define LN_ADC_SAMPT_239_5 7

// STAT
#define LN_ADC_STAT_WDE (1 << 0)  // watchdog flag
#define LN_ADC_STAT_EOC (1 << 1)  // end of conversion
#define LN_ADC_STAT_EOIC (1 << 2) // end of inserted conversion
#define LN_ADC_STAT_STIC (1 << 3) // start inserted
#define LN_ADC_STAT_STRC (1 << 4) // start

// CTL0

#define LN_ADC_CTL0_EOCIE (1 << 5)  // end interrupt
#define LN_ADC_CTL0_WDEIE (1 << 6)  // watchdog interrupt
#define LN_ADC_CTL0_EOICIE (1 << 7) // End of inserted interrupt
#define LN_ADC_CTL0_SM (1 << 8)     // Scan mode
#define LN_ADC_CTL0_WDSC (1 << 9)   // Watchdog single channel enable
#define LN_ADC_CTL0_ICA (1 << 10)   // inserted auto
#define LN_ADC_CTL0_DISRC (1 << 11) // discontinuous mode on regular channel
#define LN_ADC_CTL0_DISIC (1 << 12) // discontinuous mode on inserted channels
#define LN_ADC_CTL0_IWDEN (1 << 22) // inserted watchdog enable
#define LN_ADC_CTL0_RWDEN (1 << 23) // regular watchdog enable

#define LN_ADC_CTL0_DISNUM_MASK (~(0x7 << 13)) // Number of conversion in discontinuous mode
#define LN_ADC_CTL0_DISNUM_SET(x) ((x << 13))  // Number of conversion in discontinuous mode

#define LN_ADC_CTL0_SYNCM_FREE_MODE 0
#define LN_ADC_CTL0_SYNCM_COMBINED 1

#define LN_ADC_CTL0_SYNCM_INSERTED_ONLY 5
#define LN_ADC_CTL0_SYNCM_REGULAR_ONLY 6
#define LN_ADC_CTL0_SYNCM_FAST_FOLLOW_UP 7
#define LN_ADC_CTL0_SYNCM_SLOW_FOLLOW_UP 8

#define LN_ADC_CTL0_SYNCM_MASK (~(0xf << 16)) // Sync mode selection
#define LN_ADC_CTL0_SYNCM_SET(x) ((x << 16))  // Sync mode selection

#define LN_ADC_CTL0_WDCHSEL_MASK (~(0x1f << 0)) // Watchdog channel select
#define LN_ADC_CTL0_WDCHSEL_SET(x) ((x << 0))   // Watchdog channel select

// CTL1

#define LN_ADC_CTL1_ADCON (1 << 0)   // ADC enable
#define LN_ADC_CTL1_CTN (1 << 1)     // Continuous mode
#define LN_ADC_CTL1_CLB (1 << 2)     // calibration
#define LN_ADC_CTL1_RSTCLB (1 << 3)  // reset calibration
#define LN_ADC_CTL1_DMA (1 << 8)     // DMA
#define LN_ADC_CTL1_DAL (1 << 11)    // Data alignment
#define LN_ADC_CTL1_ETEIC (1 << 15)  // external trigger for inserted
#define LN_ADC_CTL1_ETERC (1 << 20)  // external trigger for regular
#define LN_ADC_CTL1_SWICST (1 << 21) // start on inserted channel
#define LN_ADC_CTL1_SWRCST (1 << 22) // start on regular channel
#define LN_ADC_CTL1_TSVREN (1 << 23) // enable channel 16/17

#define LN_ADC_CTL1_ETSIC_MASK (~(0x7 << 12)) // External trigger source / inserted
#define LN_ADC_CTL1_ETSIC_SET(x) ((x << 12))

#define LN_ADC_CTL1_ETSRC_SOURCE_T0CH0 0
#define LN_ADC_CTL1_ETSRC_SOURCE_T0CH1 1
#define LN_ADC_CTL1_ETSRC_SOURCE_T0CH2 2
#define LN_ADC_CTL1_ETSRC_SOURCE_T1CH1 3
#define LN_ADC_CTL1_ETSRC_SOURCE_T2TRGO 4
#define LN_ADC_CTL1_ETSRC_SOURCE_T3CH3 5
#define LN_ADC_CTL1_ETSRC_SOURCE_EXTI11 6
#define LN_ADC_CTL1_ETSRC_SOURCE_SW 7

#define LN_ADC_CTL1_ETSRC_MASK (~(0x7 << 17)) // External trigger source / regular
#define LN_ADC_CTL1_ETSRC_SET(x) ((x << 17))

// OVRS
#define LN_ADC_OVRS_OVSEN (1 << 0) // oversampling enable
#define LN_ADC_CTL1_CTN (1 << 1)   // Continuous mode

#define LN_ADC_OVRS_OVSR_TOVS (1 << 9)      // Oversampling ratio
#define LN_ADC_OVRS_OVSR_MASK (~(0x7 << 2)) // Oversampling ratio
#define LN_ADC_OVRS_OVSR_SET(x) ((x << 2))

#define LN_ADC_OVRS_OVSS_MASK (~(0xf << 5)) // Oversampling shift
#define LN_ADC_OVRS_OVSS_SET(x) ((x << 5))

#define LN_ADC_OVRS_DRES_MASK (~(0x3 << 12)) // ADC resolution
#define LN_ADC_OVRS_DRES_SET(x) ((x << 12))

struct LN_ADC_DESCR
{
    int dmaEngine;
    int dmaChannel;
    LN_ADC_Registers *registers;
};

extern LN_ADC_Registers *aadc0;
extern LN_ADC_Registers *aadc1;

extern const LN_ADC_DESCR lnAdcDesc[];

// EOF