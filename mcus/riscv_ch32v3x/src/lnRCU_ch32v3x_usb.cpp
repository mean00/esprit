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
//
#include "ch32vxx_priv.h"
extern LN_RCU *arcu;
//
#define CH32V3_USBHS_PLL_SRC_HSI (0 << 27)
#define CH32V3_USBHS_PLL_SRC_HSE (1 << 27) // the datasheet is wrong
#define CH32V3_USBHS_PREFIX(x) ((x) << 24)
#define CH32V3_USBHS_CKREF_SEL(x) ((x) << 28)
#define CH32V3_USBHS_CKREF_SRC_USB_PHY (1 << 31)
#define CH32V3_USBHS_CKREF_SRC_PLL (0 << 31)
#define CH32V3_USBHS_USB_PHY_ENABLED (1 << 30)
#define CH32V3_USBHS_USB_PHY_DISABLED (0 << 30)
//
#define CH32_EXTEN_CTR_ADR 0x40023800UL
#define CH32_EXTEN_CTR2_ADR 0x40023808UL
//
#define CH32_EXTEN_CTR_USBD_LOWSPEED (1 << 0)      // 0 full speed, 1 low speed
#define CH32_EXTEN_CTR_USBD_PULLUP_ENABLE (1 << 1) //
#define CH32_EXTEN_CTR_ETH_10M_ENABLE (1 << 2)     //
//
volatile uint32_t *ch32v_exten = (volatile uint32_t *)CH32_EXTEN_CTR_ADR;

/**
    \brief This is the USB HS clock for CH32V3x chip
    NB: The CFG1 register is called RCC_CFGR2 in the datasheet
*/
void lnPeripherals::enableUsbHS48Mhz_ch32v3x()
{
    uint32_t cfgr2 = arcu->CFG1;
    cfgr2 &= 0xffffff;
    cfgr2 |= CH32V3_USBHS_PLL_SRC_HSE;       // src = HSE
    cfgr2 |= CH32V3_USBHS_PREFIX(1);         // 0= no div, 1=div by 2
    cfgr2 |= CH32V3_USBHS_CKREF_SEL(1);      // Ref= 4Mhz
    cfgr2 |= CH32V3_USBHS_CKREF_SRC_USB_PHY; // Src=USB PHY
    cfgr2 |= CH32V3_USBHS_USB_PHY_ENABLED;   // USB PHY PLL ALive
    arcu->CFG1 = cfgr2;
}
/**
 * @brief enable usb internal pullup/down
 */
void lnCh32_enableUsbPullUp()
{
    *ch32v_exten |= CH32_EXTEN_CTR_USBD_PULLUP_ENABLE;
}
/**
 * @brief  enable internal 10M ethernet phy
 */
void lnCh32_enableInternalEth()
{
    *ch32v_exten |= CH32_EXTEN_CTR_ETH_10M_ENABLE;
}

// EOF
