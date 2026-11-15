
#pragma once
#include "stdint.h"
/*
 *
 */
typedef struct
{
    uint8_t id[16]; // SEGGER RTT
    uint32_t max_num_up_buffers;
    uint32_t max_num_down_buffers;
} lnRTTHeader;
/*
 *
 */
typedef struct
{
    uint32_t name_addr; // pointer to name
    uint32_t buffer_addr;
    uint32_t buffer_size;
    uint32_t write_offset;
    uint32_t read_offset;
    uint32_t flags;
} lnRTTChannel;

typedef struct
{
    lnRTTHeader header;
    lnRTTChannel channel;
} MY_RTT_DESC;
