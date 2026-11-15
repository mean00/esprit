/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */
#pragma once
#pragma once
#include "lnIRQ.h"
#include "lnPeripheral_priv.h"

// Each timer has channel 0,1,2,3

/**
 */
struct LN_Timers_Registersx
{
    uint32_t CTL0;      // CR1 0
    uint32_t CTL1;      // 4
    uint32_t SMCFG;     // 8   Slave Mode
    uint32_t DMAINTEN;  // C   DMA update
    uint32_t INTF;      // 10
    uint32_t SWEV;      // EGR 14
    uint32_t CHCTLs[2]; // CCM1R 18/1C

    uint32_t CHCTL2;   // CCER 20
    uint32_t CNT;      // 24
    uint32_t PSC;      // 28
    uint32_t CAR;      // ARR 2C
    uint32_t FILLER0;  // 30
    uint32_t CHCVs[4]; // CCR1 34 38 3c 40
    uint32_t FILLER1;  // 44
    uint32_t DMACFG;   // 48
    uint32_t DMATB;    // 4C
};
/**
 */
typedef volatile LN_Timers_Registersx LN_Timers_Registers;
/**
 */

//-------------- CTL0 ---------

#define LN_TIMER_CTL0_CEN (1 << 0)   // counter enable
#define LN_TIMER_CTL0_UPDIS (1 << 1) // update disable
#define LN_TIMER_CTL0_UPS (1 << 2)   // update source
#define LN_TIMER_CTL0_SPM (1 << 3)   // single pulse mode
#define LN_TIMER_CTL0_DIR (1 << 4)   // DIR 0=UP, 1=DOWN
#define LN_TIMER_CTL0_CAM_SHIFT (5)  // counter align mode
#define LN_TIMER_CTL0_ARSE (1 << 7)  // auto reload shadow
#define LN_TIMER_CTL0_CKDIV_SHIFT (8)

//-------------- CTL1 ---------

#define LN_TIMER_CTL1_DMAS (1 << 3) // DMA request source
#define LN_TIMER_CTL1_MMC_SHIFT (4) // Master mode control
#define LN_TIMER_CTL1_TI0S (1 << 7) // Channel0 trigger  input selection

//-------------- SMFG ---------
#define LN_TIMER_SMFG_SMC_SHIFT (0)    // Slave mode control
#define LN_TIMER_SMFG_TRGS_SHIFT (4)   // Trigger selection
#define LN_TIMER_SMFG_MSM (1 << 7)     // Master/slave mode
#define LN_TIMER_SMFG_ETFC_SHIFT (8)   // External trigger filter control
#define LN_TIMER_SMFG_ETPSC_SHIFT (12) //  External trigger prescaler
#define LN_TIMER_SMFG_SMC1 (1 << 14)   // SMC external clock 1
#define LN_TIMER_SMFG_ETP (1 << 15)    // External trigger polarity

//-------------- DMAINTEN ---------
#define LN_TIMER_DMAINTEN_UPIE (1 << (0))                      // Update interrupt enable
#define LN_TIMER_DMAINTEN_CHxIE(channel) (1 << (1 + channel))  // Capture/compare interrupt enable
#define LN_TIMER_DMAINTEN_TRGIE (1 << 6)                       // Trigger interrupt enable
#define LN_TIMER_DMAINTEN_UPDEN (1 << 8)                       // UPdate DMA request enable
#define LN_TIMER_DMAINTEN_CHxDEN(channel) (1 << (9 + channel)) // Channel x capture/compare dma request enable
#define LN_TIMER_DMAINTEN_TRGDEN (1 << 14)                     // Trigger dma request enable

//-------------- INTF ---------
#define LN_TIMER_INTF_UPIF (1 << (0))                     // Update interrupt flag
#define LN_TIMER_INTF_CHxIF(channel) (1 << (1 + channel)) // Capture/compare interrupt enable
#define LN_TIMER_INTF_CHxOF(channel) (1 << (9 + channel)) // Over capture flags
#define LN_TIMER_INTF_TRGIF (1 << (6))                    // Trigger interrupt flag

//-------------- SWEVG ---------
#define LN_TIMER_SWEVG_UPG (1 << (6))                     // Generate update event
#define LN_TIMER_SWEVG_CHxG(channel) (1 << (1 + channel)) // Channel capture/compare event generation
//-------------- CHTL0/1 ---------
// /!\ these are unshifted value

#define LN_TIME_CHCTL0_MS_OUPUT (0)
#define LN_TIME_CHCTL0_MS_MASK (~(3))

#define LN_TIME_CHCTL0_EN (1 << 2)
#define LN_TIME_CHCTL0_SEN (1 << 3)
#define LN_TIME_CHCTL0_CEN (1 << 7)

#define LN_CHCTL_MODE(x) ((x) << 4)

#define LN_TIME_CHCTL0_CTL_MASK (~(7 << 4))
#define LN_TIME_CHCTL0_CTL_TIMING LN_CHCTL_MODE(0)
#define LN_TIME_CHCTL0_CTL_SET LN_CHCTL_MODE(1)
#define LN_TIME_CHCTL0_CTL_CLEAR LN_CHCTL_MODE(2)
#define LN_TIME_CHCTL0_CTL_TOGGLE LN_CHCTL_MODE(3)
#define LN_TIME_CHCTL0_CTL_FORCE_LOW LN_CHCTL_MODE(4)
#define LN_TIME_CHCTL0_CTL_FORCE_HIGH LN_CHCTL_MODE(5)
#define LN_TIME_CHCTL0_CTL_PWM0 LN_CHCTL_MODE(6)
#define LN_TIME_CHCTL0_CTL_PWM1 LN_CHCTL_MODE(7)

//-------------- CHTL2 ---------
#define LN_TIMER_CHTL2_CHxEN(channel) (1 << (0 + channel * 4)) // Channel x captue/compare enable
#define LN_TIMER_CHTL2_CHxP(channel) (1 << (1 + channel * 4))  // Channel x captue/compare polarity

// EOF
