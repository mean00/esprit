/**
 * @file ln_rp_spi_priv.h
 * @author mean00
 * @brief
 * @version 0.1
 * @date 2023-12-26
 *
 * @copyright Copyright (c) 2023
 *
 */
#pragma once
#include "ln_rp_dma.h"
/**
 */
class LN_RP_SPIx
{
  public:
    uint32_t CR0;   // 0x00
    uint32_t CR1;   // 0x04
    uint32_t DR;    // 0x08  data
    uint32_t SR;    // 0x0C  status register
    uint32_t CPSR;  // 0x10  clock prescaler
    uint32_t IMSC;  // 0x14  interrupt mask
    uint32_t RIS;   // 0x18  raw interrup status
    uint32_t MIS;   // 0x1C  masked status
    uint32_t ICR;   // 0x20  interrupt clear
    uint32_t DMACR; // 0x24
};

typedef volatile LN_RP_SPIx LN_RP_SPI;

#define LN_RP_SPI_CR0_DIVIDER(x) ((uint32_t)x << 8U)
#define LN_RP_SPI_CR0_DIVIDER_MASK (0xff << 8)
#define LN_RP_SPI_CR0_MODE(x) ((uint32_t)x << 6U) // PHASE  / POLARITY, motoroal only
#define LN_RP_SPI_CR0_FORMAT_MOTOROLA (0 << 4)
#define LN_RP_SPI_CR0_FORMAT_TI (1 << 4)
#define LN_RP_SPI_CR0_FORMAT_NL (2 << 4)
#define LN_RP_SPI_CR0_8_BITS (7 << 0)
#define LN_RP_SPI_CR0_16_BITS (0xf << 0)
#define LN_RP_SPI_CR0_BITS_MASK (0xf)
//
#define LN_RP_SPI_CR1_ENABLE (1 << 1)
// Status
#define LN_RP_SPI_SR_TFE (1U << 0U) // Tx empty
#define LN_RP_SPI_SR_TFN (1U << 1U) // tx not full
#define LN_RP_SPI_SR_RNE (1U << 2U) // rx not empty
#define LN_RP_SPI_SR_RFF (1U << 3U) // rx full
#define LN_RP_SPI_SR_BSY (1U << 4U) // busy

// interrupt mask set
#define LN_RP_SPI_INT_OVR (1 << 0) // overrun RORIM
#define LN_RP_SPI_INT_TIM (1 << 1) // timeout RTIM
#define LN_RP_SPI_INT_RX (1 << 2)  // receive
#define LN_RP_SPI_INT_TX (1 << 3)  // transmit

// interrupt raw status

// Masked interrup tstatus status
// both same as mask set

// Interrupt clear register
#define LN_RP_SPI_ICR_OVR (1 << 0) // overrun
#define LN_RP_SPI_ICR_TIM (1 << 1) // timeout

// DMA
#define LN_RP_SPI_DMACR_RX (1 << 0) //
#define LN_RP_SPI_DMACR_TX (1 << 1) //

//----