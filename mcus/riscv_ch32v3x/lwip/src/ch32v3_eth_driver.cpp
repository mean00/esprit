//
#include "esprit.h"
#include "eth_driver.h"
#include "lwipopts.h"
#include "string.h"
#include "task.h"

#include "ch32v3xx_eth_private.h"
//
#include "ln_lwip_debug.h"
/* MII/MDI interface select */
ETH_DMADESCTypeDef *DMATxDescToSet;
ETH_DMADESCTypeDef *DMARxDescToGet;
//
#define PHY_ADDRESS 0x01

#define USE10BASE_T

/* extern variable */
extern void Ethernet_LED_LINKSET(uint8_t setbit);
extern void Ethernet_LED_DATASET(uint8_t setbit);

#define ETH_DMARxDesc_FrameLengthShift 16

static void waitForBitToClear(uint32_t bit)
{
    int32_t timeout = 1000; /* 最大超时十秒   */
    while (1)
    {
        uint32_t RegValue = ETH_ReadPHYRegister(PHY_ADDRESS, PHY_BSR);
        if ((RegValue & (bit)) != 0)
        {
            timeout--;
            if (timeout <= 0)
            {
                Logger("Error:Timeout!!\n");
                xAssert(0);
            }
            lnDelayMs(10);
        }
        else
            return;
    }
}
static void waitForBit(uint32_t bit)
{
    int32_t timeout = 1000; /* 最大超时十秒   */
    while (1)
    {
        uint32_t RegValue = ETH_ReadPHYRegister(PHY_ADDRESS, PHY_BSR);
        Logger("BSR=0x%x\n", RegValue);
        if ((RegValue & (bit)) == 0)
        {
            timeout--;
            if (timeout <= 0)
            {
                Logger("Error:Timeout!!\n");
                xAssert(0);
            }
            lnDelayMs(10);
        }
        else
            return;
    }
}
static void alterBits(volatile uint32_t *reg, uint32_t mask, uint32_t val)
{
    volatile uint32_t tmpreg = *reg;
    tmpreg &= mask;
    tmpreg |= val;
    *reg = tmpreg;
}
/*
 *  It is assumed ETh clock has been set already
 *  And PPL3 is running at 60 Mhz
 */
