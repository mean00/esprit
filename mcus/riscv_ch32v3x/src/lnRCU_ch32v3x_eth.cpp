/**
 * @file lnRCU_ch32v3x_usb.cpp
 * @author 2021/2024 MEAN00
 * @brief USB HS clock management
 *
 * @copyright Copyright (c) 2021/2024
 */
#include "esprit.h"
#include "lnRCU.h"
#include "lnRCU_priv.h"
extern LN_RCU *arcu;

/**
    \brief This is the CH32V208 setup with a 32 Mhz xtal
    See 3.3.1 p 31 fig 3-5 fig 27-8 p488
    internal phy = PLL2= 60 Mhz max (named pll3 in the doc)

  MCO= 25 Mhz ?

    32 Mhz => prediv 2 => 16 Mhz=> PLL MUL  15 SYSCLK

    MCO is the external ethenet clock

*/
void lnPeripherals::initEthClock()
{

    // RCC_PLL3Cmd(DISABLE);
    arcu->CTL &= ~LN_RCU_CTL_PLL2EN;
    RCC_PREDIV2Config(RCC_PREDIV2_Div2);
    //
    // RCC_PLL3Config(RCC_PLL3Mul_15);
    uint32_t cfgr1 = arcu->CFG1;
    cfgr1 &= LN_RCU_CFG1_PLL2_MUL_MASK;
    cfgr1 |= LN_RCU_CFG1_PLL2_MUL(13); // 13-> x15
    arcu->CFG1 = cfgr1;
    // RCC_MCOConfig(RCC_MCO_PLL3CLK);
    uint32_t cfgr0 = arcu->CFG0;
    cfgr0 &= LN_RCU_CFG0_MCO_SEL_MASK;
    cfgr0 |= LN_RCU_CFG0_MCO_SEL(LN_RCU_CFG0_MCO_SEL_PLL3);
    arcu->CFG0 = cfgr0;
    // RCC_PLL3Cmd(ENABLE);
    arcu->CTL |= LN_RCU_CTL_PLL2EN;

    RCC_AHBPeriphClockCmd(RCC_AHBPeriph_ETH_MAC | RCC_AHBPeriph_ETH_MAC_Tx | RCC_AHBPeriph_ETH_MAC_Rx, ENABLE);
    ETH_DeInit();

    ETH_SoftwareReset();

    uint32_t cfgr2 = arcu->CFG1;
    cfgr2 &= 0xffffff;
    cfgr2 |= CH32V3_USBHS_PLL_SRC_HSE;       // src = HSE
    cfgr2 |= CH32V3_USBHS_PREFIX(1);         // 0= no div, 1=div by 2
    cfgr2 |= CH32V3_USBHS_CKREF_SEL(1);      // Ref= 4Mhz
    cfgr2 |= CH32V3_USBHS_CKREF_SRC_USB_PHY; // Src=USB PHY
    cfgr2 |= CH32V3_USBHS_USB_PHY_ENABLED;   // USB PHY PLL ALive
    arcu->CFG1 = cfgr2;
}
// EOF
