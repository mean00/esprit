#pragma once
#pragma once
#include "lnIRQ.h"
#include "lnPeripheral_priv.h"

// Each timer has channel 0,1,2,3

/**
 */
struct LN_BTimers_Registersx
{
    uint32_t CTL0;     // 0
    uint32_t CTL1;     // 4
    uint32_t DUMMY03;  // 8
    uint32_t DMAINTEN; // C
    uint32_t INTF;     // 10
    uint32_t SWEV;     // 14
    uint32_t DUMMY00;  // 18
    uint32_t DUMMY01;  // 1C
    uint32_t DUMMY02;  // 20
    uint32_t CNT;      // 24
    uint32_t PSC;      // 28
    uint32_t CAR;      // 2C
};
/**
 */
typedef volatile LN_BTimers_Registersx LN_BTimers_Registers;
/**
 */

//-------------- CTL0 ---------

#define LN_BTIMER_CTL0_EN (1 << 0)    // counter enable
#define LN_BTIMER_CTL0_UPDIS (1 << 1) // update disable
#define LN_BTIMER_CTL0_UPS (1 << 2)   // update source
#define LN_BTIMER_CTL0_SPM (1 << 3)   // single pulse mode
#define LN_BTIMER_CTL0_ARSE (1 << 7)  // auto reload shadow

//-------------- CTL1 ---------

#define LN_BTIMER_CTL1_MMC_SHIFT (4) // Master mode control
#define LN_BTIMER_CTL1_MMC_MASK (~(7 << 4))

//-------------- DMAINTEN ---------
#define LN_BTIMER_DMAINTEN_UPIE (1 << (0))  // Update interrupt enable
#define LN_BTIMER_DMAINTEN_UPDEN (1 << (8)) // Update dma request flag
//-------------- INTF ---------
#define LN_BTIMER_INTF_UPIF (1 << (0)) // Update interrupt flag

//-------------- SWEVG ---------
#define LN_BTIMER_SWEVG_UPG (1 << (0)) // Generate update event

//-------------- CNT ---------
//-------------- SWEVG ---------

// EOF
