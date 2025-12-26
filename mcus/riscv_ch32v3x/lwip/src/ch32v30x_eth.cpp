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
#include "lwipopts.h"
//
#include "esprit.h"
//
extern void enable_eth_irq(void);
#define ETH_RXBUFNB 4
#define ETH_TXBUFNB 4
//
__attribute__((aligned(4))) ETH_DMADESCTypeDef DMARxDscrTab[ETH_RXBUFNB];      /* 接收描述符表 */
__attribute__((aligned(4))) ETH_DMADESCTypeDef DMATxDscrTab[ETH_TXBUFNB];      /* 发送描述符表 */
__attribute__((aligned(4))) uint8_t Rx_Buff[ETH_RXBUFNB][ETH_MAX_PACKET_SIZE]; /* 接收队列 */
__attribute__((aligned(4))) uint8_t Tx_Buff[ETH_TXBUFNB][ETH_MAX_PACKET_SIZE]; /* 发送队列 */
volatile ETHMAC_TypeDef *ETH = (ETHMAC_TypeDef *)ETH_MAC_BASE;
volatile ETHDMA_TypeDef *ETHDMA = (ETHDMA_TypeDef *)ETH_DMA_BASE;
extern ETH_DMADESCTypeDef *DMATxDescToSet;
extern ETH_DMADESCTypeDef *DMARxDescToGet;
/*********************************************************************
 * @fn      ETH_DMAITConfig
 *
 * @brief   Enables or disables the specified ETHERNET DMA interrupts.
 *
 * @param   ETH_DMA_IT - specifies the ETHERNET DMA interrupt sources to be
 * enabled or disabled. ETH_DMA_IT_NIS - Normal interrupt summary ETH_DMA_IT_AIS
 * - Abnormal interrupt summary ETH_DMA_IT_ER - Early receive interrupt
 *            ETH_DMA_IT_FBE - Fatal bus error interrupt
 *            ETH_DMA_IT_ET - Early transmit interrupt
 *            ETH_DMA_IT_RWT - Receive watchdog timeout interrupt
 *            ETH_DMA_IT_RPS - Receive process stopped interrupt
 *            ETH_DMA_IT_RBU - Receive buffer unavailable interrupt
 *            ETH_DMA_IT_R - Receive interrupt
 *            ETH_DMA_IT_TU - Underflow interrupt
 *            ETH_DMA_IT_RO - Overflow interrupt
 *            ETH_DMA_IT_TJT - Transmit jabber timeout interrupt
 *            ETH_DMA_IT_TBU - Transmit buffer unavailable interrupt
 *            ETH_DMA_IT_TPS - Transmit process stopped interrupt
 *            ETH_DMA_IT_T - Transmit interrupt
 *            ETH_DMA_Overflow_RxFIFOCounter - Overflow for FIFO Overflow
 * Counter ETH_DMA_Overflow_MissedFrameCounter - Overflow for Missed Frame
 * Counter NewState - new state of the specified ETHERNET DMA interrupts.
 *
 * @return  new state of the specified ETHERNET DMA interrupts.
 */
static void ETH_DMAITConfig(uint32_t ETH_DMA_IT, bool NewState)
{
    if (NewState != false)
    {
        ETHDMA->DMAIER |= ETH_DMA_IT;
    }
    else
    {
        ETHDMA->DMAIER &= (~(uint32_t)ETH_DMA_IT);
    }
}
/*********************************************************************
 * @fn      ETH_DMAReceptionCmd
 *
 * @brief   Enables or disables the DMA reception.
 *
 * @param   NewState - new state of the DMA reception.
 *
 * @return  none
 */
static void ETH_DMAReceptionCmd(bool NewState)
{
    if (NewState != false)
    {
        ETHDMA->DMAOMR |= ETH_DMAOMR_SR;
    }
    else
    {
        ETHDMA->DMAOMR &= ~ETH_DMAOMR_SR;
    }
}

