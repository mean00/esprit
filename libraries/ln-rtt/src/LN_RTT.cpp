
#include "LN_RTT.h"
#include "esprit.h"
#include "lnBarrier.h"
#include "ln-rtt_priv.h"
#include "string.h"

#define RTT_BUFFER_SIZE (1024)
#define RTT_FLAGS_MODE_NO_BLOCK_TRIM (1) // == SEGGER_RTT_MODE_NO_BLOCK_TRIM: write as much as fits, never block

//
uint32_t rtt_buffer[RTT_BUFFER_SIZE >> 2];
// #define my_rtt _SEGGER_RTT
//
static const char channel_name[] = "Logger";
//
extern MY_RTT_DESC my_rtt;

/*
 *
 *
 */
void LN_RTT_Init(void)
{
    my_rtt.channel.name_addr = (uint32_t)channel_name;
    my_rtt.channel.buffer_addr = (uint32_t)rtt_buffer;
    my_rtt.channel.buffer_size = (uint32_t)RTT_BUFFER_SIZE;
    my_rtt.channel.read_offset = 0;
    my_rtt.channel.write_offset = 0;
    my_rtt.channel.flags = RTT_FLAGS_MODE_NO_BLOCK_TRIM; // trim mode: matches LN_RTT_Write() behavior (never blocks)
    my_rtt.header.max_num_up_buffers = 1;
    my_rtt.header.max_num_down_buffers = 0;
    memcpy(my_rtt.header.id, "SEGGER RTT", 11);
}
/*
 *
 *
 *
 *  Concurrency contract:
 *  - LN_RTT_Write() is NOT internally locked. The caller must guarantee a single
 *    producer at a time. Logger_chars() serializes every logging producer
 *    (Logger, Logger_C, Rust logger!) through loggerMutex before reaching here.
 *  - The debug probe is the only consumer and only ever advances read_offset;
 *    producer vs consumer is a safe single-writer/single-reader pattern.
 *  - Non-blocking: if there is not enough free space, only what fits is written
 *    (trim mode) and the remainder is dropped. Returns the number of bytes written.
 */
uint32_t LN_RTT_Write(uint32_t bufferIndex, const uint8_t *buffer, uint32_t size)
{
    xAssert(bufferIndex == 0);
    volatile lnRTTChannel *chan = &(my_rtt.channel);
    uint32_t write_offset = chan->write_offset;
    uint32_t read_offset = chan->read_offset;
    uint32_t done = 0;
    // clamp the size to the available size
    uint32_t total_avail = (read_offset + chan->buffer_size - write_offset - 1) & (chan->buffer_size - 1);
    if (!total_avail)
    {
        return 0;
    }
    if (size > total_avail)
    {
        size = total_avail;
    }

    //
    if (chan->write_offset >= read_offset)
    {
        // right part
        int avail = chan->buffer_size - write_offset;
        int chunk = size;
        if (chunk > avail)
        {
            chunk = avail;
        }
        if (chunk)
        {
            memcpy((uint8_t *)(chan->buffer_addr + write_offset), buffer, chunk);
            buffer += chunk;
            size -= chunk;
            write_offset += chunk;
            done += chunk;
            write_offset &= chan->buffer_size - 1;
        }
    }
    // left part
    if (size)
    {
        int avail = read_offset - write_offset;
        int chunk = size;
        if (chunk > avail)
        {
            chunk = avail;
        }
        if (chunk)
        {
            memcpy((uint8_t *)(chan->buffer_addr + write_offset), buffer, chunk);
            buffer += chunk;
            size -= chunk;
            write_offset += chunk;
            done += chunk;
            write_offset &= chan->buffer_size - 1;
        }
    }
    asm volatile("" ::: "memory"); // compiler barrier
    LN_DATA_BARRIER();             // make the data writes globally visible before we publish write_offset
    chan->write_offset = write_offset;
    return done;
}
/*
 *
 */
void rttLoggerFunction(int n, const char *data)
{
    LN_RTT_Write(0, (const uint8_t *)data, n);
}
// EOF
