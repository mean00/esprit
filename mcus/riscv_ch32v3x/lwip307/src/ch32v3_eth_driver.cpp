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
#define PHY_ADDRESS 0x01

#define USE10BASE_T
#ifndef USE10BASE_T
//    #define USE_GIGA_MAC
#ifndef USE_GIGA_MAC
#define USE_FAST_MAC
// #define USE_RMII
#endif
#endif

__attribute__((aligned(4))) ETH_DMADESCTypeDef DMARxDscrTab[ETH_RXBUFNB];      /* 接收描述符表 */
__attribute__((aligned(4))) ETH_DMADESCTypeDef DMATxDscrTab[ETH_TXBUFNB];      /* 发送描述符表 */
__attribute__((aligned(4))) uint8_t Rx_Buff[ETH_RXBUFNB][ETH_MAX_PACKET_SIZE]; /* 接收队列 */
__attribute__((aligned(4))) uint8_t Tx_Buff[ETH_TXBUFNB][ETH_MAX_PACKET_SIZE]; /* 发送队列 */

/* extern variable */
extern void Ethernet_LED_LINKSET(uint8_t setbit);
extern void Ethernet_LED_DATASET(uint8_t setbit);
extern void enable_eth_irq(void);
/* Macro */
#ifndef ETH_ERROR
#define ETH_ERROR ((uint32_t)0)
#endif

#ifndef ETH_SUCCESS
#define ETH_SUCCESS ((uint32_t)1)
#endif

#define ETH_DMARxDesc_FrameLengthShift 16
/*
 *  It is assumed ETh clock has been set already
 *  And PPL3 is running at 60 Mhz
 */