/*********************************************************************
 * @fn      ETH_MACReceptionCmd
 *
 * @brief   Enables or disables the MAC reception.
 *
 * @param   NewState - new state of the MAC reception.
 *
 * @return  none
 */
static void ETH_MACReceptionCmd(bool NewState)
{
    if (NewState != false)
    {
        ETH->MACCR |= ETH_MACCR_RE;
    }
    else
    {
        ETH->MACCR &= ~ETH_MACCR_RE;
    }
}

/*********************************************************************
 * @fn      ETH_MACTransmissionCmd
 *
 * @brief   Enables or disables the MAC transmission.
 *
 * @param   NewState - new state of the MAC transmission.
 *
 * @return  none
 */
static void ETH_MACTransmissionCmd(bool NewState)
{
    if (NewState != false)
    {
        ETH->MACCR |= ETH_MACCR_TE;
    }
    else
    {
        ETH->MACCR &= ~ETH_MACCR_TE;
    }
}

/*********************************************************************
 * @fn      ETH_GetFlushTransmitFIFOStatus
 *
 * @brief   Checks whether the ETHERNET transmit FIFO bit is cleared or not.
 *
 * @return  The new state of ETHERNET flush transmit FIFO bit (true or false).
 */
static bool ETH_GetFlushTransmitFIFOStatus(void)
{
    bool bitstatus = false;
    if ((ETHDMA->DMAOMR & ETH_DMAOMR_FTF) != (uint32_t)false)
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
 * @fn      ETH_FlushTransmitFIFO
 *
 * @brief   Clears the ETHERNET transmit FIFO.
 *
 * @return  none
 */
static void ETH_FlushTransmitFIFO(void)
{
    ETHDMA->DMAOMR |= ETH_DMAOMR_FTF;
}

/*********************************************************************
 * @fn      ETH_DMATransmissionCmd
 *
 * @brief   Enables or disables the DMA transmission.
 *
 * @param   NewState - new state of the DMA transmission.
 *
 * @return  none
 */
static void ETH_DMATransmissionCmd(bool NewState)
{
    if (NewState != false)
    {
        ETHDMA->DMAOMR |= ETH_DMAOMR_ST;
    }
    else
    {
        ETHDMA->DMAOMR &= ~ETH_DMAOMR_ST;
    }
}

/*********************************************************************
 * @fn      ETH_DMARxDescChainInit
 *
 * @brief   Initializes the DMA Rx descriptors in chain mode.
 *
 * @param   DMARxDescTab - Pointer on the first Rx desc list.
 *          RxBuff - Pointer on the first RxBuffer list.
 *          RxBuffCount - Number of the used Rx desc in the list.
 *
 * @return  none
 */
static void ETH_DMARxDescChainInit(ETH_DMADESCTypeDef *DMARxDescTab, uint8_t *RxBuff, uint32_t RxBuffCount)
{
    uint32_t i = 0;
    ETH_DMADESCTypeDef *DMARxDesc;

    DMARxDescToGet = DMARxDescTab;

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
    }

    ETHDMA->DMARDLAR = (uint32_t)DMARxDescTab;
}

/*********************************************************************
 * @fn      ETH_DMATxDescChainInit
 *
 * @brief   Initializes the DMA Tx descriptors in chain mode.
 *
 * @param   DMATxDescTab - Pointer on the first Tx desc list
 *          TxBuff - Pointer on the first TxBuffer list
 *          TxBuffCount - Number of the used Tx desc in the list
 *
 * @return  none
 */
static void ETH_DMATxDescChainInit(ETH_DMADESCTypeDef *DMATxDescTab, uint8_t *TxBuff, uint32_t TxBuffCount)
{
    uint32_t i = 0;
    ETH_DMADESCTypeDef *DMATxDesc;

    DMATxDescToSet = DMATxDescTab;

    for (i = 0; i < TxBuffCount; i++)
    {
        DMATxDesc = DMATxDescTab + i;
        DMATxDesc->Status = ETH_DMATxDesc_TCH | ETH_DMATxDesc_IC;
        DMATxDesc->Buffer1Addr = (uint32_t)(&TxBuff[i * ETH_MAX_PACKET_SIZE]);

        if (i < (TxBuffCount - 1))
        {
            DMATxDesc->Buffer2NextDescAddr = (uint32_t)(DMATxDescTab + i + 1);
        }
        else
        {
            DMATxDesc->Buffer2NextDescAddr = (uint32_t)DMATxDescTab;
        }
    }

    ETHDMA->DMATDLAR = (uint32_t)DMATxDescTab;
}

