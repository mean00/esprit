/********************************** (C) COPYRIGHT
 ******************************** File Name          : ch32v30x_eth.c Author :
 *WCH Version            : V1.0.0 Date               : 2021/06/06 Description :
 *This file provides all the ETH firmware functions. Copyright (c) 2021 Nanjing
 *Qinheng Microelectronics Co., Ltd. SPDX-License-Identifier: Apache-2.0
 *******************************************************************************/
#include "ch32v3xx_eth_private.h"
//
#include "ch32v30x_eth.h"
//
#include "esprit.h"
//
ETH_DMADESCTypeDef *DMAPTPTxDescToSet;
ETH_DMADESCTypeDef *DMAPTPRxDescToGet;

/*********************************************************************
 * @fn      ETH_EnablePTPTimeStampAddend
 *
 * @brief   Updated the PTP block for fine correction with the Time Stamp Addend
 * register value.
 *
 * @return  none
 */
void ETH_EnablePTPTimeStampAddend(void)
{
    ETHPTP->PTPTSCR |= ETH_PTPTSCR_TSARU;
}

/*********************************************************************
 * @fn      ETH_EnablePTPTimeStampInterruptTrigger
 *
 * @brief   Enable the PTP Time Stamp interrupt trigger
 *
 * @return  none
 */
void ETH_EnablePTPTimeStampInterruptTrigger(void)
{
    ETHPTP->PTPTSCR |= ETH_PTPTSCR_TSITE;
}

/*********************************************************************
 * @fn      ETH_EnablePTPTimeStampUpdate
 *
 * @brief   Updated the PTP system time with the Time Stamp Update register
 * value.
 *
 * @return  none
 */
void ETH_EnablePTPTimeStampUpdate(void)
{
    ETHPTP->PTPTSCR |= ETH_PTPTSCR_TSSTU;
}

/*********************************************************************
 * @fn      ETH_InitializePTPTimeStamp
 *
 * @brief   Initialize the PTP Time Stamp.
 *
 * @return  none
 */
void ETH_InitializePTPTimeStamp(void)
{
    ETHPTP->PTPTSCR |= ETH_PTPTSCR_TSSTI;
}

/*********************************************************************
 * @fn      ETH_PTPUpdateMethodConfig
 *
 * @brief   Selects the PTP Update method.
 *
 * @param   UpdateMethod - the PTP Update method.
 *
 * @return  none
 */
void ETH_PTPUpdateMethodConfig(uint32_t UpdateMethod)
{
    if (UpdateMethod != ETH_PTP_CoarseUpdate)
    {
        ETHPTP->PTPTSCR |= ETH_PTPTSCR_TSFCU;
    }
    else
    {
        ETHPTP->PTPTSCR &= (~(uint32_t)ETH_PTPTSCR_TSFCU);
    }
}

/*********************************************************************
 * @fn      ETH_PTPTimeStampCmd
 *
 * @brief   Enables or disables the PTP time stamp for transmit and receive
 * frames.
 *
 * @param   NewState - new state of the PTP time stamp for transmit and receive
 * frames.
 *
 * @return  none
 */
void ETH_PTPTimeStampCmd(bool NewState)
{
    if (NewState != false)
    {
        ETHPTP->PTPTSCR |= ETH_PTPTSCR_TSE;
    }
    else
    {
        ETHPTP->PTPTSCR &= (~(uint32_t)ETH_PTPTSCR_TSE);
    }
}

/*********************************************************************
 * @fn      ETH_GetPTPbool
 *
 * @brief   Checks whether the specified ETHERNET PTP flag is set or not.
 *
 * @param   The new state of ETHERNET PTP Flag (true or false).
 *
 * @return  none
 */
bool ETH_GetPTPbool(uint32_t ETH_PTP_FLAG)
{
    bool bitstatus = false;

    if ((ETHPTP->PTPTSCR & ETH_PTP_FLAG) != (uint32_t)false)
    {
        bitstatus = true;
    }
    else
    {
        bitstatus = false;
    }
    return bitstatus;
}

/*********************************************************************
 * @fn      ETH_SetPTPSubSecondIncrement
 *
 * @brief   Sets the system time Sub-Second Increment value.
 *
 * @param   SubSecondValue - specifies the PTP Sub-Second Increment Register
 * value.
 *
 * @return  none
 */
void ETH_SetPTPSubSecondIncrement(uint32_t SubSecondValue)
{
    ETHPTP->PTPSSIR = SubSecondValue;
}

