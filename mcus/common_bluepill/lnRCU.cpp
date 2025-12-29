/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */

#include "lnRCU.h"
#include "esprit.h"
#include "lnCpuID.h"
#include "lnPeripheral_priv.h"
#include "lnRCU_priv.h"
//
LN_RCU *arcu = (LN_RCU *)LN_RCU_ADR;
/**
 */
struct RCU_Peripheral
{
    uint8_t periph;
    uint8_t AHB_APB; // 1=APB 1, 2=APB2,8=AHB
    uint32_t enable;
};

#define RCU_RESET 1
#define RCU_ENABLE 2
#define RCU_DISABLE 3

extern void lnCh32_enableUsbPullUp();
__attribute__((weak)) void lnCh32_enableUsbPullUp()
{
#warning FIXME BADLY
#warning FIXME BADLY
#warning FIXME BADLY
#warning FIXME BADLY
#warning FIXME BADLY
#warning FIXME BADLY
#warning FIXME BADLY
}
/**
 */
static const RCU_Peripheral _peripherals[] = {
    // PERIP          APB         BIT
    {pNONE, 0, 0},
    {pSPI0, 2, LN_RCU_APB2_SPI0EN},
    {pSPI1, 1, LN_RCU_APB1_SPI1EN},
    {pSPI2, 1, LN_RCU_APB1_SPI2EN},
    {pUART0, 2, LN_RCU_APB2_USART0EN},
    {pUART1, 1, LN_RCU_APB1_USART1EN},
    {pUART2, 1, LN_RCU_APB1_USART2EN},
    {pUART3, 1, LN_RCU_APB1_USART3EN},
    {pUART4, 1, LN_RCU_APB1_USART4EN},
    {pI2C0, 1, LN_RCU_APB1_I2C0EN},
    {pI2C1, 1, LN_RCU_APB1_I2C1EN},
    {pCAN0, 1, LN_RCU_APB1_CAN0EN},
    {pCAN1, 1, LN_RCU_APB1_CAN1EN},
    {pDAC, 1, LN_RCU_APB1_DACEN},
    {pPMU, 1, LN_RCU_APB1_PMUEN},
    {pBKPI, 1, LN_RCU_APB1_BKPIEN},
    {pWWDGT, 1, LN_RCU_APB1_WWDGTEN},
    {pTIMER0, 2, LN_RCU_APB2_TIMER0EN},
    {pTIMER1, 1, LN_RCU_APB1_TIMER1EN},
    {pTIMER2, 1, LN_RCU_APB1_TIMER2EN},
    {pTIMER3, 1, LN_RCU_APB1_TIMER3EN},
    {pTIMER4, 1, LN_RCU_APB1_TIMER4EN},
    {pTIMER5, 1, LN_RCU_APB1_TIMER5EN},
    {pTIMER6, 1, LN_RCU_APB1_TIMER6EN},
    {pUSB, 1, LN_RCU_APB1_USBDEN}, // not sure
                                   // PERIP        AHB/APB         APB         BIT
    {pADC0, 2, LN_RCU_APB2_ADC0EN},
    {pADC1, 2, LN_RCU_APB2_ADC1EN},

    {pGPIOA, 2, LN_RCU_APB2_PAEN},
    {pGPIOB, 2, LN_RCU_APB2_PBEN},
    {pGPIOC, 2, LN_RCU_APB2_PCEN},
    {pGPIOD, 2, LN_RCU_APB2_PDEN},
    {pGPIOE, 2, LN_RCU_APB2_PEEN},
    {pAF, 2, LN_RCU_APB2_AFEN},
    // PERIP        AHB/APB         APB         BIT
    {pDMA0, 8, LN_RCU_AHB_DMA0EN},
    {pDMA1, 8, LN_RCU_AHB_DMA1EN},
    {pETHERNET, 8, LN_RCU_AHB_ETHMAC},
    {pUSBHS_CH32v3x, 8, LN_RCU_AHB_USBHSEN_CH32V3x},
    {pUSBFS_OTG_CH32v3x, 8, LN_RCU_AHB_USBFSEN_OTG_CH32V3x},
};

// 1 : Reset
// 2: Enable
// 3: disable
static void _rcuAction(const Peripherals periph, int action)
{
    xAssert((int)periph < 100);
    RCU_Peripheral *o = (RCU_Peripheral *)(_peripherals + (int)periph);
    xAssert(o->periph == periph);
    xAssert(o->enable);
    switch (o->AHB_APB)
    {
    case 1: // APB1
        switch (action)
        {
        case RCU_RESET:
            arcu->APB1RST |= o->enable;
            arcu->APB1RST &= ~(o->enable); // not sure if it auto clears itself
            break;
        case RCU_ENABLE:
            arcu->APB1EN |= o->enable;
            break;
        case RCU_DISABLE:
            arcu->APB1EN &= ~o->enable;
            break;
        default:
            xAssert(0);
            break;
        }
        break;
    case 2: // APB2
        switch (action)
        {
        case RCU_RESET:
            arcu->APB2RST |= o->enable;
            arcu->APB2RST &= ~(o->enable); // not sure if it auto clears itself
            break;
        case RCU_ENABLE:
            arcu->APB2EN |= o->enable;
            break;
        case RCU_DISABLE:
            arcu->APB2EN &= ~o->enable;
            break;
        default:
            xAssert(0);
            break;
        }
        break;
    case 8: // AHB
        switch (action)
        {
        case RCU_RESET:
            if (periph == pETHERNET)
            {
                arcu->AHBRST |= LN_RCU_AHBRST_ETHMACRST;
                arcu->AHBRST &= ~LN_RCU_AHBRST_ETHMACRST;
            }
            // We can only reset USB
            //  if(periph==pUSB) xAssert(0);
            // else just ignore
            break;
        case RCU_ENABLE:
            arcu->AHBEN |= o->enable;
            break;
        case RCU_DISABLE:
            arcu->AHBEN &= ~o->enable;
            break;
        default:
            xAssert(0);
            break;
        }

        break;
    default:
        xAssert(0);
    }
}

/**
 *
 * @param periph
 */
void lnPeripherals::reset(const Peripherals periph)
{
    _rcuAction(periph, RCU_RESET);
}
/**
 *
 * @param periph
 */
void lnPeripherals::enable(const Peripherals periph)
{
    _rcuAction(periph, RCU_ENABLE);
}
/**
 *
 * @param periph
 */
void lnPeripherals::disable(const Peripherals periph)
{
    _rcuAction(periph, RCU_DISABLE);
}
/**
 *
 * @param divider
 */
void lnPeripherals::setAdcDivider(lnADC_DIVIDER divider)
{
    uint32_t val = arcu->CFG0;

    val &= LN_RCU_ADC_PRESCALER_MASK;
    int r = (int)divider;
    if (r & 4)
    {
        if (lnCpuID::vendor() == lnCpuID::LN_MCU_GD32) // only up to 8
        {
            val |= LN_RCU_ADC_PRESCALER_HIGHBIT;
            val |= LN_RCU_ADC_PRESCALER_LOWBIT(r & 3);
        }
        else
            val |= LN_RCU_ADC_PRESCALER_LOWBIT(lnADC_CLOCK_DIV_BY_8);
    }
    else
    {
        val |= LN_RCU_ADC_PRESCALER_LOWBIT(r);
    }
    arcu->CFG0 = val;
}
// EOF
