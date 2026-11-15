#if 0
/*********************************************************************
 * @fn      ETH_DeInit
 *
 * @brief   ETH hardware initialize again.
 *
 * @return  none
 */
#ifdef CH32V30x_D8C
void ETH_DeInit(void)
{
    RCC_AHBPeriphResetCmd(RCC_AHBPeriph_ETH_MAC, true);
    RCC_AHBPeriphResetCmd(RCC_AHBPeriph_ETH_MAC, false);
}

#endif

/*********************************************************************
 * @fn      ETH_HandleTxPkt
 *
 * @brief   Transmits a packet, from application buffer, pointed by ppkt.
 *
 * @param   ppkt - pointer to the application's packet buffer to transmit.
 *          FrameLength - Tx Packet size.
 *
 * @return  false - in case of Tx desc owned by DMA.
 *          true - for correct transmission.
 */
bool ETH_HandleTxPkt(uint8_t *ppkt, uint16_t FrameLength)
{
    uint32_t offset = 0;

    if ((DMATxDescToSet->Status & ETH_DMATxDesc_OWN) != (uint32_t)false)
    {
        return false;
    }

    for (offset = 0; offset < FrameLength; offset++)
    {
        (*(__IO uint8_t *)((DMATxDescToSet->Buffer1Addr) + offset)) = (*(ppkt + offset));
    }

    DMATxDescToSet->ControlBufferSize = (FrameLength & ETH_DMATxDesc_TBS1);
    DMATxDescToSet->Status |= ETH_DMATxDesc_LS | ETH_DMATxDesc_FS;
    DMATxDescToSet->Status |= ETH_DMATxDesc_OWN;

    if ((ETHDMA->DMASR & ETH_DMASR_TBUS) != (uint32_t)false)
    {
        ETHDMA->DMASR = ETH_DMASR_TBUS;
        ETHDMA->DMATPDR = 0;
    }

    if ((DMATxDescToSet->Status & ETH_DMATxDesc_TCH) != (uint32_t)false)
    {
        DMATxDescToSet = (ETH_DMADESCTypeDef *)(DMATxDescToSet->Buffer2NextDescAddr);
    }
    else
    {
        if ((DMATxDescToSet->Status & ETH_DMATxDesc_TER) != (uint32_t)false)
        {
            DMATxDescToSet = (ETH_DMADESCTypeDef *)(ETHDMA->DMATDLAR);
        }
        else
        {
            DMATxDescToSet =
                (ETH_DMADESCTypeDef *)((uint32_t)DMATxDescToSet + 0x10 + ((ETHDMA->DMABMR & ETH_DMABMR_DSL) >> 2));
        }
    }

    return true;
}

/*********************************************************************
 * @fn      ETH_HandleRxPkt
 *
 * @brief   Receives a packet and copies it to memory pointed by ppkt.
 *
 * @param   ppkt - pointer to the application packet receive buffer.
 *
 * @return  false - if there is error in reception
 *          framelength - received packet size if packet reception is correct
 */