/*********************************************************************
 * @fn      ETH_SetPTPTimeStampUpdate
 *
 * @brief   Sets the Time Stamp update sign and values.
 *
 * @param   Sign - specifies the PTP Time update value sign.
 *          SecondValue - specifies the PTP Time update second value.
 *          SubSecondValue - specifies the PTP Time update sub-second value.
 *
 * @return  none
 */
void ETH_SetPTPTimeStampUpdate(uint32_t Sign, uint32_t SecondValue, uint32_t SubSecondValue)
{
    ETHPTP->PTPTSHUR = SecondValue;
    ETHPTP->PTPTSLUR = Sign | SubSecondValue;
}

/*********************************************************************
 * @fn      ETH_SetPTPTimeStampAddend
 *
 * @brief   Sets the Time Stamp Addend value.
 *
 * @param   Value - specifies the PTP Time Stamp Addend Register value.
 *
 * @return  none
 */
void ETH_SetPTPTimeStampAddend(uint32_t Value)
{
    /* Set the PTP Time Stamp Addend Register */
    ETHPTP->PTPTSAR = Value;
}

/*********************************************************************
 * @fn      ETH_SetPTPTargetTime
 *
 * @brief   Sets the Target Time registers values.
 *
 * @param   HighValue - specifies the PTP Target Time High Register value.
 *          LowValue - specifies the PTP Target Time Low Register value.
 *
 * @return  none
 */
void ETH_SetPTPTargetTime(uint32_t HighValue, uint32_t LowValue)
{
    ETHPTP->PTPTTHR = HighValue;
    ETHPTP->PTPTTLR = LowValue;
}

/*********************************************************************
 * @fn      ETH_GetPTPRegister
 *
 * @brief   Get the specified ETHERNET PTP register value.
 *
 * @param   ETH_PTPReg - specifies the ETHERNET PTP register.
 *            ETH_PTPTSCR - Sub-Second Increment Register
 *            ETH_PTPSSIR - Sub-Second Increment Register
 *            ETH_PTPTSHR - Time Stamp High Register
 *            ETH_PTPTSLR - Time Stamp Low Register
 *            ETH_PTPTSHUR - Time Stamp High Update Register
 *            ETH_PTPTSLUR - Time Stamp Low Update Register
 *            ETH_PTPTSAR - Time Stamp Addend Register
 *            ETH_PTPTTHR - Target Time High Register
 *            ETH_PTPTTLR - Target Time Low Register
 *
 * @return  The value of ETHERNET PTP Register value.
 */
uint32_t ETH_GetPTPRegister(uint32_t ETH_PTPReg)
{
    return (*(__IO uint32_t *)(ETH_MAC_BASE + ETH_PTPReg));
}
/*********************************************************************
 * @fn      ETH_DMAPTPTxDescChainInit
 *
 * @brief   Initializes the DMA Tx descriptors in chain mode with PTP.
 *
 * @param   DMATxDescTab - Pointer on the first Tx desc list.
 *          DMAPTPTxDescTab - Pointer on the first PTP Tx desc list.
 *          TxBuff - Pointer on the first TxBuffer list.
 *          TxBuffCount - Number of the used Tx desc in the list.
 *
 * @return  none.
 */
void ETH_DMAPTPTxDescChainInit(ETH_DMADESCTypeDef *DMATxDescTab, ETH_DMADESCTypeDef *DMAPTPTxDescTab, uint8_t *TxBuff,
                               uint32_t TxBuffCount)
{
    uint32_t i = 0;
    ETH_DMADESCTypeDef *DMATxDesc;

    DMATxDescToSet = DMATxDescTab;
    DMAPTPTxDescToSet = DMAPTPTxDescTab;

    for (i = 0; i < TxBuffCount; i++)
    {
        DMATxDesc = DMATxDescTab + i;
        DMATxDesc->Status = ETH_DMATxDesc_TCH | ETH_DMATxDesc_TTSE;
        DMATxDesc->Buffer1Addr = (uint32_t)(&TxBuff[i * ETH_MAX_PACKET_SIZE]);

        if (i < (TxBuffCount - 1))
        {
            DMATxDesc->Buffer2NextDescAddr = (uint32_t)(DMATxDescTab + i + 1);
        }
        else
        {
            DMATxDesc->Buffer2NextDescAddr = (uint32_t)DMATxDescTab;
        }

        (&DMAPTPTxDescTab[i])->Buffer1Addr = DMATxDesc->Buffer1Addr;
        (&DMAPTPTxDescTab[i])->Buffer2NextDescAddr = DMATxDesc->Buffer2NextDescAddr;
    }

    (&DMAPTPTxDescTab[i - 1])->Status = (uint32_t)DMAPTPTxDescTab;

    ETHDMA->DMATDLAR = (uint32_t)DMATxDescTab;
}

