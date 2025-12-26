/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */

#pragma once

struct LN_RCUx
{
    uint32_t CTL;     // 00 CR
    uint32_t CFG0;    // 04 CFGR
    uint32_t INT;     // 08 CIR
    uint32_t APB2RST; // 0c APB2RSTR
    uint32_t APB1RST; // 10 APB1RSTR
    uint32_t AHBEN;   // 14 AHBENR
    uint32_t APB2EN;  // 18 APB2ENR
    uint32_t APB1EN;  // 1c APB1ENR
    uint32_t BDCTL;   // 20 BDCR
    uint32_t RSTCLK;  // 24 CSR
    uint32_t AHBRST;  // 28 AHBRSTR
    uint32_t CFG1;    // 2c CFGR2
    uint32_t DSV;     // 30 N/A
};
typedef volatile LN_RCUx LN_RCU;

// CTL
#define LN_RCU_CTL_IRC8MEN (1 << 0)
#define LN_RCU_CTL_IRC8MSTB (1 << 1)
#define LN_RCU_CTL_HXTALEN (1 << 16) // enable high speed external crystal (8Mhz)
#define LN_RCU_CTL_HXTASTB (1 << 17)
#define LN_RCU_CTL_HXTABPS (1 << 18)
#define LN_RCU_CTL_CKMEN (1 << 19)
#define LN_RCU_CTL_PLLEN (1 << 24)
#define LN_RCU_CTL_PLLSTB (1 << 25)
#define LN_RCU_CTL_PLL1EN (1 << 26)
#define LN_RCU_CTL_PLL1STB (1 << 27)
#define LN_RCU_CTL_PLL2EN (1 << 28)
#define LN_RCU_CTL_PLL2STB (1 << 29)

// APB1 EN // APB1 RESET
#define LN_RCU_APB1_TIMER1EN (1 << 0)
#define LN_RCU_APB1_TIMER2EN (1 << 1)
#define LN_RCU_APB1_TIMER3EN (1 << 2)
#define LN_RCU_APB1_TIMER4EN (1 << 3)
#define LN_RCU_APB1_TIMER5EN (1 << 4)
#define LN_RCU_APB1_TIMER6EN (1 << 5)
#define LN_RCU_APB1_WWDGTEN (1 << 11)
#define LN_RCU_APB1_SPI1EN (1 << 14)
#define LN_RCU_APB1_SPI2EN (1 << 15)
#define LN_RCU_APB1_USART1EN (1 << 17)
#define LN_RCU_APB1_USART2EN (1 << 18)
#define LN_RCU_APB1_USART3EN (1 << 19)
#define LN_RCU_APB1_USART4EN (1 << 20)
#define LN_RCU_APB1_I2C0EN (1 << 21)
#define LN_RCU_APB1_I2C1EN (1 << 22)
#define LN_RCU_APB1_USBDEN (1 << 23)
#define LN_RCU_APB1_CAN0EN (1 << 25)
#define LN_RCU_APB1_CAN1EN (1 << 26)
#define LN_RCU_APB1_BKPIEN (1 << 27)
#define LN_RCU_APB1_PMUEN (1 << 28)
#define LN_RCU_APB1_DACEN (1 << 29)

// APB2 EN // APB2 RST

#define LN_RCU_APB2_AFEN (1 << 0)
#define LN_RCU_APB2_PAEN (1 << 2)
#define LN_RCU_APB2_PBEN (1 << 3)
#define LN_RCU_APB2_PCEN (1 << 4)
#define LN_RCU_APB2_PDEN (1 << 5)
#define LN_RCU_APB2_PEEN (1 << 6)

#define LN_RCU_APB2_ADC0EN (1 << 9)
#define LN_RCU_APB2_ADC1EN (1 << 10)
#define LN_RCU_APB2_TIMER0EN (1 << 11)
#define LN_RCU_APB2_SPI0EN (1 << 12)
#define LN_RCU_APB2_USART0EN (1 << 14)

// AHBEN

#define LN_RCU_AHB_DMA0EN (1 << 0)
#define LN_RCU_AHB_DMA1EN (1 << 1)
#define LN_RCU_AHB_SRAMSP (1 << 2)
#define LN_RCU_AHB_FMCSPEN (1 << 4)
#define LN_RCU_AHB_CRCEN (1 << 6)
#define LN_RCU_AHB_EXMCEN (1 << 8)
#define LN_RCU_AHB_RNGEN_CH32V3x (1 << 9)
#define LN_RCU_AHB_SDIOEN_CH32V3x (1 << 10)
#define LN_RCU_AHB_USBHSEN_CH32V3x (1 << 11)
#define LN_RCU_AHB_USBFSEN_OTG_CH32V3x (1 << 12)
#define LN_RCU_AHB_USBFSEN (1 << 12)
#define LN_RCU_AHB_ETHMAC (7 << 14) // Rx + Tx /Enable

// AHB RESET
#define LN_RCU_AHBRST_USBFSRST (1 << 12)
#define LN_RCU_AHBRST_ETHMACRST (1 << 14)

// CFG0
#define LN_RCU_CFG0_PLL_USED (1 << 3)
#define LN_RCU_CFG0_SYSCLOCK_MASK (3)
#define LN_RCU_CFG0_SYSCLOCK_PLL (1 << 1)
#define LN_RCU_CFG0_SYSCLOCK_IRC8 (0)
#define LN_RCU_CFG0_HPRE_WCH(x) ((x) << 4)
#define LN_RCU_CFG0_PLLSEL (1 << 16)
#define LN_RCU_CFG0_PREDIV (1 << 17)

#define LN_RCU_CFG0_USBPSC_MASK (~(3 << 22))
#define LN_RCU_CFG0_USBPSC(x) ((x & 3) << 22)

#define LN_RCU_ADC_PRESCALER_MASK (~((1 << 28) + (3 << 14)))
#define LN_RCU_ADC_PRESCALER_HIGHBIT (1 << 28)
#define LN_RCU_ADC_PRESCALER_LOWBIT(x) ((x & 3) << 14)

#define LN_RCU_CFG0_ETH_PRESCALER_WCH(x) ((x << 28))
#define LN_RCU_CFG0_MCO_SEL(x) ((x << 24))
#define LN_RCU_CFG0_MCO_SEL_MASK (~(LN_RCU_CFG0_MCO_SEL(0xf)))
// CFG1
#define LN_RCU_CFG1_PRED_CLOCKSEL (1 << 16)
#define LN_RCU_CFG1_PLL2_MUL(x) ((x) << 12)
#define LN_RCU_CFG1_PLL2_MUL_MASK (~(LN_RCU_CFG1_PLL2_MUL(0xf)))
#define LN_RCU_CFG1_PLL1_MUL(x) ((x) << 8)
#define LN_RCU_CFG1_PREDV1_DIV(x) ((x) << 4)
#define LN_RCU_CFG1_PREDV0_DIV(x) ((x) << 0)

// This is Gigadevice specific, at least on GD32F303
// It drives the internal 48Mhz clock that can be used
// to drive USB in crystal less setup
#define LN_GD_RCU_ADDCTL_IRC48M_EN (1 << 16)
#define LN_GD_RCU_ADDCTL_IRC48M_STB (1 << 17)
#define LN_GD_RCU_ADDCTL_IRC48M_SEL (1 << 0)

// EOF