/*********************************************************************
 * @fn      ETH_Start
 *
 * @brief   Enables ENET MAC and DMA reception/transmission.
 *
 * @return  none
 */
void ETH_Start(void)
{
    ETH_MACTransmissionCmd(true);
    ETH_FlushTransmitFIFO();
    ETH_MACReceptionCmd(true);
    ETH_DMATransmissionCmd(true);
    ETH_DMAReceptionCmd(true);
}

/*********************************************************************
 * @fn      ETH_DMAClearITPendingBit
 *
 * @brief   Clears the ETHERNET?s DMA IT pending bit.
 *
 * @param   ETH_DMA_IT - specifies the interrupt pending bit to clear.
 *            ETH_DMA_IT_NIS - Normal interrupt summary
 *            ETH_DMA_IT_AIS - Abnormal interrupt summary
 *            ETH_DMA_IT_ER - Early receive interrupt
 *            ETH_DMA_IT_FBE - Fatal bus error interrupt
 *            ETH_DMA_IT_ETI - Early transmit interrupt
 *            ETH_DMA_IT_RWT - Receive watchdog timeout interrupt
 *            ETH_DMA_IT_RPS - Receive process stopped interrupt
 *            ETH_DMA_IT_RBU - Receive buffer unavailable interrupt
 *            ETH_DMA_IT_R - Receive interrupt
 *            ETH_DMA_IT_TU - Transmit Underflow interrupt
 *            ETH_DMA_IT_RO - Receive Overflow interrupt
 *            ETH_DMA_IT_TJT - Transmit jabber timeout interrupt
 *            ETH_DMA_IT_TBU - Transmit buffer unavailable interrupt
 *            ETH_DMA_IT_TPS - Transmit process stopped interrupt
 *            ETH_DMA_IT_T - Transmit interrupt
 *
 * @return  none
 */
static void ETH_DMAClearITPendingBit(uint32_t ETH_DMA_IT)
{
    ETHDMA->DMASR = (uint32_t)ETH_DMA_IT;
}
/*********************************************************************
 * @fn      ETH_GetCurrentTxBufferAddress
 *
 * @brief   Get the ETHERNET DMA DMACHTBAR register value.
 *
 * @return  The value of the current Tx buffer address.
 */
uint32_t ETH_GetCurrentTxBufferAddress(void)
{
    return (DMATxDescToSet->Buffer1Addr);
}

/*********************************************************************
 * @fn      ETH_ReadPHYRegister
 *
 * @brief   Read a PHY register.
 *
 * @param   PHYAddress - PHY device address, is the index of one of supported 32
 * PHY devices. PHYReg - PHY register address, is the index of one of the 32 PHY
 * register.
 *
 * @return  false - in case of timeout.
 *          MAC MIIDR register value - Data read from the selected PHY register.
 */
uint16_t ETH_ReadPHYRegister(uint16_t PHYAddress, uint16_t PHYReg)
{
    uint32_t tmpreg = 0;
    __IO uint32_t timeout = 0;

    tmpreg = ETH->MACMIIAR;
    tmpreg &= ~MACMIIAR_CR_MASK;
    tmpreg |= (((uint32_t)PHYAddress << 11) & ETH_MACMIIAR_PA);
    tmpreg |= (((uint32_t)PHYReg << 6) & ETH_MACMIIAR_MR);
    tmpreg &= ~ETH_MACMIIAR_MW;
    tmpreg |= ETH_MACMIIAR_MB;
    ETH->MACMIIAR = tmpreg;

    do
    {
        timeout++;
        tmpreg = ETH->MACMIIAR;
        Logger("MACMIAR=0x%x\n", tmpreg);
    } while ((tmpreg & ETH_MACMIIAR_MB) && (timeout < (uint32_t)PHY_READ_TO));

    if (timeout == PHY_READ_TO)
    {
        return (uint16_t)false;
    }

    return (uint16_t)(ETH->MACMIIDR);
}