/*********************************************************************
 * @fn      ETH_DMAPTPRxDescChainInit
 *
 * @brief   Initializes the DMA Rx descriptors in chain mode.
 *
 * @param   DMARxDescTab - Pointer on the first Rx desc list.
 *          DMAPTPRxDescTab - Pointer on the first PTP Rx desc list.
 *          RxBuff - Pointer on the first RxBuffer list.
 *          RxBuffCount - Number of the used Rx desc in the list.
 *
 * @return  none.
 */
void ETH_DMAPTPRxDescChainInit(ETH_DMADESCTypeDef *DMARxDescTab, ETH_DMADESCTypeDef *DMAPTPRxDescTab, uint8_t *RxBuff,
                               uint32_t RxBuffCount)
{
    uint32_t i = 0;
    ETH_DMADESCTypeDef *DMARxDesc;

    DMARxDescToGet = DMARxDescTab;
    DMAPTPRxDescToGet = DMAPTPRxDescTab;

    for (i = 0; i < RxBuffCount; i++)
    {
        DMARxDesc = DMARxDescTab + i;
        DMARxDesc->Status = ETH_DMARxDesc_OWN;
        DMARxDesc->ControlBufferSize = ETH_DMARxDesc_RCH | (uint32_t)ETH_MAX_PACKET_SIZE;
        DMARxDesc->Buffer1Addr = (uint32_t)(&RxBuff[i * ETH_MAX_PACKET_SIZE]);

        if (i < (RxBuffCount - 1))
        {
            DMARxDesc->Buffer2NextDescAddr = (uint32_t)(DMARxDescTab + i + 1);
        }
        else
        {
            DMARxDesc->Buffer2NextDescAddr = (uint32_t)(DMARxDescTab);
        }

        (&DMAPTPRxDescTab[i])->Buffer1Addr = DMARxDesc->Buffer1Addr;
        (&DMAPTPRxDescTab[i])->Buffer2NextDescAddr = DMARxDesc->Buffer2NextDescAddr;
    }

    (&DMAPTPRxDescTab[i - 1])->Status = (uint32_t)DMAPTPRxDescTab;
    ETHDMA->DMARDLAR = (uint32_t)DMARxDescTab;
}

/*********************************************************************
 * @fn      ETH_HandlePTPTxPkt
 *
 * @brief   Transmits a packet, from application buffer, pointed by ppkt with
 * Time Stamp values.
 *
 * @param   ppkt - pointer to application packet buffer to transmit.
 *          FrameLength - Tx Packet size.
 *          PTPTxTab - Pointer on the first PTP Tx table to store Time stamp
 * values.
 *
 * @return  none.
 */
bool ETH_HandlePTPTxPkt(uint8_t *ppkt, uint16_t FrameLength, uint32_t *PTPTxTab)
{
    uint32_t offset = 0, timeout = 0;

    if ((DMATxDescToSet->Status & ETH_DMATxDesc_OWN) != (uint32_t)false)
    {
        return false;
    }

    for (offset = 0; offset < FrameLength; offset++)
    {
        (*(__IO uint8_t *)((DMAPTPTxDescToSet->Buffer1Addr) + offset)) = (*(ppkt + offset));
    }

    DMATxDescToSet->ControlBufferSize = (FrameLength & (uint32_t)0x1FFF);
    DMATxDescToSet->Status |= ETH_DMATxDesc_LS | ETH_DMATxDesc_FS;
    DMATxDescToSet->Status |= ETH_DMATxDesc_OWN;

    if ((ETHDMA->DMASR & ETH_DMASR_TBUS) != (uint32_t)false)
    {
        ETHDMA->DMASR = ETH_DMASR_TBUS;
        ETHDMA->DMATPDR = 0;
    }

    do
    {
        timeout++;
    } while (!(DMATxDescToSet->Status & ETH_DMATxDesc_TTSS) && (timeout < 0xFFFF));

    if (timeout == PHY_READ_TO)
    {
        return false;
    }

    DMATxDescToSet->Status &= ~ETH_DMATxDesc_TTSS;
    *PTPTxTab++ = DMATxDescToSet->Buffer1Addr;
    *PTPTxTab = DMATxDescToSet->Buffer2NextDescAddr;

    if ((DMATxDescToSet->Status & ETH_DMATxDesc_TCH) != (uint32_t)false)
    {
        DMATxDescToSet = (ETH_DMADESCTypeDef *)(DMAPTPTxDescToSet->Buffer2NextDescAddr);
        if (DMAPTPTxDescToSet->Status != 0)
        {
            DMAPTPTxDescToSet = (ETH_DMADESCTypeDef *)(DMAPTPTxDescToSet->Status);
        }
        else
        {
            DMAPTPTxDescToSet++;
        }
    }
    else
    {
        if ((DMATxDescToSet->Status & ETH_DMATxDesc_TER) != (uint32_t)false)
        {
            DMATxDescToSet = (ETH_DMADESCTypeDef *)(ETHDMA->DMATDLAR);
            DMAPTPTxDescToSet = (ETH_DMADESCTypeDef *)(ETHDMA->DMATDLAR);
        }
        else
        {
            DMATxDescToSet =
                (ETH_DMADESCTypeDef *)((uint32_t)DMATxDescToSet + 0x10 + ((ETHDMA->DMABMR & ETH_DMABMR_DSL) >> 2));
            DMAPTPTxDescToSet =
                (ETH_DMADESCTypeDef *)((uint32_t)DMAPTPTxDescToSet + 0x10 + ((ETHDMA->DMABMR & ETH_DMABMR_DSL) >> 2));
        }
    }

    return true;
}

