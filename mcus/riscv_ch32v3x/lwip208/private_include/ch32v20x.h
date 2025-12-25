/********************************** (C) COPYRIGHT  *******************************
 * File Name          : ch32v20x.h
 * Author             : WCH
 * Version            : V1.0.0
 * Date               : 2021/06/06
 * Description        : CH32V20x Device Peripheral Access Layer Header File.
 * Copyright (c) 2021 Nanjing Qinheng Microelectronics Co., Ltd.
 * SPDX-License-Identifier: Apache-2.0
 *******************************************************************************/
#pragma once


/******************************************************************************/
/*                                  ETH10M                                    */
/******************************************************************************/
/* ETH register */
#define R8_ETH_EIE              (*((volatile uint8_t *)(0x40028000+3))) /* Interrupt Enable Register */
#define  RB_ETH_EIE_INTIE       0x80                  /* RW interrupt enable*/
#define  RB_ETH_EIE_RXIE        0x40                  /* RW Receive complete interrupt enable */
#define  RB_ETH_EIE_LINKIE      0x10                  /* RW Link Change Interrupt Enable */
#define  RB_ETH_EIE_TXIE        0x08                  /* RW send complete interrupt enable */
#define  RB_ETH_EIE_R_EN50      0x04                  /* RW TX 50�� resistor adjustment. 1: On-chip 50�� connected 0: On-chip 50�� disconnected */
#define  RB_ETH_EIE_TXERIE      0x02                  /* RW Transmit Error Interrupt Enable */
#define  RB_ETH_EIE_RXERIE      0x01                  /* RW1 receive error flag */
#define R8_ETH_EIR              (*((volatile uint8_t *)(0x40028000+4))) /* Interrupt Flag Register */
#define  RB_ETH_EIR_RXIF        0x40                  /* RW1 Receive complete flag */
#define  RB_ETH_EIR_LINKIF      0x10                  /* RW1 Link Change Flag */
#define  RB_ETH_EIR_TXIF        0x08                  /* RW1 Link Change Flag */
#define  RB_ETH_EIR_TXERIF      0x02                  /* RW1 send error flag */
#define  RB_ETH_EIR_RXERIF      0x01                  /* RW1 receive error flag */
#define R8_ETH_ESTAT            (*((volatile uint8_t *)(0x40028000+5))) /* status register */
#define  RB_ETH_ESTAT_INT       0x80                  /* RW1 interrupt */
#define  RB_ETH_ESTAT_BUFER     0x40                  /* RW1 Buffer error */
#define  RB_ETH_ESTAT_RXCRCER   0x20                  /* RO receive crc error */
#define  RB_ETH_ESTAT_RXNIBBLE  0x10                  /* RO receives nibble error */
#define  RB_ETH_ESTAT_RXMORE    0x08                  /* RO receives more than maximum packets */
#define  RB_ETH_ESTAT_RXBUSY    0x04                  /* RO receive busy */
#define  RB_ETH_ESTAT_TXABRT    0x02                  /* RO send interrupted by mcu */
#define R8_ETH_ECON2            (*((volatile uint8_t *)(0x40028000+6))) /* ETH PHY Analog Block Control Register */
#define  RB_ETH_ECON2_RX        0x0E                  /* 011b must be written */
#define  RB_ETH_ECON2_TX        0x01
#define  RB_ETH_ECON2_MUST      0x06                  /* 011b must be written */
#define R8_ETH_ECON1            (*((volatile uint8_t *)(0x40028000+7))) /* Transceiver Control Register */
#define  RB_ETH_ECON1_TXRST     0x80                  /* RW Send module reset */
#define  RB_ETH_ECON1_RXRST     0x40                  /* RW Receiver module reset */
#define  RB_ETH_ECON1_TXRTS     0x08                  /* RW The transmission starts, and it is automatically cleared after the transmission is completed. */
#define  RB_ETH_ECON1_RXEN      0x04                  /* RW Receive is enabled, when cleared, the error flag RXERIF will change to 1 if it is receiving */

#define R32_ETH_TX              (*((volatile uint32_t *)(0x40028000+8))) /* send control */
#define R16_ETH_ETXST           (*((volatile uint16_t *)(0x40028000+8))) /* RW Send DMA buffer start address */
#define R16_ETH_ETXLN           (*((volatile uint16_t *)(0x40028000+0xA))) /* RW send length */
#define R32_ETH_RX              (*((volatile uint32_t *)(0x40028000+0xC))) /* receive control */
#define R16_ETH_ERXST           (*((volatile uint16_t *)(0x40028000+0xC))) /* RW Receive DMA buffer start address */
#define R16_ETH_ERXLN           (*((volatile uint16_t *)(0x40028000+0xE))) /* RO receive length */

