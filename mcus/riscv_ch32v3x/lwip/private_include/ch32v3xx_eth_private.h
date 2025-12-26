#pragma once
#define __IO volatile
#include "stdint.h"
typedef struct
{
    __IO uint32_t MACCR;     // 0x00
    __IO uint32_t MACFFR;    // 0x04
    __IO uint32_t MACHTHR;   // 0x08
    __IO uint32_t MACHTLR;   // 0x0c
    __IO uint32_t MACMIIAR;  // 0x10
    __IO uint32_t MACMIIDR;  // 0x14
    __IO uint32_t MACFCR;    // 0x18
    __IO uint32_t MACVLANTR; // 0x1c
    uint32_t RESERVED0[2];
    __IO uint32_t MACRWUFFR; // 0x28
    __IO uint32_t MACPMTCSR; // 0x2C
    uint32_t RESERVED1[2];
    __IO uint32_t MACSR;                    // 0x38
    __IO uint32_t MACIMR;                   // 0x3c
    __IO uint32_t MACA0HR;                  // 0x40
    __IO uint32_t MACA0LR;                  // 0x44
    __IO uint32_t MACA1HR;                  // 0x48
    __IO uint32_t MACA1LR;                  // 0x4c
    __IO uint32_t MACA2HR;                  // 0x50
    __IO uint32_t MACA2LR;                  // 0x54
    __IO uint32_t MACA3HR;                  // 0x58
    __IO uint32_t MACA3LR;                  // 0x5c
    uint32_t RESERVED2[14];                 // 0x60+14*4= 0x98
    __IO uint32_t MACCFG0;                  // 0x98
    uint32_t RESERVED3[(0x100 - 0x9c) / 4]; // 0x60+14*4= 0x98
} ETHMAC_TypeDef;
typedef struct
{
    __IO uint32_t DMABMR; // 0x100
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
} ETHDMA_TypeDef;
typedef struct
{
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
} ETHMMC_TypeDef;
typedef struct
{
    __IO uint32_t PTPTSCR;
    __IO uint32_t PTPSSIR;
    __IO uint32_t PTPTSHR;
    __IO uint32_t PTPTSLR;
    __IO uint32_t PTPTSHUR;
    __IO uint32_t PTPTSLUR;
    __IO uint32_t PTPTSAR;
    __IO uint32_t PTPTTHR;
    __IO uint32_t PTPTTLR;
} ETHPTP_TypeDef;

#define PERIPH_BASE ((uint32_t)0x40000000)
#define AHBPERIPH_BASE (PERIPH_BASE + 0x20000)
#define ETH_BASE (AHBPERIPH_BASE + 0x8000)
//
#ifdef USE_CH32V208W
#define ETH_MAC_BASE (ETH_BASE)
// NOT SUPPORTED #define ETH_MMC_BASE (ETH_BASE + 0x0100)
// NOT SUPPORTED #define ETH_PTP_BASE (ETH_BASE + 0x0700)
#define ETH_DMA_BASE (ETH_BASE + 0x100)
#else
#define ETH_MAC_BASE (ETH_BASE)
#define ETH_MMC_BASE (ETH_BASE + 0x0100)
#define ETH_PTP_BASE (ETH_BASE + 0x0700)
#define ETH_DMA_BASE (ETH_BASE + 0x1000)
#endif
#define EXTEN_BASE (0x40000000 + 0x20000 + 0x3800)

typedef struct
{
    __IO uint32_t EXTEN_CTR;
} EXTEN_TypeDef;

#define EXTEN ((EXTEN_TypeDef *)EXTEN_BASE)
#define EXTEN_ETH_10M_EN ((uint32_t)0x00000004) /* Bit 2 */

extern volatile ETHMAC_TypeDef *ETH;
extern volatile ETHDMA_TypeDef *ETHDMA;
#ifndef USE_CH32V208W
extern volatile ETHMMC_TypeDef *ETHMMC;
extern volatile ETHPTP_TypeDef *ETHPTP;
#endif