void ch32v3_init_phy(void)
{
    uint32_t tmpreg;
    uint32_t ETH_Mode; // Selects the MAC duplex mode: Half-Duplex or Full-Duplex mode
    Logger("CH32Vxx Init Phy.\n");
    Ethernet_LED_LINKSET(1); /* turn off link led. */
    Ethernet_LED_DATASET(1); /* turn off data led. */
    /* Software reset */
    ETH_SoftwareReset(); // reset dma

    tmpreg = ETH->MACMIIAR;
    tmpreg &= MACMIIAR_CR_MASK;
    tmpreg |= (uint32_t)ETH_MACMIIAR_CR_Div42; // set phy com speed slow enough for high speed busclock
    ETH->MACMIIAR = (uint32_t)tmpreg;

    /* Reset PHY */
    ETH_WritePHYRegister(PHY_ADDRESS, PHY_BCR, 0);
    ETH_WritePHYRegister(PHY_ADDRESS, PHY_BCR, PHY_Reset);

    lnDelayMs(100);
    Logger("Waiting for reset to clear ...\n");
    waitForBitToClear(PHY_Reset);
    Logger("Waiting for link up...\n");
    waitForBit(PHY_Linked_Status);
    Logger("Waiting for autonego ...\n");
    waitForBit(PHY_AutoNego_Complete);
    Logger("Link up.\n");

    // 16/0x10 is wch special status register
    {
        uint32_t RegValue = ETH_ReadPHYRegister(PHY_ADDRESS, 0x10);
        Logger("PHY_SR value:%04x. \r\n", RegValue);

        if (RegValue & (1 << 2))
        {
            ETH_Mode = ETH_Mode_FullDuplex;
            Logger("Full Duplex. \r\n");
        }
        else
        {
            ETH_Mode = ETH_Mode_HalfDuplex;
            Logger("Half Duplex. \r\n");
        }
        if (RegValue & (1 << 3))
        {
            Logger("Loopback_10M \r\n");
        }
        else
        {
            Logger("Loopback disabled \r\n");
        }
    }
    lnDelayMs(10);

    /* 点亮link状态led灯 */
    Ethernet_LED_LINKSET(0);

    /*------------------------ MAC寄存器配置  -----------------------
     * --------------------*/
    /* MACCCR在RGMII接口模式下具有调整RGMII接口时序的域，请注意 */
    uint32_t val = (uint32_t)(ETH_Watchdog_Enable | ETH_Jabber_Enable | ETH_InterFrameGap_96Bit |
                              ETH_CarrierSense_Enable | ETH_Speed_10M | ETH_ReceiveOwn_Enable |
                              ETH_LoopbackMode_Disable | ETH_Mode | ETH_RetryTransmission_Disable |
                              ETH_AutomaticPadCRCStrip_Disable | ETH_BackOffLimit_10 | ETH_DeferralCheck_Disable);
#ifdef CHECKSUM_BY_HARDWARE
    val |= ETH_ChecksumOffload_Enable;
#else
    val |= ETH_ChecksumOffload_Disable;
#endif
#ifdef USE10BASE_T
    val = ETH_Internal_Pull_Up_Res_Enable; /* 启用内部上拉  */
#endif
    alterBits(&(ETH->MACCR), MACCR_CLEAR_MASK, val);
    alterBits(&(ETH->MACFFR), 0,
              ETH_ReceiveAll_Enable | ETH_SourceAddrFilter_Disable | ETH_PassControlFrames_BlockAll |
                  ETH_BroadcastFramesReception_Enable | ETH_DestinationAddrFilter_Normal | ETH_PromiscuousMode_Enable |
                  ETH_MulticastFramesFilter_Perfect | ETH_UnicastFramesFilter_Perfect);

    /*--------------- ETHERNET MACHTHR and MACHTLR Configuration ---------------*/
    ETH->MACHTHR = 0U; //(uint32_t)ETH_InitStructure.ETH_HashTableHigh;
    ETH->MACHTLR = 0U; //(uint32_t)ETH_InitStructure.ETH_HashTableLow;
    /*----------------------- ETHERNET MACFCR Configuration --------------------*/
    alterBits(&(ETH->MACFCR), MACFCR_CLEAR_MASK,
              (uint32_t)((0 << 16) | // pause time
                         ETH_ZeroQuantaPause_Disable | ETH_PauseLowThreshold_Minus4 |
                         ETH_UnicastPauseFrameDetect_Disable | ETH_ReceiveFlowControl_Disable |
                         ETH_TransmitFlowControl_Disable));
    ///*/*/*/*
    ETH->MACVLANTR = (uint32_t)(ETH_VLANTagComparison_16Bit | 0x0); // last param is vlang tag id =0
                                                                    //

    alterBits(&(ETHDMA->DMAOMR), DMAOMR_CLEAR_MASK,

              (uint32_t)(ETH_DropTCPIPChecksumErrorFrame_Enable | ETH_ReceiveStoreForward_Enable |
                         ETH_FlushReceivedFrame_Enable | ETH_TransmitStoreForward_Enable |
                         ETH_TransmitThresholdControl_64Bytes | ETH_ForwardErrorFrames_Enable |
                         ETH_ForwardUndersizedGoodFrames_Enable | ETH_ReceiveThresholdControl_64Bytes |
                         ETH_SecondFrameOperate_Disable));

    alterBits(&(ETHDMA->DMABMR), 0,
              (uint32_t)(ETH_DMABMR_USP | ETH_AddressAlignedBeats_Enable | ETH_FixedBurst_Enable |
                         ETH_RxDMABurstLength_32Beat | ETH_TxDMABurstLength_32Beat |
                         (0x0 << 2) | //  Deccriptor skip length
                         ETH_DMAArbitration_RoundRobin_RxTx_2_1));
    WCH_macInit();
}

/*********************************************************************
 * @fn      CH30x_RNG_GENERATE
 *
 * @brief   CH30x_RNG_GENERATE api function for lwip.
 *
 * @param   None.
 *
 * @return  None.
 */
static uint32_t rndom = 0xF012345;
uint32_t CH30x_RNG_GENERATE()
{
    return rndom++;
}
/**
 *
 *
 * received data
 *
 *
 */
