#pragma once
#define __IO volatile
#include "stdint.h"
typedef struct
{
    __IO uint32_t MACCR;
    __IO uint32_t MACFFR;
    __IO uint32_t MACHTHR;
    __IO uint32_t MACHTLR;
    __IO uint32_t MACMIIAR;
    __IO uint32_t MACMIIDR;
    __IO uint32_t MACFCR;
    __IO uint32_t MACVLANTR;
    uint32_t RESERVED0[2];
    __IO uint32_t MACRWUFFR;
    __IO uint32_t MACPMTCSR;
    uint32_t RESERVED1[2];
    __IO uint32_t MACSR;
    __IO uint32_t MACIMR;
    __IO uint32_t MACA0HR;
    __IO uint32_t MACA0LR;
    __IO uint32_t MACA1HR;
    __IO uint32_t MACA1LR;
    __IO uint32_t MACA2HR;
    __IO uint32_t MACA2LR;
    __IO uint32_t MACA3HR;
    __IO uint32_t MACA3LR;
    uint32_t RESERVED2[14];
    __IO uint32_t MACCFG0;
    uint32_t RESERVED10[25];
    __IO uint32_t MMCCR;
    __IO uint32_t MMCRIR;
    __IO uint32_t MMCTIR;
    __IO uint32_t MMCRIMR;
    __IO uint32_t MMCTIMR;
    uint32_t RESERVED3[14];
    __IO uint32_t MMCTGFSCCR;
    __IO uint32_t MMCTGFMSCCR;
    uint32_t RESERVED4[5];
    __IO uint32_t MMCTGFCR;
    uint32_t RESERVED5[10];
    __IO uint32_t MMCRFCECR;
    __IO uint32_t MMCRFAECR;
    uint32_t RESERVED6[10];
    __IO uint32_t MMCRGUFCR;
    uint32_t RESERVED7[334];
    __IO uint32_t PTPTSCR;
    __IO uint32_t PTPSSIR;
    __IO uint32_t PTPTSHR;
    __IO uint32_t PTPTSLR;
    __IO uint32_t PTPTSHUR;
    __IO uint32_t PTPTSLUR;
    __IO uint32_t PTPTSAR;
    __IO uint32_t PTPTTHR;
    __IO uint32_t PTPTTLR;
    uint32_t RESERVED8[567];
    __IO uint32_t DMABMR;
    __IO uint32_t DMATPDR;
    __IO uint32_t DMARPDR;
    __IO uint32_t DMARDLAR;
    __IO uint32_t DMATDLAR;
    __IO uint32_t DMASR;
    __IO uint32_t DMAOMR;
    __IO uint32_t DMAIER;
    __IO uint32_t DMAMFBOCR;
    uint32_t RESERVED9[9];
    __IO uint32_t DMACHTDR;
    __IO uint32_t DMACHRDR;
    __IO uint32_t DMACHTBAR;
    __IO uint32_t DMACHRBAR;
} ETH_TypeDef;

#define PERIPH_BASE ((uint32_t)0x40000000)
#define AHBPERIPH_BASE (PERIPH_BASE + 0x20000)
#define ETH_BASE (AHBPERIPH_BASE + 0x8000)
#define ETH_MAC_BASE (ETH_BASE)
#define ETH_MMC_BASE (ETH_BASE + 0x0100)
#define ETH_PTP_BASE (ETH_BASE + 0x0700)
#define ETH_DMA_BASE (ETH_BASE + 0x1000)
#define EXTEN_BASE (AHBPERIPH_BASE + 0x3800)

typedef struct
{
    __IO uint32_t EXTEN_CTR;
} EXTEN_TypeDef;

#define EXTEN ((EXTEN_TypeDef *)EXTEN_BASE)

#define EXTEN_USBD_PU_EN ((uint32_t)0x00000002)    /* Bit 1 */
#define EXTEN_ETH_10M_EN ((uint32_t)0x00000004)    /* Bit 2 */
#define EXTEN_ETH_RGMII_SEL ((uint32_t)0x00000008) /* Bit 3 */
#define EXTEN_PLL_HSI_PRE ((uint32_t)0x00000010)   /* Bit 4 */
#define EXTEN_LOCKUP_EN ((uint32_t)0x00000040)     /* Bit 5 */
#define EXTEN_LOCKUP_RSTF ((uint32_t)0x00000080)   /* Bit 7 */

#define EXTEN_ULLDO_TRIM ((uint32_t)0x00000300)  /* ULLDO_TRIM[1:0] bits */
#define EXTEN_ULLDO_TRIM0 ((uint32_t)0x00000100) /* Bit 0 */
#define EXTEN_ULLDO_TRIM1 ((uint32_t)0x00000200) /* Bit 1 */

#define EXTEN_LDO_TRIM ((uint32_t)0x00000C00)  /* LDO_TRIM[1:0] bits */
#define EXTEN_LDO_TRIM0 ((uint32_t)0x00000400) /* Bit 0 */
#define EXTEN_LDO_TRIM1 ((uint32_t)0x00000800) /* Bit 1 */

extern volatile ETH_TypeDef *ETH;