/*********************************************************************
 * @fn      ETH_HandlePTPRxPkt
 *
 * @brief   Receives a packet and copies it to memory pointed by ppkt with Time
 * Stamp values.
 *
 * @param   ppkt - pointer to application packet receive buffer.
 *          PTPRxTab - Pointer on the first PTP Rx table to store Time stamp
 * values.
 *
 * @return  false - if there is error in reception.
 *          framelength - received packet size if packet reception is correct.
 */
uint32_t ETH_HandlePTPRxPkt(uint8_t *ppkt, uint32_t *PTPRxTab)
{
    uint32_t offset = 0, framelength = 0;

    if ((DMARxDescToGet->Status & ETH_DMARxDesc_OWN) != (uint32_t)false)
    {
        return false;
    }
    if (((DMARxDescToGet->Status & ETH_DMARxDesc_ES) == (uint32_t)false) &&
        ((DMARxDescToGet->Status & ETH_DMARxDesc_LS) != (uint32_t)false) &&
        ((DMARxDescToGet->Status & ETH_DMARxDesc_FS) != (uint32_t)false))
    {
        framelength = ((DMARxDescToGet->Status & ETH_DMARxDesc_FL) >> ETH_DMARXDESC_FRAME_LENGTHSHIFT) - 4;

        for (offset = 0; offset < framelength; offset++)
        {
            (*(ppkt + offset)) = (*(__IO uint8_t *)((DMAPTPRxDescToGet->Buffer1Addr) + offset));
        }
    }
    else
    {
        framelength = false;
    }

    if ((ETHDMA->DMASR & ETH_DMASR_RBUS) != (uint32_t)false)
    {
        ETHDMA->DMASR = ETH_DMASR_RBUS;
        ETHDMA->DMARPDR = 0;
    }

    *PTPRxTab++ = DMARxDescToGet->Buffer1Addr;
    *PTPRxTab = DMARxDescToGet->Buffer2NextDescAddr;
    DMARxDescToGet->Status |= ETH_DMARxDesc_OWN;

    if ((DMARxDescToGet->ControlBufferSize & ETH_DMARxDesc_RCH) != (uint32_t)false)
    {
        DMARxDescToGet = (ETH_DMADESCTypeDef *)(DMAPTPRxDescToGet->Buffer2NextDescAddr);
        if (DMAPTPRxDescToGet->Status != 0)
        {
            DMAPTPRxDescToGet = (ETH_DMADESCTypeDef *)(DMAPTPRxDescToGet->Status);
        }
        else
        {
            DMAPTPRxDescToGet++;
        }
    }
    else
    {
        if ((DMARxDescToGet->ControlBufferSize & ETH_DMARxDesc_RER) != (uint32_t)false)
        {
            DMARxDescToGet = (ETH_DMADESCTypeDef *)(ETHDMA->DMARDLAR);
        }
        else
        {
            DMARxDescToGet =
                (ETH_DMADESCTypeDef *)((uint32_t)DMARxDescToGet + 0x10 + ((ETHDMA->DMABMR & ETH_DMABMR_DSL) >> 2));
        }
    }

    return (framelength);
}
// EOF
