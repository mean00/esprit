// MEANX
#include "ch32v30x_usb.h"
/*
 */
#include "dcd_usbfs_platform.h"
/* USBOTG_FS Registers */
typedef struct
{
    uint8_t BASE_CTRL;
    uint8_t UDEV_CTRL;
    uint8_t INT_EN;
    uint8_t DEV_ADDR;
    uint8_t Reserve0;
    uint8_t MIS_ST;
    uint8_t INT_FG;
    uint8_t INT_ST;
    uint16_t RX_LEN;
    uint16_t Reserve1;
    uint8_t UEP4_1_MOD;
    uint8_t UEP2_3_MOD;
    uint8_t UEP5_6_MOD;
    uint8_t UEP7_MOD;
    uint32_t UEP0_DMA;
    uint32_t UEP1_DMA;
    uint32_t UEP2_DMA;
    uint32_t UEP3_DMA;
    uint32_t UEP4_DMA;
    uint32_t UEP5_DMA;
    uint32_t UEP6_DMA;
    uint32_t UEP7_DMA;
    uint16_t UEP0_TX_LEN;
    uint8_t UEP0_TX_CTRL;
    uint8_t UEP0_RX_CTRL;
    uint16_t UEP1_TX_LEN;
    uint8_t UEP1_TX_CTRL;
    uint8_t UEP1_RX_CTRL;
    uint16_t UEP2_TX_LEN;
    uint8_t UEP2_TX_CTRL;
    uint8_t UEP2_RX_CTRL;
    uint16_t UEP3_TX_LEN;
    uint8_t UEP3_TX_CTRL;
    uint8_t UEP3_RX_CTRL;
    uint16_t UEP4_TX_LEN;
    uint8_t UEP4_TX_CTRL;
    uint8_t UEP4_RX_CTRL;
    uint16_t UEP5_TX_LEN;
    uint8_t UEP5_TX_CTRL;
    uint8_t UEP5_RX_CTRL;
    uint16_t UEP6_TX_LEN;
    uint8_t UEP6_TX_CTRL;
    uint8_t UEP6_RX_CTRL;
    uint16_t UEP7_TX_LEN;
    uint8_t UEP7_TX_CTRL;
    uint8_t UEP7_RX_CTRL;
    uint32_t Reserve2;
    uint32_t OTG_CR;
    uint32_t OTG_SR;
} USBFSD_TypeDefx;
typedef volatile USBFSD_TypeDefx USBFSD_TypeDef;
#define USBOTG_BASE 0x50000000UL
USBFSD_TypeDef *USBOTG_FS = ((USBFSD_TypeDef *)USBOTG_BASE);