#define R32_ETH_HTL             (*((volatile uint32_t *)(0x40028000+0x10)))
#define R8_ETH_EHT0             (*((volatile uint8_t *)(0x40028000+0x10))) /* RW Hash Table Byte0 */
#define R8_ETH_EHT1             (*((volatile uint8_t *)(0x40028000+0x11))) /* RW Hash Table Byte1 */
#define R8_ETH_EHT2             (*((volatile uint8_t *)(0x40028000+0x12))) /* RW Hash Table Byte2 */
#define R8_ETH_EHT3             (*((volatile uint8_t *)(0x40028000+0x13))) /* RW Hash Table Byte3 */
#define R32_ETH_HTH             (*((volatile uint32_t *)(0x40028000+0x14)))
#define R8_ETH_EHT4             (*((volatile uint8_t *)(0x40028000+0x14))) /* RW Hash Table Byte4 */
#define R8_ETH_EHT5             (*((volatile uint8_t *)(0x40028000+0x15))) /* RW Hash Table Byte5 */
#define R8_ETH_EHT6             (*((volatile uint8_t *)(0x40028000+0x16))) /* RW Hash Table Byte6 */
#define R8_ETH_EHT7             (*((volatile uint8_t *)(0x40028000+0x17))) /* RW Hash Table Byte7 */

#define R32_ETH_MACON           (*((volatile uint32_t *)(0x40028000+0x18)))
#define R8_ETH_ERXFCON          (*((volatile uint8_t *)(0x40028000+0x18))) /* Received Packet Filtering Control Register */
/* RW 0=Do not enable this filter condition, 1=When ANDOR=1,
target address mismatch will be filtered, when ANDOR=0, target address match will be accepted */
#define  RB_ETH_ERXFCON_UCEN    0x80
#define  RB_ETH_ERXFCON_CRCEN   0x20
#define  RB_ETH_ERXFCON_EN      0x10
#define  RB_ETH_ERXFCON_MPEN    0x08
#define  RB_ETH_ERXFCON_HTEN    0x04
#define  RB_ETH_ERXFCON_MCEN    0x02
#define  RB_ETH_ERXFCON_BCEN    0x01
#define R8_ETH_MACON1           (*((volatile uint8_t *)(0x40028000+0x19))) /* Mac flow control registers */
/* RW When FULDPX=0 is invalid, when FULDPX=1, 11=send 0 timer pause frame,
then stop sending, 10=send pause frame periodically, 01=send pause frame once, then stop sending, 00=stop sending pause frame */
#define  RB_ETH_MACON1_FCEN     0x30
#define  RB_ETH_MACON1_TXPAUS   0x08                  /* RW Send pause frame enable*/
#define  RB_ETH_MACON1_RXPAUS   0x04                  /* RW Receive pause frame enable */
#define  RB_ETH_MACON1_PASSALL  0x02                  /* RW 1=Unfiltered control frames will be written to the buffer, 0=Control frames will be filtered */
#define  RB_ETH_MACON1_MARXEN   0x01                  /* RW MAC layer receive enable */
#define R8_ETH_MACON2           (*((volatile uint8_t *)(0x40028000+0x1A))) /* Mac Layer Packet Control Register */
#define  RB_ETH_MACON2_PADCFG   0xE0                  /* RW Short Packet Padding Settings */
#define  RB_ETH_MACON2_TXCRCEN  0x10                  /* RW Send to add crc, if you need to add crc in PADCFG, this position is 1 */
#define  RB_ETH_MACON2_PHDREN   0x08                  /* RW Special 4 bytes do not participate in crc check */
#define  RB_ETH_MACON2_HFRMEN   0x04                  /* RW Allow jumbo frames */
#define  RB_ETH_MACON2_FULDPX   0x01                  /* RW full duplex */
#define R8_ETH_MABBIPG          (*((volatile uint8_t *)(0x40028000+0x1B))) /* Minimum Interpacket Interval Register */
#define  RB_ETH_MABBIPG_MABBIPG 0x7F                  /* RW Minimum number of bytes between packets */

#define R32_ETH_TIM             (*((volatile uint32_t *)(0x40028000+0x1C)))
#define R16_ETH_EPAUS           (*((volatile uint16_t *)(0x40028000+0x1C))) /* RW Flow Control Pause Frame Time Register */
#define R16_ETH_MAMXFL          (*((volatile uint16_t *)(0x40028000+0x1E))) /* RW Maximum Received Packet Length Register */
#define R16_ETH_MIRD            (*((volatile uint16_t *)(0x40028000+0x20))) /* RW MII read data register */

#define R32_ETH_MIWR            (*((volatile uint32_t *)(0x40028000+0x24)))
#define R8_ETH_MIREGADR         (*((volatile uint8_t *)(0x40028000+0x24))) /* MII address register*/
#define  RB_ETH_MIREGADR_MASK   0x1F                  /* RW PHY register address mask */
#define R8_ETH_MISTAT           (*((volatile uint8_t *)(0x40028000+0x25))) /* RW PHY register address mask */
//#define  RB_ETH_MIREGADR_MIIWR  0x20                  /* WO MII write command */
#define R16_ETH_MIWR            (*((volatile uint16_t *)(0x40028000+0x26))) /* WO MII Write Data Register */
#define R32_ETH_MAADRL          (*((volatile uint32_t *)(0x40028000+0x28))) /* RW MAC 1-4 */
#define R8_ETH_MAADRL1          (*((volatile uint8_t *)(0x40028000+0x28))) /* RW MAC 1 */
#define R8_ETH_MAADRL2          (*((volatile uint8_t *)(0x40028000+0x29))) /* RW MAC 2 */
#define R8_ETH_MAADRL3          (*((volatile uint8_t *)(0x40028000+0x2A))) /* RW MAC 3 */
#define R8_ETH_MAADRL4          (*((volatile uint8_t *)(0x40028000+0x2B))) /* RW MAC 4 */
#define R16_ETH_MAADRH          (*((volatile uint16_t *)(0x40028000+0x2C))) /* RW MAC 5-6 */
#define R8_ETH_MAADRL5          (*((volatile uint8_t *)(0x40028000+0x2C))) /* RW MAC 4 */
#define R8_ETH_MAADRL6          (*((volatile uint8_t *)(0x40028000+0x2D))) /* RW MAC 4 */

