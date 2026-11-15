/**
 * @file ln_rp_dma_priv.h
 * @author mean00
 * @brief simple dma
 * @version 0.1
 * @date 2023-11-01
 *
 * @copyright Copyright (c) 2023
 *
 */

#pragma once
#include "esprit.h"
#include "ln_rp_memory_map.h"
/**
 * @brief describe one channel
 *
 */
struct LN_RP_DMA_channelx
{
    uint32_t DMA_READ;    // read address pointer
    uint32_t DMA_WRITE;   // dma write address
    uint32_t DMA_COUNT;   // # of transfer
    uint32_t DMA_CONTROL; // Control/setup
};

struct LN_RP_DMAx
{
    uint32_t INTR;  // interrupt status
    uint32_t INTE0; // int enable for IRQ 0
    uint32_t INTF0; // int force for IRQ 0
    uint32_t INTS0; // int status for IRQ 0, write 1 to clear the interrupt
    uint32_t dummy[1];
    uint32_t INTE1; // int enable for IRQ 1
    uint32_t INTF1; // int force for IRQ 1
    uint32_t INTS1; // int status for IRQ 1
};

typedef volatile LN_RP_DMA_channelx LN_RP_DMA_channel;
typedef volatile LN_RP_DMAx LN_RP_DMA;

#define RP_DMA_CHANNEL(x) (LN_RP_DMA_channel *)(LN_RP_DMA_CHANNEL_BASE + x * 0x40)

// --  Control register --
#define LN_RP_DMA_CONTROL_ENABLE (1 << 0)    // enable
#define LN_RP_DMA_CONTROL_HIGH_PRIO (1 << 1) // high prio
#define LN_RP_DMA_CONTROL_SET_DATA_SIZE_8 (0 << 2)
#define LN_RP_DMA_CONTROL_SET_DATA_SIZE_16 (1 << 2)
#define LN_RP_DMA_CONTROL_SET_DATA_SIZE_32 (2 << 2)
#define LN_RP_DMA_CONTROL_SET_DATA_SIZE_MASK (3 << 2)
#define LN_RP_DMA_CONTROL_INCR_READ (1 << 4)
#define LN_RP_DMA_CONTROL_DECR_READ (1 << 5) // if 1 decrement rathe than increment
#define LN_RP_DMA_CONTROL_INCR_WRITE (1 << 6)
#define LN_RP_DMA_CONTROL_DEX_WRITE (1 << 7) // if 1, decrement instead of increment
#define LN_RP_DMA_CONTROL_CIRCULAR_SIZE(pow_of_2) ((pow_of_2) << 8)
#define LN_RP_DMA_CONTROL_CIRCULAR_READ (0 << 12)
#define LN_RP_DMA_CONTROL_CIRCULAR_WRITE (1 << 12)
#define LN_RP_DMA_CONTROL_CHAIN_TO(x) (x << 13)

#define LN_RP_DMA_CONTROL_TREQ(x) (x << 17)
// quiet ignored
// bswp ignored
// sniff ignored
#define LN_RP_DMA_CONTROL_IS_BUSY_MASK (1 << 26)
#define LN_RP_DMA_CONTROL_IS_ERROR_MASK (7 << 29)

#define LN_RP_DMA_CHANNEL_COUNT 16

// EOF
