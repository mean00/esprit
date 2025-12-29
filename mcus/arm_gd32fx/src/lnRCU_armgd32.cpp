/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */

#include "esprit.h"
#include "lnCpuID.h"
#include "lnPeripheral_priv.h"
#include "lnRCU.h"
#include "lnRCU_priv.h"
//
extern LN_RCU *arcu;
/**
 * @brief
 *
 */
void lnPeripherals::enableUsb48Mhz()
{
    static bool usb48M = false;
    if (usb48M)
        return;
    usb48M = true;
    // This is a not well coded
    // In short, the GD32F303 has a internal 48M RC oscillator we can
    // use to drive USB
    // We assume that if we are running a GD32 chip it is a GD32F303

#if defined(LN_USE_INTERNAL_CLOCK)
    volatile uint32_t *addctl = (volatile uint32_t *)(LN_RCU_ADR + 0xc0);
    switch (lnCpuID::vendor())
    {
    case lnCpuID::LN_MCU_GD32:
        // activate 48M  internal RC
        *addctl |= LN_GD_RCU_ADDCTL_IRC48M_EN; // enable 48M
        while (1)
        {
            if (*addctl & LN_GD_RCU_ADDCTL_IRC48M_STB) // clock stabilized
            {
                break;
            }
        }
        // Use IRC48M as usb source
        *addctl |= LN_GD_RCU_ADDCTL_IRC48M_SEL;
        return;
        break;
    default:
        break;
    }
#endif

    // careful, the usb clock must be off !
    int scaler = (2 * lnPeripherals::getClock(pSYSCLOCK)) / 48000000;
    int x = 0;
    switch (lnCpuID::vendor())
    {
    case lnCpuID::LN_MCU_STM32:
        // STM32F1 chip only supports div by 1 and div by 1.5, i.e. x=0 or 1
        switch (scaler)
        {
        case 3:
            x = 0;
            break; // 3/2=1.5
        case 2:
            x = 1;
            break; // 2/2=1
        default:
            xAssert(0); // invalid sys clock
            break;
        }
        break;
        break;
        // only GD32 has more dividers
    case lnCpuID::LN_MCU_GD32:
        switch (scaler)
        {
        case 3:
            x = 0;
            break; // 3/2=1.5
        case 2:
            x = 1;
            break; // 2/2=1
        case 5:
            x = 2;
            break; // 5/2=2.5
        case 4:
            x = 3;
            break; // 4/2=2
        default:
            xAssert(0); // invalid sys clock
            break;
        }
        break;
    default:
        xAssert(0);
        break;
    }
    uint32_t cfg0 = arcu->CFG0;
    cfg0 &= LN_RCU_CFG0_USBPSC_MASK;
    cfg0 |= LN_RCU_CFG0_USBPSC(x);
    arcu->CFG0 = cfg0;
}
//