//PHY address
#define PHY_BMCR                0x00                                            /* Control Register */
#define PHY_BMSR                0x01                                            /* Status Register */
#define PHY_ANAR                0x04                                            /* Auto-Negotiation Advertisement Register */
#define PHY_ANLPAR              0x05                                            /* Auto-Negotiation Link Partner Base  Page Ability Register*/
#define PHY_ANER                0x06                                            /* Auto-Negotiation Expansion Register */
#define PHY_MDIX                0x1e                                            /* Custom MDIX Mode Register */
//Custom MDIX Mode Register  @PHY_MDIX
#define PN_NORMAL               0x04                                            /* Analog p, n polarity selection */
#define MDIX_MODE_MASK          0x03                                            /* mdix settings */
#define MDIX_MODE_AUTO          0x00                                            /*  */
#define MDIX_MODE_MDIX          0x01
#define MDIX_MODE_MDI           0x02
//ECON2 test mode, to be determined
#define RX_VCM_MODE_0
#define RX_VCM_MODE_1
#define RX_VCM_MODE_2
#define RX_VCM_MODE_3
//RX reference voltage value setting  @RX_REF
#define RX_REF_25mV             (0<<2)                                          /* 25mV */
#define RX_REF_49mV             (1<<2)                                          /* 49mV */
#define RX_REF_74mV             (2<<2)                                          /* 74mV */
#define RX_REF_98mV             (3<<2)                                          /* 98mV */
#define RX_REF_123mV            (4<<2)                                          /* 123mV */
#define RX_REF_148mV            (5<<2)                                          /* 148mV */
#define RX_REF_173mV            (6<<2)                                          /* 173mV */
#define RX_REF_198mV            (7<<2)                                          /* 198mV */
//TX DRIVER Bias Current  @TX_AMP
#define TX_AMP_0                (0<<0)                                          /* 43mA   / 14.5mA   (1.4V/0.7V) */
#define TX_AMP_1                (1<<0)                                          /* 53.1mA / 18mA     (1.8V/0.9V) */
#define TX_AMP_2                (2<<0)                                          /* 75.6mA / 25.6mA   (2.6V/1.3V) */
#define TX_AMP_3                (3<<0)                                          /* 122mA  / 41.45mA  (4.1V/2.3V) */
//FCEN pause frame control      @FCEN
#define FCEN_0_TIMER            (3<<4)                                          /* Send a 0 timer pause frame, then stop sending */
#define FCEN_CYCLE              (2<<4)                                          /* Periodically send pause frames */
#define FCEN_ONCE               (1<<4)                                          /* Send pause frame once, then stop sending */
#define FCEN_STOP               (0<<4)                                          /* Stop sending pause frames */
//PADCFG short packet control  @PADCFG
#define PADCFG_AUTO_0           (7<<5)                                          /* All short packets are filled with 00h to 64 bytes, then 4 bytes crc */
#define PADCFG_NO_ACT_0         (6<<5)                                          /* No padding for short packets */
/* The detected VLAN network packet whose field is 8100h is automatically filled
with 00h to 64 bytes, otherwise the short packet is filled with 60 bytes of 0, and then 4 bytes of crc after filling */
#define PADCFG_DETE_AUTO        (5<<5)
#define PADCFG_NO_ACT_1         (4<<5)                                          /* No padding for short packets */
#define PADCFG_AUTO_1           (3<<5)                                          /* All short packets are filled with 00h to 64 bytes, then 4 bytes crc */
#define PADCFG_NO_ACT_2         (2<<5)                                          /* No padding for short packets */
#define PADCFG_AUTO_3           (1<<5)                                          /* All short packets are filled with 00h to 60 bytes, and then 4 bytes crc */
#define PADCFG_NO_ACT_3         (0<<5)                                          /* No padding for short packets */

/* Bit or field definition for PHY basic status register */
#define PHY_Linked_Status       ((uint16_t)0x0004)      /* Valid link established */

#define PHY_Reset                               ((uint16_t)0x8000)      /* PHY Reset */

#define PHY_AutoNego_Complete                   ((uint16_t)0x0020)      /* Auto-Negotioation process completed */

//MII control
#define  RB_ETH_MIREGADR_MIIWR  0x20                                            /* WO MII write command */
#define  RB_ETH_MIREGADR_MIRDL  0x1f                                            /* RW PHY register address */



