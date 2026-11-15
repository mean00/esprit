/**
  Minimalistic copy / paste of ch32 structure used
*/
#pragma once
#define PERIPH_BASE ((uint32_t)0x40000000) /* Peripheral base address in the alias region */
#define APB1PERIPH_BASE (PERIPH_BASE)
#define APB2PERIPH_BASE (PERIPH_BASE + 0x10000)
#define AHBPERIPH_BASE (PERIPH_BASE + 0x20000)

#define USBHS_BASE (AHBPERIPH_BASE + 0x3400)

typedef struct
{
    uint16_t MAX_LEN;
    uint16_t dummy;
} USBHS_LEN;

typedef struct
{
    uint16_t TX_LEN;
    uint8_t TX_CTRL;
    uint8_t RX_CTRL;
} USBHS_EP_CTRL;

typedef struct
{
    uint8_t CONTROL;
    uint8_t HOST_CTRL;
    uint8_t INT_EN;
    uint8_t DEV_AD;
    uint16_t FRAME_NO;
    uint8_t SUSPEND;
    uint8_t RESERVED0;
    uint8_t SPEED_TYPE;
    uint8_t MIS_ST;
    uint8_t INT_FG;
    uint8_t INT_ST;
    uint16_t RX_LEN;
    uint16_t RESERVED1;
    uint32_t ENDP_CONFIG;
    uint32_t ENDP_TYPE;
    uint32_t BUF_MODE;
    uint32_t UEP_0_DMA;
    uint32_t UEP_RX_DMA[15];
    uint32_t UEP_TX_DMA[15];
    USBHS_LEN EP_LENS[16];
    USBHS_EP_CTRL EP_CTRLS[16];
} LN_USBHSD_DEVICEx;

typedef volatile LN_USBHSD_DEVICEx LN_USBHSD_DEVICE;