uint32_t ETH_HandleRxPkt(uint8_t *ppkt)
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
            (*(ppkt + offset)) = (*(__IO uint8_t *)((DMARxDescToGet->Buffer1Addr) + offset));
        }
    }
    else
    {
        framelength = false;
    }

    DMARxDescToGet->Status = ETH_DMARxDesc_OWN;

    if ((ETHDMA->DMASR & ETH_DMASR_RBUS) != (uint32_t)false)
    {
        ETHDMA->DMASR = ETH_DMASR_RBUS;
        ETHDMA->DMARPDR = 0;
    }

    if ((DMARxDescToGet->ControlBufferSize & ETH_DMARxDesc_RCH) != (uint32_t)false)
    {
        DMARxDescToGet = (ETH_DMADESCTypeDef *)(DMARxDescToGet->Buffer2NextDescAddr);
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

/*********************************************************************
 * @fn      ETH_GetRxPktSize
 *
 * @brief   Get the size of received the received packet.
 *
 * @return  framelength - received packet size
 */
uint32_t ETH_GetRxPktSize(void)
{
    uint32_t frameLength = 0;
    if (((DMARxDescToGet->Status & ETH_DMARxDesc_OWN) == (uint32_t)false) &&
        ((DMARxDescToGet->Status & ETH_DMARxDesc_ES) == (uint32_t)false) &&
        ((DMARxDescToGet->Status & ETH_DMARxDesc_LS) != (uint32_t)false) &&
        ((DMARxDescToGet->Status & ETH_DMARxDesc_FS) != (uint32_t)false))
    {
        frameLength = ETH_GetDMARxDescFrameLength(DMARxDescToGet);
    }

    return frameLength;
}

/*********************************************************************
 * @fn      ETH_DropRxPkt
 *
 * @brief   Drop a Received packet.
 *
 * @return  none
 */
void ETH_DropRxPkt(void)
{
    DMARxDescToGet->Status = ETH_DMARxDesc_OWN;

    if ((DMARxDescToGet->ControlBufferSize & ETH_DMARxDesc_RCH) != (uint32_t)false)
    {
        DMARxDescToGet = (ETH_DMADESCTypeDef *)(DMARxDescToGet->Buffer2NextDescAddr);
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
}
/*********************************************************************
 * @fn      ETH_PHYLoopBackCmd
 *
 * @brief   Enables or disables the PHY loopBack mode.
 *
 * @param   PHYAddress - PHY device address, is the index of one of supported 32
 * PHY devices. NewState - new state of the PHY loopBack mode.
 *
 * @return  false - in case of bad PHY configuration.
 *          true - for correct PHY configuration.
 */
bool ETH_PHYLoopBackCmd(uint16_t PHYAddress, bool NewState)
{
    uint16_t tmpreg = 0;

    tmpreg = ETH_ReadPHYRegister(PHYAddress, PHY_BCR);

    if (NewState != false)
    {
        tmpreg |= PHY_Loopback;
    }
    else
    {
        tmpreg &= (uint16_t)(~(uint16_t)PHY_Loopback);
    }

    if (ETH_WritePHYRegister(PHYAddress, PHY_BCR, tmpreg) != (uint32_t)false)
    {
        return true;
    }
    else
    {
        return false;
    }
}
/*********************************************************************
 * @fn      ETH_GetFlowControlBusyStatus
 *
 * @brief   Enables or disables the MAC reception.
 *
 * @return  The new state of flow control busy status bit (true or false).
 */
bool ETH_GetFlowControlBusyStatus(void)
{
    bool bitstatus = false;

    if ((ETH->MACFCR & ETH_MACFCR_FCBBPA) != (uint32_t)false)
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
 * @fn      ETH_InitiatePauseControlFrame
 *
 * @brief   Initiate a Pause Control Frame (Full-duplex only).
 *
 * @return  none
 */
void ETH_InitiatePauseControlFrame(void)
{
    ETH->MACFCR |= ETH_MACFCR_FCBBPA;
}

/*********************************************************************
 * @fn      ETH_BackPressureActivationCmd
 *
 * @brief   Enables or disables the MAC BackPressure operation activation
 * (Half-duplex only).
 *
 * @param   NewState - new state of the MAC BackPressure operation activation.
 *
 * @return  none
 */
void ETH_BackPressureActivationCmd(bool NewState)
{
    if (NewState != false)
    {
        ETH->MACFCR |= ETH_MACFCR_FCBBPA;
    }
    else
    {
        ETH->MACFCR &= ~ETH_MACFCR_FCBBPA;
    }
}

/*********************************************************************
 * @fn      ETH_GetMACbool
 *
 * @brief   Checks whether the specified ETHERNET MAC flag is set or not.
 *
 * @param   ETH_MAC_FLAG - specifies the flag to check.
 *
 * @return  The new state of ETHERNET MAC flag (true or false).
 */
bool ETH_GetMACFlagStatus(uint32_t ETH_MAC_FLAG)
{
    bool bitstatus = false;

    if ((ETH->MACSR & ETH_MAC_FLAG) != (uint32_t)false)
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
 * @fn      ETH_GetMACbool
 *
 * @brief   Checks whether the specified ETHERNET MAC interrupt has occurred or
 * not.
 *
 * @param   ETH_MAC_IT - specifies the interrupt source to check.
 *
 * @return  The new state of ETHERNET MAC interrupt (true or false).
 */
bool ETH_GetMACITStatus(uint32_t ETH_MAC_IT)
{
    bool bitstatus = false;

    if ((ETH->MACSR & ETH_MAC_IT) != (uint32_t)false)
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
 * @fn      ETH_MACITConfig
 *
 * @brief   Enables or disables the specified ETHERNET MAC interrupts.
 *
 * @param   ETH_MAC_IT - specifies the interrupt source to check.
 *          NewState - new state of the specified ETHERNET MAC interrupts.
 *
 * @return  none
 */
void ETH_MACITConfig(uint32_t ETH_MAC_IT, bool NewState)
{
    if (NewState != false)
    {
        ETH->MACIMR &= (~(uint32_t)ETH_MAC_IT);
    }
    else
    {
        ETH->MACIMR |= ETH_MAC_IT;
    }
}

/*********************************************************************
 * @fn      ETH_MACAddressConfig
 *
 * @brief   Configures the selected MAC address.
 *
 * @param   MacAddr - The MAC addres to configure.
 *            ETH_MAC_Address0 - MAC Address0
 *            ETH_MAC_Address1 - MAC Address1
 *            ETH_MAC_Address2 - MAC Address2
 *            ETH_MAC_Address3 - MAC Address3
 *            Addr - Pointer on MAC address buffer data (6 bytes).
 *
 * @return  none
 */
void ETH_MACAddressConfig(uint32_t MacAddr, uint8_t *Addr)
{
    uint32_t tmpreg;

    tmpreg = ((uint32_t)Addr[5] << 8) | (uint32_t)Addr[4];
    (*(__IO uint32_t *)(ETH_MAC_ADDR_HBASE + MacAddr)) = tmpreg;
    tmpreg = ((uint32_t)Addr[3] << 24) | ((uint32_t)Addr[2] << 16) | ((uint32_t)Addr[1] << 8) | Addr[0];

    (*(__IO uint32_t *)(ETH_MAC_ADDR_LBASE + MacAddr)) = tmpreg;
}

/*********************************************************************
 * @fn      ETH_GetMACAddress
 *
 * @brief   Get the selected MAC address.
 *
 * @param   MacAddr - The MAC address to return.
 *            ETH_MAC_Address0 - MAC Address0
 *            ETH_MAC_Address1 - MAC Address1
 *            ETH_MAC_Address2 - MAC Address2
 *            ETH_MAC_Address3 - MAC Address3
 *            Addr - Pointer on MAC address buffer data (6 bytes).
 *
 * @return  none
 */
void ETH_GetMACAddress(uint32_t MacAddr, uint8_t *Addr)
{
    uint32_t tmpreg;

    tmpreg = (*(__IO uint32_t *)(ETH_MAC_ADDR_HBASE + MacAddr));

    Addr[5] = ((tmpreg >> 8) & (uint8_t)0xFF);
    Addr[4] = (tmpreg & (uint8_t)0xFF);
    tmpreg = (*(__IO uint32_t *)(ETH_MAC_ADDR_LBASE + MacAddr));
    Addr[3] = ((tmpreg >> 24) & (uint8_t)0xFF);
    Addr[2] = ((tmpreg >> 16) & (uint8_t)0xFF);
    Addr[1] = ((tmpreg >> 8) & (uint8_t)0xFF);
    Addr[0] = (tmpreg & (uint8_t)0xFF);
}

/*********************************************************************
 * @fn      ETH_MACAddressPerfectFilterCmd
 *
 * @brief   Enables or disables the Address filter module uses the specified.
 *
 * @param   MacAddr - The MAC address to return.
 *            ETH_MAC_Address0 - MAC Address0
 *            ETH_MAC_Address1 - MAC Address1
 *            ETH_MAC_Address2 - MAC Address2
 *            ETH_MAC_Address3 - MAC Address3
 *            NewState - new state of the specified ETHERNET MAC address use.
 *
 * @return  none
 */
void ETH_MACAddressPerfectFilterCmd(uint32_t MacAddr, bool NewState)
{
    if (NewState != false)
    {
        (*(__IO uint32_t *)(ETH_MAC_ADDR_HBASE + MacAddr)) |= ETH_MACA1HR_AE;
    }
    else
    {
        (*(__IO uint32_t *)(ETH_MAC_ADDR_HBASE + MacAddr)) &= (~(uint32_t)ETH_MACA1HR_AE);
    }
}

/*********************************************************************
 * @fn      ETH_MACAddressFilterConfig
 *
 * @brief   Set the filter type for the specified ETHERNET MAC address.
 *
 * @param   MacAddr - specifies the ETHERNET MAC address.
 *            ETH_MAC_Address0 - MAC Address0
 *            ETH_MAC_Address1 - MAC Address1
 *            ETH_MAC_Address2 - MAC Address2
 *            ETH_MAC_Address3 - MAC Address3
 *          Filter - specifies the used frame received field for comparaison.
 *            ETH_MAC_AddressFilter_SA - MAC Address is used to compare with the
 *        SA fields of the received frame.
 *            ETH_MAC_AddressFilter_DA - MAC Address is used to compare with the
 *        DA fields of the received frame.
 *
 * @return  none
 */
void ETH_MACAddressFilterConfig(uint32_t MacAddr, uint32_t Filter)
{
    if (Filter != ETH_MAC_AddressFilter_DA)
    {
        (*(__IO uint32_t *)(ETH_MAC_ADDR_HBASE + MacAddr)) |= ETH_MACA1HR_SA;
    }
    else
    {
        (*(__IO uint32_t *)(ETH_MAC_ADDR_HBASE + MacAddr)) &= (~(uint32_t)ETH_MACA1HR_SA);
    }
}

/*********************************************************************
 * @fn      ETH_MACAddressMaskBytesFilterConfig
 *
 * @brief   Set the filter type for the specified ETHERNET MAC address.
 *
 * @param   MacAddr - specifies the ETHERNET MAC address.
 *            ETH_MAC_Address1 - MAC Address1
 *            ETH_MAC_Address2 - MAC Address2
 *            ETH_MAC_Address3 - MAC Address3
 *          MaskByte - specifies the used address bytes for comparaison
 *            ETH_MAC_AddressMask_Byte5 - Mask MAC Address high reg bits [7:0].
 *            ETH_MAC_AddressMask_Byte4 - Mask MAC Address low reg bits [31:24].
 *            ETH_MAC_AddressMask_Byte3 - Mask MAC Address low reg bits [23:16].
 *            ETH_MAC_AddressMask_Byte2 - Mask MAC Address low reg bits [15:8].
 *            ETH_MAC_AddressMask_Byte1 - Mask MAC Address low reg bits [7:0].
 *
 * @return  none
 */
void ETH_MACAddressMaskBytesFilterConfig(uint32_t MacAddr, uint32_t MaskByte)
{
    (*(__IO uint32_t *)(ETH_MAC_ADDR_HBASE + MacAddr)) &= (~(uint32_t)ETH_MACA1HR_MBC);
    (*(__IO uint32_t *)(ETH_MAC_ADDR_HBASE + MacAddr)) |= MaskByte;
}
/*********************************************************************
 * @fn      ETH_DMATxDescRingInit
 *
 * @brief   Initializes the DMA Tx descriptors in ring mode.
 *
 * @param   DMATxDescTab - Pointer on the first Tx desc list.
 *            TxBuff1 - Pointer on the first TxBuffer1 list.
 *            TxBuff2 - Pointer on the first TxBuffer2 list.
 *            TxBuffCount - Number of the used Tx desc in the list.
 *
 * @return  none
 */
void ETH_DMATxDescRingInit(ETH_DMADESCTypeDef *DMATxDescTab, uint8_t *TxBuff1, uint8_t *TxBuff2, uint32_t TxBuffCount)
{
    uint32_t i = 0;
    ETH_DMADESCTypeDef *DMATxDesc;

    DMATxDescToSet = DMATxDescTab;

    for (i = 0; i < TxBuffCount; i++)
    {
        DMATxDesc = DMATxDescTab + i;
        DMATxDesc->Buffer1Addr = (uint32_t)(&TxBuff1[i * ETH_MAX_PACKET_SIZE]);
        DMATxDesc->Buffer2NextDescAddr = (uint32_t)(&TxBuff2[i * ETH_MAX_PACKET_SIZE]);

        if (i == (TxBuffCount - 1))
        {
            DMATxDesc->Status = ETH_DMATxDesc_TER;
        }
    }

    ETHDMA->DMATDLAR = (uint32_t)DMATxDescTab;
}

/*********************************************************************
 * @fn      ETH_GetDMATxDescbool
 *
 * @brief   Checks whether the specified ETHERNET DMA Tx Desc flag is set or
 * not.
 *
 * @param   DMATxDesc - pointer on a DMA Tx descriptor
 *          ETH_DMATxDescFlag - specifies the flag to check.
 *            ETH_DMATxDesc_OWN - OWN bit - descriptor is owned by DMA engine
 *            ETH_DMATxDesc_IC - Interrupt on completetion
 *            ETH_DMATxDesc_LS - Last Segment
 *            ETH_DMATxDesc_FS - First Segment
 *            ETH_DMATxDesc_DC - Disable CRC
 *            ETH_DMATxDesc_DP - Disable Pad
 *            ETH_DMATxDesc_TTSE - Transmit Time Stamp Enable
 *            ETH_DMATxDesc_TER - Transmit End of Ring
 *            ETH_DMATxDesc_TCH - Second Address Chained
 *            ETH_DMATxDesc_TTSS - Tx Time Stamp Status
 *            ETH_DMATxDesc_IHE - IP Header Error
 *            ETH_DMATxDesc_ES - Error summary
 *            ETH_DMATxDesc_JT - Jabber Timeout
 *            ETH_DMATxDesc_FF - Frame Flushed - DMA/MTL flushed the frame due
 * to SW flush ETH_DMATxDesc_PCE - Payload Checksum Error ETH_DMATxDesc_LCA -
 * Loss of Carrier - carrier lost during tramsmission ETH_DMATxDesc_NC - No
 * Carrier - no carrier signal from the tranceiver ETH_DMATxDesc_LCO - Late
 * Collision - transmission aborted due to collision ETH_DMATxDesc_EC -
 * Excessive Collision - transmission aborted after 16 collisions
 *            ETH_DMATxDesc_VF - VLAN Frame
 *            ETH_DMATxDesc_CC - Collision Count
 *            ETH_DMATxDesc_ED - Excessive Deferral
 *            ETH_DMATxDesc_UF - Underflow Error - late data arrival from the
 * memory ETH_DMATxDesc_DB - Deferred Bit
 *
 * @return  The new state of ETH_DMATxDescFlag (true or false).
 */
bool ETH_GetDMATxDescFlagStatus(ETH_DMADESCTypeDef *DMATxDesc, uint32_t ETH_DMATxDescFlag)
{
    bool bitstatus = false;

    if ((DMATxDesc->Status & ETH_DMATxDescFlag) != (uint32_t)false)
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
 * @fn      ETH_GetDMATxDescCollisionCount
 *
 * @brief   Returns the specified ETHERNET DMA Tx Desc collision count.
 *
 * @param   pointer on a DMA Tx descriptor.
 *
 * @return  The Transmit descriptor collision counter value.
 */
uint32_t ETH_GetDMATxDescCollisionCount(ETH_DMADESCTypeDef *DMATxDesc)
{
    return ((DMATxDesc->Status & ETH_DMATxDesc_CC) >> ETH_DMATXDESC_COLLISION_COUNTSHIFT);
}

/*********************************************************************
 * @fn      ETH_SetDMATxDescOwnBit
 *
 * @brief   Set the specified DMA Tx Desc Own bit.
 *
 * @param   DMATxDesc - Pointer on a Tx desc
 *
 * @return  none
 */
void ETH_SetDMATxDescOwnBit(ETH_DMADESCTypeDef *DMATxDesc)
{
    DMATxDesc->Status |= ETH_DMATxDesc_OWN;
}

/*********************************************************************
 * @fn      ETH_DMATxDescTransmitITConfig
 *
 * @brief   Enables or disables the specified DMA Tx Desc Transmit interrupt.
 *
 * @param   Pointer on a Tx desc.
 *          NewState - new state of the DMA Tx Desc transmit interrupt.
 *
 * @return  none
 */
void ETH_DMATxDescTransmitITConfig(ETH_DMADESCTypeDef *DMATxDesc, bool NewState)
{
    if (NewState != false)
    {
        DMATxDesc->Status |= ETH_DMATxDesc_IC;
    }
    else
    {
        DMATxDesc->Status &= (~(uint32_t)ETH_DMATxDesc_IC);
    }
}

/*********************************************************************
 * @fn      ETH_DMATxDescFrameSegmentConfig
 *
 * @brief   Enables or disables the specified DMA Tx Desc Transmit interrupt.
 *
 * @param   PDMATxDesc - Pointer on a Tx desc.
 *          ETH_DMATxDesc_FirstSegment - actual Tx desc contain first segment.
 *
 * @return  none
 */
void ETH_DMATxDescFrameSegmentConfig(ETH_DMADESCTypeDef *DMATxDesc, uint32_t DMATxDesc_FrameSegment)
{
    DMATxDesc->Status |= DMATxDesc_FrameSegment;
}

/*********************************************************************
 * @fn      ETH_DMATxDescChecksumInsertionConfig
 *
 * @brief   Selects the specified ETHERNET DMA Tx Desc Checksum Insertion.
 *
 * @param   DMATxDesc - pointer on a DMA Tx descriptor.
 *          DMATxDesc_Checksum - specifies is the DMA Tx desc checksum
 * insertion.
 *
 * @return  none
 */
void ETH_DMATxDescChecksumInsertionConfig(ETH_DMADESCTypeDef *DMATxDesc, uint32_t DMATxDesc_Checksum)
{
    DMATxDesc->Status |= DMATxDesc_Checksum;
}

/*********************************************************************
 * @fn      ETH_DMATxDescCRCCmd
 *
 * @brief   Enables or disables the DMA Tx Desc CRC.
 *
 * @param   DMATxDesc - pointer on a DMA Tx descriptor
 *          NewState - new state of the specified DMA Tx Desc CRC.
 *
 * @return  none
 */
void ETH_DMATxDescCRCCmd(ETH_DMADESCTypeDef *DMATxDesc, bool NewState)
{
    if (NewState != false)
    {
        DMATxDesc->Status &= (~(uint32_t)ETH_DMATxDesc_DC);
    }
    else
    {
        DMATxDesc->Status |= ETH_DMATxDesc_DC;
    }
}

/*********************************************************************
 * @fn      ETH_DMATxDescEndOfRingCmd
 *
 * @brief   Enables or disables the DMA Tx Desc end of ring.
 *
 * @param   DMATxDesc - pointer on a DMA Tx descriptor.
 *          NewState - new state of the specified DMA Tx Desc end of ring.
 *
 * @return  none
 */
void ETH_DMATxDescEndOfRingCmd(ETH_DMADESCTypeDef *DMATxDesc, bool NewState)
{
    if (NewState != false)
    {
        DMATxDesc->Status |= ETH_DMATxDesc_TER;
    }
    else
    {
        DMATxDesc->Status &= (~(uint32_t)ETH_DMATxDesc_TER);
    }
}

/*********************************************************************
 * @fn      ETH_DMATxDescSecondAddressChainedCmd
 *
 * @brief   Enables or disables the DMA Tx Desc second address chained.
 *
 * @param   DMATxDesc - pointer on a DMA Tx descriptor
 *          NewState - new state of the specified DMA Tx Desc second address
 * chained.
 *
 * @return  none
 */
void ETH_DMATxDescSecondAddressChainedCmd(ETH_DMADESCTypeDef *DMATxDesc, bool NewState)
{
    if (NewState != false)
    {
        DMATxDesc->Status |= ETH_DMATxDesc_TCH;
    }
    else
    {
        DMATxDesc->Status &= (~(uint32_t)ETH_DMATxDesc_TCH);
    }
}

/*********************************************************************
 * @fn      ETH_DMATxDescShortFramePaddingCmd
 *
 * @brief   Enables or disables the DMA Tx Desc padding for frame shorter than
 * 64 bytes.
 *
 * @param   DMATxDesc - pointer on a DMA Tx descriptor.
 *          NewState - new state of the specified DMA Tx Desc padding for frame
 * shorter than 64 bytes.
 *
 * @return  none
 */
void ETH_DMATxDescShortFramePaddingCmd(ETH_DMADESCTypeDef *DMATxDesc, bool NewState)
{
    if (NewState != false)
    {
        DMATxDesc->Status &= (~(uint32_t)ETH_DMATxDesc_DP);
    }
    else
    {
        DMATxDesc->Status |= ETH_DMATxDesc_DP;
    }
}

/*********************************************************************
 * @fn      ETH_DMATxDescTimeStampCmd
 *
 * @brief   Enables or disables the DMA Tx Desc time stamp.
 *
 * @param   DMATxDesc - pointer on a DMA Tx descriptor
 *          NewState - new state of the specified DMA Tx Desc time stamp.
 *
 * @return  none
 */
void ETH_DMATxDescTimeStampCmd(ETH_DMADESCTypeDef *DMATxDesc, bool NewState)
{
    if (NewState != false)
    {
        DMATxDesc->Status |= ETH_DMATxDesc_TTSE;
    }
    else
    {
        DMATxDesc->Status &= (~(uint32_t)ETH_DMATxDesc_TTSE);
    }
}

/*********************************************************************
 * @fn      ETH_DMATxDescBufferSizeConfig
 *
 * @brief   Configures the specified DMA Tx Desc buffer1 and buffer2 sizes.
 *
 * @param   DMATxDesc - Pointer on a Tx desc.
 *          BufferSize1 - specifies the Tx desc buffer1 size.
 *          RxBuff2 - Pointer on the first RxBuffer2 list
 *          BufferSize2 - specifies the Tx desc buffer2 size (put "0" if not
 * used).
 *
 * @return  none
 */
void ETH_DMATxDescBufferSizeConfig(ETH_DMADESCTypeDef *DMATxDesc, uint32_t BufferSize1, uint32_t BufferSize2)
{
    DMATxDesc->ControlBufferSize |= (BufferSize1 | (BufferSize2 << ETH_DMATXDESC_BUFFER2_SIZESHIFT));
}
/*********************************************************************
 * @fn      ETH_DMARxDescRingInit
 *
 * @brief   Initializes the DMA Rx descriptors in ring mode.
 *
 * @param   DMARxDescTab - Pointer on the first Rx desc list.
 *            RxBuff1 - Pointer on the first RxBuffer1 list.
 *            RxBuff2 - Pointer on the first RxBuffer2 list
 *            RxBuffCount - Number of the used Rx desc in the list.
 *
 * @return  none
 */
void ETH_DMARxDescRingInit(ETH_DMADESCTypeDef *DMARxDescTab, uint8_t *RxBuff1, uint8_t *RxBuff2, uint32_t RxBuffCount)
{
    uint32_t i = 0;
    ETH_DMADESCTypeDef *DMARxDesc;

    DMARxDescToGet = DMARxDescTab;

    for (i = 0; i < RxBuffCount; i++)
    {
        DMARxDesc = DMARxDescTab + i;
        DMARxDesc->Status = ETH_DMARxDesc_OWN;
        DMARxDesc->ControlBufferSize = ETH_MAX_PACKET_SIZE;
        DMARxDesc->Buffer1Addr = (uint32_t)(&RxBuff1[i * ETH_MAX_PACKET_SIZE]);
        DMARxDesc->Buffer2NextDescAddr = (uint32_t)(&RxBuff2[i * ETH_MAX_PACKET_SIZE]);

        if (i == (RxBuffCount - 1))
        {
            DMARxDesc->ControlBufferSize |= ETH_DMARxDesc_RER;
        }
    }

    ETHDMA->DMARDLAR = (uint32_t)DMARxDescTab;
}

/*********************************************************************
 * @fn      ETH_GetDMARxDescbool
 *
 * @brief   Checks whether the specified ETHERNET Rx Desc flag is set or not.
 *
 * @param   DMARxDesc - pointer on a DMA Rx descriptor.
 *          ETH_DMARxDescFlag - specifies the flag to check.
 *            ETH_DMARxDesc_OWN - OWN bit: descriptor is owned by DMA engine
 *            ETH_DMARxDesc_AFM - DA Filter Fail for the rx frame
 *            ETH_DMARxDesc_ES - Error summary
 *            ETH_DMARxDesc_DE - Desciptor error: no more descriptors for
 * receive frame ETH_DMARxDesc_SAF - SA Filter Fail for the received frame
 *            ETH_DMARxDesc_LE - Frame size not matching with length field
 *            ETH_DMARxDesc_OE - Overflow Error: Frame was damaged due to buffer
 * overflow ETH_DMARxDesc_VLAN - VLAN Tag: received frame is a VLAN frame
 *            ETH_DMARxDesc_FS - First descriptor of the frame
 *            ETH_DMARxDesc_LS - Last descriptor of the frame
 *            ETH_DMARxDesc_IPV4HCE - IPC Checksum Error/Giant Frame: Rx Ipv4
 * header checksum error ETH_DMARxDesc_LC - Late collision occurred during
 * reception ETH_DMARxDesc_FT - Frame type - Ethernet, otherwise 802.3
 *            ETH_DMARxDesc_RWT - Receive Watchdog Timeout: watchdog timer
 * expired during reception ETH_DMARxDesc_RE - Receive error: error reported by
 * MII interface ETH_DMARxDesc_DE - Dribble bit error: frame contains non int
 * multiple of 8 bits ETH_DMARxDesc_CE - CRC error ETH_DMARxDesc_MAMPCE - Rx MAC
 * Address/Payload Checksum Error: Rx MAC address matched/ Rx Payload Checksum
 * Error
 *
 * @return  The new state of ETH_DMARxDescFlag (true or false).
 */
bool ETH_GetDMARxDescFlagStatus(ETH_DMADESCTypeDef *DMARxDesc, uint32_t ETH_DMARxDescFlag)
{
    bool bitstatus = false;

    if ((DMARxDesc->Status & ETH_DMARxDescFlag) != (uint32_t)false)
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
 * @fn      ETH_SetDMARxDescOwnBit
 *
 * @brief   Set the specified DMA Rx Desc Own bit.
 *
 * @param   DMARxDesc - Pointer on a Rx desc
 *
 * @return  none
 */
void ETH_SetDMARxDescOwnBit(ETH_DMADESCTypeDef *DMARxDesc)
{
    DMARxDesc->Status |= ETH_DMARxDesc_OWN;
}

/*********************************************************************
 * @fn      ETH_GetDMARxDescFrameLength
 *
 * @brief   Returns the specified DMA Rx Desc frame length.
 *
 * @param   DMARxDesc - pointer on a DMA Rx descriptor
 *
 * @return  The Rx descriptor received frame length.
 */
uint32_t ETH_GetDMARxDescFrameLength(ETH_DMADESCTypeDef *DMARxDesc)
{
    return ((DMARxDesc->Status & ETH_DMARxDesc_FL) >> ETH_DMARXDESC_FRAME_LENGTHSHIFT);
}

/*********************************************************************
 * @fn      ETH_DMARxDescReceiveITConfig
 *
 * @brief   Enables or disables the specified DMA Rx Desc receive interrupt.
 *
 * @param   DMARxDesc - Pointer on a Rx desc
 *          NewState - true or false.
 *
 * @return  none
 */
void ETH_DMARxDescReceiveITConfig(ETH_DMADESCTypeDef *DMARxDesc, bool NewState)
{
    if (NewState != false)
    {
        DMARxDesc->ControlBufferSize &= (~(uint32_t)ETH_DMARxDesc_DIC);
    }
    else
    {
        DMARxDesc->ControlBufferSize |= ETH_DMARxDesc_DIC;
    }
}

/*********************************************************************
 * @fn      ETH_DMARxDescEndOfRingCmd
 *
 * @brief   Enables or disables the DMA Rx Desc end of ring.
 *
 * @param   DMARxDesc - pointer on a DMA Rx descriptor.
 *          NewState - new state of the specified DMA Rx Desc end of ring.
 *
 * @return  none
 */
void ETH_DMARxDescEndOfRingCmd(ETH_DMADESCTypeDef *DMARxDesc, bool NewState)
{
    if (NewState != false)
    {
        DMARxDesc->ControlBufferSize |= ETH_DMARxDesc_RER;
    }
    else
    {
        DMARxDesc->ControlBufferSize &= (~(uint32_t)ETH_DMARxDesc_RER);
    }
}

/*********************************************************************
 * @fn      ETH_DMARxDescSecondAddressChainedCmd
 *
 * @brief   Returns the specified ETHERNET DMA Rx Desc buffer size.
 *
 * @param   DMARxDesc - pointer on a DMA Rx descriptor.
 *          NewState - new state of the specified DMA Rx Desc second address
 * chained.
 *
 * @return  none
 */
void ETH_DMARxDescSecondAddressChainedCmd(ETH_DMADESCTypeDef *DMARxDesc, bool NewState)
{
    if (NewState != false)
    {
        DMARxDesc->ControlBufferSize |= ETH_DMARxDesc_RCH;
    }
    else
    {
        DMARxDesc->ControlBufferSize &= (~(uint32_t)ETH_DMARxDesc_RCH);
    }
}

/*********************************************************************
 * @fn      ETH_GetDMARxDescBufferSize
 *
 * @brief   Returns the specified ETHERNET DMA Rx Desc buffer size.
 *
 * @param   DMARxDesc - pointer on a DMA Rx descriptor.
 *          DMARxDesc_Buffer - specifies the DMA Rx Desc buffer.
 *          ETH_DMARxDesc_Buffer1 - DMA Rx Desc Buffer1
 *          ETH_DMARxDesc_Buffer2 - DMA Rx Desc Buffer2
 *
 * @return  The Receive descriptor frame length.
 */
uint32_t ETH_GetDMARxDescBufferSize(ETH_DMADESCTypeDef *DMARxDesc, uint32_t DMARxDesc_Buffer)
{
    if (DMARxDesc_Buffer != ETH_DMARxDesc_Buffer1)
    {
        return ((DMARxDesc->ControlBufferSize & ETH_DMARxDesc_RBS2) >> ETH_DMARXDESC_BUFFER2_SIZESHIFT);
    }
    else
    {
        return (DMARxDesc->ControlBufferSize & ETH_DMARxDesc_RBS1);
    }
}

/*********************************************************************
 * @fn      ETH_GetlinkStaus
 *
 * @brief   Checks whether the internal 10BASE-T PHY is link or not.
 *
 * @return  Internal 10BASE-T PHY is link or not.
 */
bool ETH_GetlinkStaus(void)
{
    bool bitstatus = false;

    if ((ETHDMA->DMASR & 0x80000000) != (uint32_t)false)
    {
        bitstatus = (bool)PHY_10BASE_T_LINKED;
    }
    else
    {
        bitstatus = (bool)PHY_10BASE_T_NOT_LINKED;
    }

    return bitstatus;
}

/*********************************************************************
 * @fn      ETH_GetDMAbool
 *
 * @brief   Checks whether the specified ETHERNET DMA flag is set or not.
 *
 * @param   ETH_DMA_FLAG - specifies the flag to check.
 *            ETH_DMA_FLAG_TST - Time-stamp trigger flag
 *            ETH_DMA_FLAG_PMT - PMT flag
 *            ETH_DMA_FLAG_MMC - MMC flag
 *            ETH_DMA_FLAG_DataTransferError - Error bits 0-data buffer, 1-desc.
 * access ETH_DMA_FLAG_ReadWriteError - Error bits 0-write trnsf, 1-read transfr
 *            ETH_DMA_FLAG_AccessError - Error bits 0-Rx DMA, 1-Tx DMA
 *            ETH_DMA_FLAG_NIS - Normal interrupt summary flag
 *            ETH_DMA_FLAG_AIS - Abnormal interrupt summary flag
 *            ETH_DMA_FLAG_ER - Early receive flag
 *            ETH_DMA_FLAG_FBE - Fatal bus error flag
 *            ETH_DMA_FLAG_ET - Early transmit flag
 *            ETH_DMA_FLAG_RWT - Receive watchdog timeout flag
 *            ETH_DMA_FLAG_RPS - Receive process stopped flag
 *            ETH_DMA_FLAG_RBU - Receive buffer unavailable flag
 *            ETH_DMA_FLAG_R - Receive flag
 *            ETH_DMA_FLAG_TU - Underflow flag
 *            ETH_DMA_FLAG_RO - Overflow flag
 *            ETH_DMA_FLAG_TJT - Transmit jabber timeout flag
 *            ETH_DMA_FLAG_TBU - Transmit buffer unavailable flag
 *            ETH_DMA_FLAG_TPS - Transmit process stopped flag
 *            ETH_DMA_FLAG_T - Transmit flag
 *
 * @return  Internal 10BASE-T PHY is link or not.
 */
bool ETH_GetDMAFlagStatus(uint32_t ETH_DMA_FLAG)
{
    bool bitstatus = false;

    if ((ETHDMA->DMASR & ETH_DMA_FLAG) != (uint32_t)false)
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
 * @fn      ETH_DMAClearFlag
 *
 * @brief   Checks whether the specified ETHERNET DMA interrupt has occured or
 * not.
 *
 * @param   ETH_DMA_FLAG - specifies the flag to clear.
 *            ETH_DMA_FLAG_NIS - Normal interrupt summary flag
 *            ETH_DMA_FLAG_AIS - Abnormal interrupt summary  flag
 *            ETH_DMA_FLAG_ER - Early receive flag
 *            ETH_DMA_FLAG_FBE - Fatal bus error flag
 *            ETH_DMA_FLAG_ETI - Early transmit flag
 *            ETH_DMA_FLAG_RWT - Receive watchdog timeout flag
 *            ETH_DMA_FLAG_RPS - Receive process stopped flag
 *            ETH_DMA_FLAG_RBU - Receive buffer unavailable flag
 *            ETH_DMA_FLAG_R - Receive flag
 *            ETH_DMA_FLAG_TU - Transmit Underflow flag
 *            ETH_DMA_FLAG_RO - Receive Overflow flag
 *            ETH_DMA_FLAG_TJT - Transmit jabber timeout flag
 *            ETH_DMA_FLAG_TBU - Transmit buffer unavailable flag
 *            ETH_DMA_FLAG_TPS - Transmit process stopped flag
 *            ETH_DMA_FLAG_T - Transmit flag
 *
 * @return  none
 */
void ETH_DMAClearFlag(uint32_t ETH_DMA_FLAG)
{
    ETHDMA->DMASR = (uint32_t)ETH_DMA_FLAG;
}

/*********************************************************************
 * @fn      ETH_GetDMAbool
 *
 * @brief   Checks whether the specified ETHERNET DMA interrupt has occured or
 * not.
 *
 * @param   ETH_DMA_IT - specifies the interrupt pending bit to clear.
 *            ETH_DMA_IT_TST - Time-stamp trigger interrupt
 *            ETH_DMA_IT_PMT - PMT interrupt
 *            ETH_DMA_IT_MMC - MMC interrupt
 *            ETH_DMA_IT_NIS - Normal interrupt summary
 *            ETH_DMA_IT_AIS - Abnormal interrupt summary
 *            ETH_DMA_IT_ER  - Early receive interrupt
 *            ETH_DMA_IT_FBE - Fatal bus error interrupt
 *            ETH_DMA_IT_ET  - Early transmit interrupt
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
 *
 * @return  The new state of ETH_DMA_IT (true or false).
 */
bool ETH_GetDMAITStatus(uint32_t ETH_DMA_IT)
{
    bool bitstatus = false;

    if ((ETHDMA->DMASR & ETH_DMA_IT) != (uint32_t)false)
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
 * @fn      ETH_GetTransmitProcessState
 *
 * @brief   Returns the ETHERNET DMA Transmit Process State.
 *
 * @return  The new ETHERNET DMA Transmit Process State -
 *            ETH_DMA_TransmitProcess_Stopped - Stopped - Reset or Stop Tx
 * Command issued ETH_DMA_TransmitProcess_Fetching - Running - fetching the Tx
 * descriptor ETH_DMA_TransmitProcess_Waiting - Running - waiting for status
 *            ETH_DMA_TransmitProcess_Reading - unning - reading the data from
 * host memory ETH_DMA_TransmitProcess_Suspended - Suspended - Tx Desciptor
 * unavailabe ETH_DMA_TransmitProcess_Closing - Running - closing Rx descriptor
 */
uint32_t ETH_GetTransmitProcessState(void)
{
    return ((uint32_t)(ETHDMA->DMASR & ETH_DMASR_TS));
}

/*********************************************************************
 * @fn      ETH_GetReceiveProcessState
 *
 * @brief   Returns the ETHERNET DMA Receive Process State.
 *
 * @return  The new ETHERNET DMA Receive Process State:
 *            ETH_DMA_ReceiveProcess_Stopped - Stopped - Reset or Stop Rx
 * Command issued ETH_DMA_ReceiveProcess_Fetching - Running - fetching the Rx
 * descriptor ETH_DMA_ReceiveProcess_Waiting - Running - waiting for packet
 *            ETH_DMA_ReceiveProcess_Suspended - Suspended - Rx Desciptor
 * unavailable ETH_DMA_ReceiveProcess_Closing - Running - closing descriptor
 *            ETH_DMA_ReceiveProcess_Queuing - Running - queuing the recieve
 * frame into host memory
 */
uint32_t ETH_GetReceiveProcessState(void)
{
    return ((uint32_t)(ETHDMA->DMASR & ETH_DMASR_RS));
}
/*********************************************************************
 * @fn      ETH_GetDMAOverflowStatus
 *
 * @brief   Checks whether the specified ETHERNET DMA overflow flag is set or
 * not.
 *
 * @param   ETH_DMA_Overflow - specifies the DMA overflow flag to check.
 *            ETH_DMA_Overflow_RxFIFOCounter - Overflow for FIFO Overflow
 * Counter ETH_DMA_Overflow_MissedFrameCounter - Overflow for Missed Frame
 * Counter
 *
 * @return  The new state of ETHERNET DMA overflow Flag (true or false).
 */
bool ETH_GetDMAOverflowStatus(uint32_t ETH_DMA_Overflow)
{
    bool bitstatus = false;

    if ((ETHDMA->DMAMFBOCR & ETH_DMA_Overflow) != (uint32_t)false)
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
 * @fn      ETH_GetRxOverflowMissedFrameCounter
 *
 * @brief   Get the ETHERNET DMA Rx Overflow Missed Frame Counter value.
 *
 * @return  The value of Rx overflow Missed Frame Counter.
 */
uint32_t ETH_GetRxOverflowMissedFrameCounter(void)
{
    return ((uint32_t)((ETHDMA->DMAMFBOCR & ETH_DMAMFBOCR_MFA) >> ETH_DMA_RX_OVERFLOW_MISSEDFRAMES_COUNTERSHIFT));
}

/*********************************************************************
 * @fn      ETH_GetBufferUnavailableMissedFrameCounter
 *
 * @brief   Get the ETHERNET DMA Buffer Unavailable Missed Frame Counter value.
 *
 * @return  The value of Buffer unavailable Missed Frame Counter.
 */
uint32_t ETH_GetBufferUnavailableMissedFrameCounter(void)
{
    return ((uint32_t)(ETHDMA->DMAMFBOCR) & ETH_DMAMFBOCR_MFC);
}

/*********************************************************************
 * @fn      ETH_GetCurrentTxDescStartAddress
 *
 * @brief   Get the ETHERNET DMA DMACHTDR register value.
 *
 * @return  The value of the current Tx desc start address.
 */
uint32_t ETH_GetCurrentTxDescStartAddress(void)
{
    return ((uint32_t)(ETHDMA->DMACHTDR));
}

/*********************************************************************
 * @fn      ETH_GetCurrentRxDescStartAddress
 *
 * @brief   Get the ETHERNET DMA DMACHRDR register value.
 *
 * @return  The value of the current Rx desc start address.
 */
uint32_t ETH_GetCurrentRxDescStartAddress(void)
{
    return ((uint32_t)(ETHDMA->DMACHRDR));
}
/*********************************************************************
 * @fn      ETH_GetCurrentRxBufferAddress
 *
 * @brief   Get the ETHERNET DMA DMACHRBAR register value.
 *
 * @return  The value of the current Rx buffer address.
 */
uint32_t ETH_GetCurrentRxBufferAddress(void)
{
    return ((uint32_t)(ETHDMA->DMACHRBAR));
}

/*********************************************************************
 * @fn      ETH_ResumeDMATransmission
 *
 * @brief   Resumes the DMA Transmission by writing to the DmaTxPollDemand
 * register
 *
 * @return  none
 */
void ETH_ResumeDMATransmission(void)
{
    ETHDMA->DMATPDR = 0;
}

/*********************************************************************
 * @fn      ETH_ResumeDMAReception
 *
 * @brief   Resumes the DMA Transmission by writing to the DmaRxPollDemand
 * register.
 *
 * @return  none
 */
void ETH_ResumeDMAReception(void)
{
    ETHDMA->DMARPDR = 0;
}

/*********************************************************************
 * @fn      ETH_ResetWakeUpFrameFilterRegisterPointer
 *
 * @brief   Reset Wakeup frame filter register pointer.
 *
 * @return  none
 */
void ETH_ResetWakeUpFrameFilterRegisterPointer(void)
{
    ETH->MACPMTCSR |= ETH_MACPMTCSR_WFFRPR;
}

/*********************************************************************
 * @fn      ETH_SetWakeUpFrameFilterRegister
 *
 * @brief   Populates the remote wakeup frame registers.
 *
 * @param   Buffer - Pointer on remote WakeUp Frame Filter Register buffer data
 * (8 words).
 *
 * @return  none
 */
void ETH_SetWakeUpFrameFilterRegister(uint32_t *Buffer)
{
    uint32_t i = 0;

    for (i = 0; i < ETH_WAKEUP_REGISTER_LENGTH; i++)
    {
        ETH->MACRWUFFR = Buffer[i];
    }
}

/*********************************************************************
 * @fn      ETH_GlobalUnicastWakeUpCmd
 *
 * @brief   Enables or disables any unicast packet filtered by the MAC address.
 *
 * @param   NewState - new state of the MAC Global Unicast Wake-Up.
 *
 * @return  none
 */
void ETH_GlobalUnicastWakeUpCmd(bool NewState)
{
    if (NewState != false)
    {
        ETH->MACPMTCSR |= ETH_MACPMTCSR_GU;
    }
    else
    {
        ETH->MACPMTCSR &= ~ETH_MACPMTCSR_GU;
    }
}

/*********************************************************************
 * @fn      ETH_GetPMTbool
 *
 * @brief   Checks whether the specified ETHERNET PMT flag is set or not.
 *
 * @param   ETH_PMT_FLAG - specifies the flag to check.
 *
 * @return  The new state of ETHERNET PMT Flag (true or false).
 */
bool ETH_GetPMTbool(uint32_t ETH_PMT_FLAG)
{
    bool bitstatus = false;

    if ((ETH->MACPMTCSR & ETH_PMT_FLAG) != (uint32_t)false)
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
 * @fn      ETH_WakeUpFrameDetectionCmd
 *
 * @brief   Enables or disables the MAC Wake-Up Frame Detection.
 *
 * @param   NewState - new state of the MAC Wake-Up Frame Detection.
 *
 * @return  none
 */
void ETH_WakeUpFrameDetectionCmd(bool NewState)
{
    if (NewState != false)
    {
        ETH->MACPMTCSR |= ETH_MACPMTCSR_WFE;
    }
    else
    {
        ETH->MACPMTCSR &= ~ETH_MACPMTCSR_WFE;
    }
}

/*********************************************************************
 * @fn      ETH_MagicPacketDetectionCmd
 *
 * @brief   Enables or disables the MAC Magic Packet Detection.
 *
 * @param   NewState - new state of the MAC Magic Packet Detection.
 *
 * @return  none
 */
void ETH_MagicPacketDetectionCmd(bool NewState)
{
    if (NewState != false)
    {
        ETH->MACPMTCSR |= ETH_MACPMTCSR_MPE;
    }
    else
    {
        ETH->MACPMTCSR &= ~ETH_MACPMTCSR_MPE;
    }
}

/*********************************************************************
 * @fn      ETH_PowerDownCmd
 *
 * @brief   Enables or disables the MAC Power Down.
 *
 * @param   NewState - new state of the MAC Power Down.
 *
 * @return  none
 */
void ETH_PowerDownCmd(bool NewState)
{
    if (NewState != false)
    {
        ETH->MACPMTCSR |= ETH_MACPMTCSR_PD;
    }
    else
    {
        ETH->MACPMTCSR &= ~ETH_MACPMTCSR_PD;
    }
}
/*********************************************************************
 * @fn      RGMII_TXC_Delay
 *
 * @brief   Delay time.
 *
 * @return  none
 */
void RGMII_TXC_Delay(uint8_t clock_polarity, uint8_t delay_time)
{
    if (clock_polarity)
    {
        ETH->MACCR |= (uint32_t)(1 << 1);
    }
    else
    {
        ETH->MACCR &= ~(uint32_t)(1 << 1);
    }
    if (delay_time <= 7)
    {
        ETH->MACCR |= (uint32_t)(delay_time << 29);
    }
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
void ETH_DMATxDescChainInit(ETH_DMADESCTypeDef *DMATxDescTab, uint8_t *TxBuff, uint32_t TxBuffCount)
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
 * @fn      ETH_GetSoftwareResetStatus
 *
 * @brief   Checks whether the ETHERNET software reset bit is set or not.
 *
 * @return  The new state of DMA Bus Mode register SR bit (true or false).
 */
bool ETH_GetSoftwareResetStatus(void)
{
    bool bitstatus = false;
    if ((ETHDMA->DMABMR & ETH_DMABMR_SR) != (uint32_t)false)
    {
        bitstatus = true;
    }
    else
    {
        bitstatus = false;
    }
    // printf("ETHDMA->DMABMR is:%08x\n", ETHDMA->DMABMR);

    return bitstatus;
}

#endif