void ch32v3_init_phy(void)
{

    Logger("CH32Vxx Init Phy.\n");

    Ethernet_LED_LINKSET(1); /* turn off link led. */
    Ethernet_LED_DATASET(1); /* turn off data led. */

#ifdef USE10BASE_T
    /* Enable internal 10BASE-T PHY*/
    EXTEN->EXTEN_CTR |= EXTEN_ETH_10M_EN; /* 使能10M以太网物理层   */
#endif

#ifdef USE_GIGA_MAC
    /* Enable 1G MAC*/
    EXTEN->EXTEN_CTR |= EXTEN_ETH_RGMII_SEL;       /* 使能1G以太网MAC */
    RCC_ETH1GCLKConfig(RCC_ETH1GCLKSource_PB1_IN); /* 选择外部125MHz输入 */
    RCC_ETH1G_125Mcmd(ENABLE);                     /* 使能125MHz时钟 */

    /*  Enable RGMII GPIO */
    GETH_pin_init();
#endif

#ifdef USE_FAST_MAC
    /*  Enable MII or RMII GPIO */
    FETH_pin_init();
#endif

    /* Software reset */
    ETH_SoftwareReset();

    /* Ethernet_Configuration */
    static ETH_InitTypeDef ETH_InitStructure = {0};
    static uint32_t timeout;

    /* Wait for software reset */
    timeout = 10;
    // OS_SUBNT_SET_STATE();
    if (ETH->DMABMR & ETH_DMABMR_SR)
    {
        timeout--;
        if (timeout == 0)
        {
            Logger("Error:Eth soft-reset timeout!\nPlease check RGMII TX & RX clock "
                   "line. \r\n");
        }
        vTaskDelay(100 / portTICK_PERIOD_MS);
    }

    /* ETHERNET Configuration
     * ------------------------------------------------------*/
    /* Call ETH_StructInit if you don't like to configure all ETH_InitStructure
     * parameter */
    ETH_StructInit(&ETH_InitStructure);
    /* Fill ETH_InitStructure parametrs */
    /*------------------------   MAC   -----------------------------------*/
    ETH_InitStructure.ETH_Mode = ETH_Mode_FullDuplex;
    ETH_InitStructure.ETH_Speed = ETH_Speed_1000M;
    ETH_InitStructure.ETH_AutoNegotiation = ETH_AutoNegotiation_Enable;
    ETH_InitStructure.ETH_LoopbackMode = ETH_LoopbackMode_Disable;
    ETH_InitStructure.ETH_RetryTransmission = ETH_RetryTransmission_Disable;
    ETH_InitStructure.ETH_AutomaticPadCRCStrip = ETH_AutomaticPadCRCStrip_Disable;
    ETH_InitStructure.ETH_ReceiveAll = ETH_ReceiveAll_Enable;
    ETH_InitStructure.ETH_BroadcastFramesReception = ETH_BroadcastFramesReception_Enable;
    ETH_InitStructure.ETH_PromiscuousMode = ETH_PromiscuousMode_Enable;
    ETH_InitStructure.ETH_MulticastFramesFilter = ETH_MulticastFramesFilter_Perfect;
    ETH_InitStructure.ETH_UnicastFramesFilter = ETH_UnicastFramesFilter_Perfect;
#ifdef CHECKSUM_BY_HARDWARE
    ETH_InitStructure.ETH_ChecksumOffload = ETH_ChecksumOffload_Enable;
#endif
    /*------------------------   DMA   -----------------------------------*/
    /* When we use the Checksum offload feature, we need to enable the Store and
    Forward mode: the store and forward guarantee that a whole frame is stored in
    the FIFO, so the MAC can insert/verify the checksum, if the checksum is OK the
    DMA can handle the frame otherwise the frame is dropped */
    ETH_InitStructure.ETH_DropTCPIPChecksumErrorFrame = ETH_DropTCPIPChecksumErrorFrame_Enable;
    ETH_InitStructure.ETH_ReceiveStoreForward = ETH_ReceiveStoreForward_Enable;
    ETH_InitStructure.ETH_TransmitStoreForward = ETH_TransmitStoreForward_Enable;
    ETH_InitStructure.ETH_ForwardErrorFrames = ETH_ForwardErrorFrames_Enable;
    ETH_InitStructure.ETH_ForwardUndersizedGoodFrames = ETH_ForwardUndersizedGoodFrames_Enable;
    ETH_InitStructure.ETH_SecondFrameOperate = ETH_SecondFrameOperate_Disable;
    ETH_InitStructure.ETH_AddressAlignedBeats = ETH_AddressAlignedBeats_Enable;
    ETH_InitStructure.ETH_FixedBurst = ETH_FixedBurst_Enable;
    ETH_InitStructure.ETH_RxDMABurstLength = ETH_RxDMABurstLength_32Beat;
    ETH_InitStructure.ETH_TxDMABurstLength = ETH_TxDMABurstLength_32Beat;
    ETH_InitStructure.ETH_DMAArbitration = ETH_DMAArbitration_RoundRobin_RxTx_2_1;

    /* Configure Ethernet */
    uint32_t tmpreg = 0;
    static uint16_t RegValue = 0;

    /*---------------------- 物理层配置 -------------------*/
    /* 置SMI接口时钟 ，置为主频的42分频  */
    tmpreg = ETH->MACMIIAR;
    tmpreg &= MACMIIAR_CR_MASK;
    tmpreg |= (uint32_t)ETH_MACMIIAR_CR_Div42;
    ETH->MACMIIAR = (uint32_t)tmpreg;

    /* 复位物理层 */
    ETH_WritePHYRegister(PHY_ADDRESS, PHY_BCR, PHY_Reset); /* 复位物理层  */

    vTaskDelay(100 / portTICK_PERIOD_MS);

    timeout = 10000; /* 最大超时十秒   */
    RegValue = 0;

    RegValue = ETH_ReadPHYRegister(PHY_ADDRESS, PHY_BCR);
    if ((RegValue & (PHY_Reset)))
    {
        timeout--;
        if (timeout <= 0)
        {
            Logger("Error:Wait phy software timeout!\nPlease cheak PHY/MID.\nProgram "
                   "has been "
                   "blocked!\n");
            while (1)
                ;
        }
        vTaskDelay(1000 / portTICK_PERIOD_MS);
    }

    /* 等待物理层与对端建立LINK */
    timeout = 10000; /* 最大超时十秒   */
    RegValue = 0;

    RegValue = ETH_ReadPHYRegister(PHY_ADDRESS, PHY_BSR);
    if ((RegValue & (PHY_Linked_Status)) == 0)
    {
        timeout--;
        if (timeout <= 0)
        {
            Logger("Error:Wait phy linking timeout!\nPlease cheak MID.\nProgram has "
                   "been blocked!\n");
            while (1)
                ;
        }
        vTaskDelay(1000 / portTICK_PERIOD_MS);
    }

    /* 等待物理层完成自动协商 */
    timeout = 10000; /* 最大超时十秒   */
    RegValue = 0;

    RegValue = ETH_ReadPHYRegister(PHY_ADDRESS, PHY_BSR);
    if ((RegValue & PHY_AutoNego_Complete) == 0)
    {
        timeout--;
        if (timeout <= 0)
        {
            Logger("Error:Wait phy auto-negotiation complete timeout!\nPlease cheak "
                   "MID.\nProgram has "
                   "been blocked!\n");
            while (1)
                ;
        }

        vTaskDelay(1000 / portTICK_PERIOD_MS);
    }

    RegValue = ETH_ReadPHYRegister(PHY_ADDRESS, 0x10);
    Logger("PHY_SR value:%04x. \r\n", RegValue);

    if (RegValue & (1 << 2))
    {
        ETH_InitStructure.ETH_Mode = ETH_Mode_FullDuplex;
        Logger("Full Duplex. \r\n");
    }
    else
    {
        ETH_InitStructure.ETH_Mode = ETH_Mode_HalfDuplex;
        Logger("Half Duplex. \r\n");
    }
    ETH_InitStructure.ETH_Speed = ETH_Speed_10M;
    if (RegValue & (1 << 3))
    {
        Logger("Loopback_10M \r\n");
    }
    else {}

    vTaskDelay(10 / portTICK_PERIOD_MS);

    /* 点亮link状态led灯 */
    Ethernet_LED_LINKSET(0);

    /*------------------------ MAC寄存器配置  -----------------------
     * --------------------*/
    /* MACCCR在RGMII接口模式下具有调整RGMII接口时序的域，请注意 */
    tmpreg = ETH->MACCR;
    tmpreg &= MACCR_CLEAR_MASK;
    tmpreg |=
        (uint32_t)(ETH_InitStructure.ETH_Watchdog | ETH_InitStructure.ETH_Jabber | ETH_InitStructure.ETH_InterFrameGap |
                   ETH_InitStructure.ETH_CarrierSense | ETH_InitStructure.ETH_Speed | ETH_InitStructure.ETH_ReceiveOwn |
                   ETH_InitStructure.ETH_LoopbackMode | ETH_InitStructure.ETH_Mode |
                   ETH_InitStructure.ETH_ChecksumOffload | ETH_InitStructure.ETH_RetryTransmission |
                   ETH_InitStructure.ETH_AutomaticPadCRCStrip | ETH_InitStructure.ETH_BackOffLimit |
                   ETH_InitStructure.ETH_DeferralCheck);
    /* 写MAC控制寄存器 */
    ETH->MACCR = (uint32_t)tmpreg;
#ifdef USE10BASE_T
    ETH->MACCR |= ETH_Internal_Pull_Up_Res_Enable; /* 启用内部上拉  */
#endif
    ETH->MACFFR = (uint32_t)(ETH_InitStructure.ETH_ReceiveAll | ETH_InitStructure.ETH_SourceAddrFilter |
                             ETH_InitStructure.ETH_PassControlFrames | ETH_InitStructure.ETH_BroadcastFramesReception |
                             ETH_InitStructure.ETH_DestinationAddrFilter | ETH_InitStructure.ETH_PromiscuousMode |
                             ETH_InitStructure.ETH_MulticastFramesFilter | ETH_InitStructure.ETH_UnicastFramesFilter);
    /*--------------- ETHERNET MACHTHR and MACHTLR Configuration ---------------*/
    /* Write to ETHERNET MACHTHR */
    ETH->MACHTHR = (uint32_t)ETH_InitStructure.ETH_HashTableHigh;
    /* Write to ETHERNET MACHTLR */
    ETH->MACHTLR = (uint32_t)ETH_InitStructure.ETH_HashTableLow;
    /*----------------------- ETHERNET MACFCR Configuration --------------------*/
    /* Get the ETHERNET MACFCR value */
    tmpreg = ETH->MACFCR;
    /* Clear xx bits */
    tmpreg &= MACFCR_CLEAR_MASK;

    tmpreg |= (uint32_t)((ETH_InitStructure.ETH_PauseTime << 16) | ETH_InitStructure.ETH_ZeroQuantaPause |
                         ETH_InitStructure.ETH_PauseLowThreshold | ETH_InitStructure.ETH_UnicastPauseFrameDetect |
                         ETH_InitStructure.ETH_ReceiveFlowControl | ETH_InitStructure.ETH_TransmitFlowControl);
    ETH->MACFCR = (uint32_t)tmpreg;

    ETH->MACVLANTR = (uint32_t)(ETH_InitStructure.ETH_VLANTagComparison | ETH_InitStructure.ETH_VLANTagIdentifier);

    tmpreg = ETH->DMAOMR;
    /* Clear xx bits */
    tmpreg &= DMAOMR_CLEAR_MASK;

    tmpreg |= (uint32_t)(ETH_InitStructure.ETH_DropTCPIPChecksumErrorFrame | ETH_InitStructure.ETH_ReceiveStoreForward |
                         ETH_InitStructure.ETH_FlushReceivedFrame | ETH_InitStructure.ETH_TransmitStoreForward |
                         ETH_InitStructure.ETH_TransmitThresholdControl | ETH_InitStructure.ETH_ForwardErrorFrames |
                         ETH_InitStructure.ETH_ForwardUndersizedGoodFrames |
                         ETH_InitStructure.ETH_ReceiveThresholdControl | ETH_InitStructure.ETH_SecondFrameOperate);
    ETH->DMAOMR = (uint32_t)tmpreg;

    ETH->DMABMR =
        (uint32_t)(ETH_InitStructure.ETH_AddressAlignedBeats | ETH_InitStructure.ETH_FixedBurst |
                   ETH_InitStructure.ETH_RxDMABurstLength | /* !! if 4xPBL is selected for Tx
                                                               or Rx it is applied for the
                                                               other */
                   ETH_InitStructure.ETH_TxDMABurstLength | (ETH_InitStructure.ETH_DescriptorSkipLength << 2) |
                   ETH_InitStructure.ETH_DMAArbitration | ETH_DMABMR_USP);

    /* Enable the Ethernet Rx Interrupt */
    ETH_DMAITConfig(ETH_DMA_IT_NIS | ETH_DMA_IT_R | ETH_DMA_IT_T, ENABLE);

    enable_eth_irq();
    // NVIC_EnableIRQ(ETH_IRQn);
    ETH_DMATxDescChainInit(DMATxDscrTab, &Tx_Buff[0][0], ETH_TXBUFNB);
    ETH_DMARxDescChainInit(DMARxDscrTab, &Rx_Buff[0][0], ETH_RXBUFNB);
    ETH_Start();
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
    if ((DMARxDescToGet->Status & ETH_DMARxDesc_OWN) != (uint32_t)RESET)
    {
        frame.length = ETH_ERROR;
        if ((ETH->DMASR & ETH_DMASR_RBUS) != (uint32_t)RESET)
        {
            /* Clear RBUS ETHERNET DMA flag */
            ETH->DMASR = ETH_DMASR_RBUS;
            /* Resume DMA reception */
            ETH->DMARPDR = 0;
        }
        Logger("Error:ETH_DMARxDesc_OWN.\r\n");
        /* Return error: OWN bit set */
        return frame;
    }

    if (((DMARxDescToGet->Status & ETH_DMARxDesc_ES) == (uint32_t)RESET) &&
        ((DMARxDescToGet->Status & ETH_DMARxDesc_LS) != (uint32_t)RESET) &&
        ((DMARxDescToGet->Status & ETH_DMARxDesc_FS) != (uint32_t)RESET))
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
        framelength = ETH_ERROR;
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
uint32_t ETH_TxPkt_ChainMode(uint16_t FrameLength)
{
    /* Check if the descriptor is owned by the ETHERNET DMA (when set) or CPU
     * (when reset) */
    if ((DMATxDescToSet->Status & ETH_DMATxDesc_OWN) != (uint32_t)RESET)
    {
        /* Return ERROR: OWN bit set */
        return ETH_ERROR;
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
    if ((ETH->DMASR & ETH_DMASR_TBUS) != (uint32_t)RESET)
    {
        /* Clear TBUS ETHERNET DMA flag */
        ETH->DMASR = ETH_DMASR_TBUS;
        /* Resume DMA transmission*/

        ETH->DMATPDR = 0;
    }

    /* Update the ETHERNET DMA global Tx descriptor with next Tx decriptor */
    /* Chained Mode */
    /* Selects the next DMA Tx descriptor list for next buffer to send */
    DMATxDescToSet = (ETH_DMADESCTypeDef *)(DMATxDescToSet->Buffer2NextDescAddr);

    /* Return SUCCESS */
    return ETH_SUCCESS;
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
    if (ETH->DMASR & ETH_DMA_IT_R)
    {
        ETH_DMAClearITPendingBit(ETH_DMA_IT_R);
        ln_ethernet_rx();
    }
    if (ETH->DMASR & ETH_DMA_IT_T)
    {
        ETH_DMAClearITPendingBit(ETH_DMA_IT_T);
    }

    ETH_DMAClearITPendingBit(ETH_DMA_IT_NIS);
}

// EOF
