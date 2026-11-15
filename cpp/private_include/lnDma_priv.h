/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */

#pragma once

struct DMA_channelsx
{
    uint32_t CTL;   // 8 0
    uint32_t CNT;   // c 4
    uint32_t PADDR; // 10 8
    uint32_t MADDR; // 14 C
    uint32_t empty;
};

typedef volatile DMA_channelsx DMA_channels;

struct DMA_structx
{
    uint32_t INTF;
    uint32_t INTC;

    DMA_channels channels[6];
};

typedef volatile DMA_structx DMA_struct;

#define LN_DMA_CHAN_ENABLE (1 << 0)
#define LN_DMA_CHAN_TFTFIE (1 << 1)    // transfer interrupt enable
#define LN_DMA_CHAN_HTFIE (1 << 2)     // half interrupt enable
#define LN_DMA_CHAN_ERRIE (1 << 3)     // error interrupt enable
#define LN_DMA_CHAN_DIR_P2M (0 << 4)   // peripharl -> memory
#define LN_DMA_CHAN_DIR_M2P (1 << 4)   // memory -> peripheral
#define LN_DMA_CHAN_CMEN (1 << 5)      // circular
#define LN_DMA_CHAN_PINCREASE (1 << 6) // circular
#define LN_DMA_CHAN_MINCREASE (1 << 7) // circular

#define PWIDTH_x(x) ((x) << 8)
#define MWIDTH_x(x) ((x) << 10)

#define LN_DMA_CHAN_PWIDTH_8 PWIDTH_x(0)
#define LN_DMA_CHAN_PWIDTH_16 PWIDTH_x(1)
#define LN_DMA_CHAN_PWIDTH_32 PWIDTH_x(2)
#define LN_DMA_CHAN_MWIDTH_8 MWIDTH_x(0)
#define LN_DMA_CHAN_MWIDTH_16 MWIDTH_x(1)
#define LN_DMA_CHAN_MWIDTH_32 MWIDTH_x(2)

#define LN_DMA_CHAN_PRIO_LOW (0 << 12)   // circular
#define LN_DMA_CHAN_PRIO_MED (1 << 12)   // circular
#define LN_DMA_CHAN_PRIO_HIGH (2 << 12)  // circular
#define LN_DMA_CHAN_PRIO_UHIGH (3 << 12) // circular

#define LN_DMA_CHAN_MEM (1 << 14) // circular

#define LN_DMA_CHAN_KEEP_MASK 0xffff8000U

#define LN_DMA_CHAN_WIDTH_MASK (~(PWIDTH_x(3) | MWIDTH_x(3)))
#define LN_DMA_CHAN_ALL_INTERRUPT_MASK (LN_DMA_CHAN_TFTFIE + LN_DMA_CHAN_HTFIE + LN_DMA_CHAN_ERRIE)