/*********************************************************************
 * @fn      ETH_WritePHYRegister
 *
 * @brief   Write to a PHY register.
 *
 * @param   PHYAddress - PHY device address, is the index of one of supported 32
 * PHY devices. PHYReg - PHY register address, is the index of one of the 32 PHY
 * register. PHYValue - the value to write.
 *
 * @return  false - in case of timeout.
 *          true - for correct write
 */
bool ETH_WritePHYRegister(uint16_t PHYAddress, uint16_t PHYReg, uint16_t PHYValue)
{
    uint32_t tmpreg = 0;
    __IO uint32_t timeout = 0;

    tmpreg = ETH->MACMIIAR;
    tmpreg &= ~MACMIIAR_CR_MASK;
    tmpreg |= (((uint32_t)PHYAddress << 11) & ETH_MACMIIAR_PA);
    tmpreg |= (((uint32_t)PHYReg << 6) & ETH_MACMIIAR_MR);
    tmpreg |= ETH_MACMIIAR_MW;
    tmpreg |= ETH_MACMIIAR_MB;
    ETH->MACMIIDR = PHYValue;
    ETH->MACMIIAR = tmpreg;

    do
    {
        timeout++;
        tmpreg = ETH->MACMIIAR;
    } while ((tmpreg & ETH_MACMIIAR_MB) && (timeout < (uint32_t)PHY_WRITE_TO));

    if (timeout >= PHY_WRITE_TO)
    {
        return false;
    }

    return true;
}

/*********************************************************************
 * @fn      ETH_SoftwareReset
 *
 * @brief   Resets all MAC subsystem internal registers and logic.
 *
 * @return  none
 */
void ETH_SoftwareReset(void)
{
    ETHDMA->DMABMR |= ETH_DMABMR_SR;
    /* Wait for software reset */
    uint32_t timeout = 10;
    bool cont = true;
    while (cont)
    {
        if (ETHDMA->DMABMR & ETH_DMABMR_SR)
        {
            timeout--;
            if (timeout == 0)
            {
                Logger("Error:Eth soft-reset timeout!\nPlease check RGMII TX & RX clock "
                       "line. \r\n");
                cont = false;
            }
            lnDelayMs(100);
        }
        else
            cont = false;
    }
}
/**
 * @brief [TODO:description]
 */
void WCH_macInit()
{
    ETH_DMAITConfig(ETH_DMA_IT_NIS | ETH_DMA_IT_R | ETH_DMA_IT_T, true);
    ETH_DMATxDescChainInit(DMATxDscrTab, &Tx_Buff[0][0], ETH_TXBUFNB);
    ETH_DMARxDescChainInit(DMARxDscrTab, &Rx_Buff[0][0], ETH_RXBUFNB);
    ETH_Start();
    enable_eth_irq();
}
extern void ln_ethernet_rx(void);

/*********************************************************************
 * @fn      ETH_IRQHandler
 *
 * @brief   This function handles ETH exception.
 *
 * @return  none
 */
extern "C" void ETH_IRQHandler(void)
{
    if (ETHDMA->DMASR & ETH_DMA_IT_R)
    {
        ETH_DMAClearITPendingBit(ETH_DMA_IT_R);
        ln_ethernet_rx();
    }
    if (ETHDMA->DMASR & ETH_DMA_IT_T)
    {
        ETH_DMAClearITPendingBit(ETH_DMA_IT_T);
    }

    ETH_DMAClearITPendingBit(ETH_DMA_IT_NIS);
}

// EOF