FrameTypeDef ETH_RxPkt_ChainMode(void)
{
    uint32_t framelength = 0;
    FrameTypeDef frame = {0, 0};

    /* Check if the descriptor is owned by the ETHERNET DMA (when set) or CPU
     * (when reset) */
    if ((DMARxDescToGet->Status & ETH_DMARxDesc_OWN))
    {
        frame.length = 0;
        if ((ETHDMA->DMASR & ETH_DMASR_RBUS))
        {
            /* Clear RBUS ETHERNET DMA flag */
            ETHDMA->DMASR = ETH_DMASR_RBUS;
            /* Resume DMA reception */
            ETHDMA->DMARPDR = 0;
        }
        Logger("Error:ETH_DMARxDesc_OWN.\r\n");
        /* Return error: OWN bit set */
        return frame;
    }

    if (((DMARxDescToGet->Status & ETH_DMARxDesc_ES) == 0) && ((DMARxDescToGet->Status & ETH_DMARxDesc_LS)) &&
        ((DMARxDescToGet->Status & ETH_DMARxDesc_FS)))
    {
        /* Get the Frame Length of the received packet: substruct 4 bytes of the CRC
         */
        framelength = ((DMARxDescToGet->Status & ETH_DMARxDesc_FL) >> ETH_DMARxDesc_FrameLengthShift) - 4;

        /* Get the addrees of the actual buffer */
        frame.buffer = DMARxDescToGet->Buffer1Addr;
    }
    else
    {
        /* Return ERROR */
        framelength = 0;
        Logger("Error:recv error frame,status:0x%08x.\r\n", DMARxDescToGet->Status);
    }
    DMARxDescToGet->Status |= ETH_DMARxDesc_OWN;
    frame.length = framelength;
    frame.descriptor = DMARxDescToGet;

    /* Update the ETHERNET DMA global Rx descriptor with next Rx decriptor */
    /* Chained Mode */
    /* Selects the next DMA Rx descriptor list for next buffer to read */
    DMARxDescToGet = (ETH_DMADESCTypeDef *)(DMARxDescToGet->Buffer2NextDescAddr);
    /* Return Frame */
    return (frame);
}

/*********************************************************************
 * @fn      ETH_TxPkt_ChainMode
 *
 * @brief   MAC send a ethernet frame in chain mode.
 *
 * @param   Send length
 *
 * @return  Send status.
 */
bool ETH_TxPkt_ChainMode(uint16_t FrameLength)
{
    /* Check if the descriptor is owned by the ETHERNET DMA (when set) or CPU
     * (when reset) */
    if ((DMATxDescToSet->Status & ETH_DMATxDesc_OWN))
    {
        /* Return ERROR: OWN bit set */
        return false;
    }

    /* Setting the Frame Length: bits[12:0] */
    DMATxDescToSet->ControlBufferSize = (FrameLength & ETH_DMATxDesc_TBS1);
#ifdef CHECKSUM_BY_HARDWARE
    /* Setting the last segment and first segment bits (in this case a frame is
     * transmitted in one descriptor) */
    DMATxDescToSet->Status |= ETH_DMATxDesc_LS | ETH_DMATxDesc_FS | ETH_DMATxDesc_CIC_TCPUDPICMP_Full;
#else
    DMATxDescToSet->Status |= ETH_DMATxDesc_LS | ETH_DMATxDesc_FS;
#endif
    /* Set Own bit of the Tx descriptor Status: gives the buffer back to ETHERNET
     * DMA */
    DMATxDescToSet->Status |= ETH_DMATxDesc_OWN;

    /* When Tx Buffer unavailable flag is set: clear it and resume transmission */
    if ((ETHDMA->DMASR & ETH_DMASR_TBUS))
    {
        /* Clear TBUS ETHERNET DMA flag */
        ETHDMA->DMASR = ETH_DMASR_TBUS;
        /* Resume DMA transmission*/

        ETHDMA->DMATPDR = 0;
    }

    /* Update the ETHERNET DMA global Tx descriptor with next Tx decriptor */
    /* Chained Mode */
    /* Selects the next DMA Tx descriptor list for next buffer to send */
    DMATxDescToSet = (ETH_DMADESCTypeDef *)(DMATxDescToSet->Buffer2NextDescAddr);

    /* Return SUCCESS */
    return true;
}

/*
 *
 *
 */
void WCH_GetMacAddr(uint8_t *p)
{
    uint8_t i;
    uint8_t *macaddr = (uint8_t *)(ROM_CFG_USERADR_ID + 5);

    for (i = 0; i < 6; i++)
    {
        *p = *macaddr;
        p++;
        macaddr--;
    }
}
// EOF
