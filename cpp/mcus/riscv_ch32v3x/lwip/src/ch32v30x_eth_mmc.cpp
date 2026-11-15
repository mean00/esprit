/********************************** (C) COPYRIGHT
 ******************************** File Name          : ch32v30x_eth.c Author :
 *WCH Version            : V1.0.0 Date               : 2021/06/06 Description :
 *This file provides all the ETH firmware functions. Copyright (c) 2021 Nanjing
 *Qinheng Microelectronics Co., Ltd. SPDX-License-Identifier: Apache-2.0
 *******************************************************************************/
#include "ch32v3xx_eth_private.h"
//
#include "ch32v30x_eth.h"

/*********************************************************************
 * @fn      ETH_MMCCounterFreezeCmd
 *
 * @brief   Enables or disables the MMC Counter Freeze.
 *
 * @param   NewState - new state of the MMC Counter Freeze.
 *
 * @return  none
 */
void ETH_MMCCounterFreezeCmd(bool NewState)
{
    if (NewState != false)
    {
        ETHMMC->MMCCR |= ETH_MMCCR_MCF;
    }
    else
    {
        ETHMMC->MMCCR &= ~ETH_MMCCR_MCF;
    }
}

/*********************************************************************
 * @fn      ETH_MMCResetOnReadCmd
 *
 * @brief   Enables or disables the MMC Reset On Read.
 *
 * @param   NewState - new state of the MMC Reset On Read.
 *
 * @return  none
 */
void ETH_MMCResetOnReadCmd(bool NewState)
{
    if (NewState != false)
    {
        ETHMMC->MMCCR |= ETH_MMCCR_ROR;
    }
    else
    {
        ETHMMC->MMCCR &= ~ETH_MMCCR_ROR;
    }
}

/*********************************************************************
 * @fn      ETH_MMCCounterRolloverCmd
 *
 * @brief   Enables or disables the MMC Counter Stop Rollover.
 *
 * @param   NewState - new state of the MMC Counter Stop Rollover.
 *
 * @return  none
 */
void ETH_MMCCounterRolloverCmd(bool NewState)
{
    if (NewState != false)
    {
        ETHMMC->MMCCR &= ~ETH_MMCCR_CSR;
    }
    else
    {
        ETHMMC->MMCCR |= ETH_MMCCR_CSR;
    }
}

/*********************************************************************
 * @fn      ETH_MMCCountersReset
 *
 * @brief   Resets the MMC Counters.
 *
 * @return  none
 */
void ETH_MMCCountersReset(void)
{
    ETHMMC->MMCCR |= ETH_MMCCR_CR;
}

/*********************************************************************
 * @fn      ETH_MMCITConfig
 *
 * @brief   Enables or disables the specified ETHERNET MMC interrupts.
 *
 * @param   ETH_MMC_IT - specifies the ETHERNET MMC interrupt.
 *            ETH_MMC_IT_TGF - When Tx good frame counter reaches half the
 * maximum value. ETH_MMC_IT_TGFMSC - When Tx good multi col counter reaches
 * half the maximum value. ETH_MMC_IT_TGFSC - When Tx good single col counter
 * reaches half the maximum value. ETH_MMC_IT_RGUF - When Rx good unicast frames
 * counter reaches half the maximum value. ETH_MMC_IT_RFAE - When Rx alignment
 * error counter reaches half the maximum value. ETH_MMC_IT_RFCE - When Rx crc
 * error counter reaches half the maximum value. NewState - new state of the
 * specified ETHERNET MMC interrupts.
 *
 * @return  none
 */
void ETH_MMCITConfig(uint32_t ETH_MMC_IT, bool NewState)
{
    if ((ETH_MMC_IT & (uint32_t)0x10000000) != (uint32_t)false)
    {
        ETH_MMC_IT &= 0xEFFFFFFF;

        if (NewState != false)
        {
            ETHMMC->MMCRIMR &= (~(uint32_t)ETH_MMC_IT);
        }
        else
        {
            ETHMMC->MMCRIMR |= ETH_MMC_IT;
        }
    }
    else
    {
        if (NewState != false)
        {
            ETHMMC->MMCTIMR &= (~(uint32_t)ETH_MMC_IT);
        }
        else
        {
            ETHMMC->MMCTIMR |= ETH_MMC_IT;
        }
    }
}

/*********************************************************************
 * @fn      ETH_GetMMCbool
 *
 * @brief   Checks whether the specified ETHERNET MMC IT is set or not.
 *
 * @param   ETH_MMC_IT - specifies the ETHERNET MMC interrupt.
 *            ETH_MMC_IT_TxFCGC - When Tx good frame counter reaches half the
 * maximum value. ETH_MMC_IT_TxMCGC - When Tx good multi col counter reaches
 * half the maximum value. ETH_MMC_IT_TxSCGC - When Tx good single col counter
 * reaches half the maximum value . ETH_MMC_IT_RxUGFC - When Rx good unicast
 * frames counter reaches half the maximum value. ETH_MMC_IT_RxAEC - When Rx
 * alignment error counter reaches half the maximum value. ETH_MMC_IT_RxCEC -
 * When Rx crc error counter reaches half the maximum value.
 *
 * @return  The value of ETHERNET MMC IT (true or false).
 */
bool ETH_GetMMCbool(uint32_t ETH_MMC_IT)
{
    bool bitstatus = false;

    if ((ETH_MMC_IT & (uint32_t)0x10000000) != (uint32_t)false)
    {
        if ((((ETHMMC->MMCRIR & ETH_MMC_IT) != (uint32_t)false)) && ((ETHMMC->MMCRIMR & ETH_MMC_IT) != (uint32_t)false))
        {
            bitstatus = true;
        }
        else
        {
            bitstatus = false;
        }
    }
    else
    {
        if ((((ETHMMC->MMCTIR & ETH_MMC_IT) != (uint32_t)false)) && ((ETHMMC->MMCRIMR & ETH_MMC_IT) != (uint32_t)false))
        {
            bitstatus = true;
        }
        else
        {
            bitstatus = false;
        }
    }

    return bitstatus;
}

/*********************************************************************
 * @fn      ETH_GetMMCRegister
 *
 * @brief   Get the specified ETHERNET MMC register value.
 *
 * @param   ETH_MMCReg - specifies the ETHERNET MMC register.
 *            ETH_MMCCR - MMC CR register
 *            ETH_MMCRIR - MMC RIR register
 *            ETH_MMCTIR - MMC TIR register
 *            ETH_MMCRIMR - MMC RIMR register
 *            ETH_MMCTIMR - MMC TIMR register
 *            ETH_MMCTGFSCCR - MMC TGFSCCR register
 *            ETH_MMCTGFMSCCR - MMC TGFMSCCR register
 *            ETH_MMCTGFCR - MMC TGFCR register
 *            ETH_MMCRFCECR - MMC RFCECR register
 *            ETH_MMCRFAECR - MMC RFAECR register
 *            ETH_MMCRGUFCR - MMC RGUFCRregister
 *
 * @return  The value of ETHERNET MMC Register value.
 */
uint32_t ETH_GetMMCRegister(uint32_t ETH_MMCReg)
{
    return (*(__IO uint32_t *)(ETH_MAC_BASE + ETH_MMCReg));
}
// EOF
