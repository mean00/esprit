
#include "LN_RTT.h"
#include "esprit.h"
#include "ln-rtt_priv.h"
#include "string.h"

#define RTT_BUFFER_SIZE (256)

//
uint32_t buffer[RTT_BUFFER_SIZE >> 2];
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
    my_rtt.channel.buffer_addr = (uint32_t)buffer;
    my_rtt.channel.buffer_size = (uint32_t)RTT_BUFFER_SIZE;
    my_rtt.channel.read_offset = 0;
    my_rtt.channel.write_offset = 0;
    my_rtt.channel.flags = 0;
    my_rtt.header.max_num_up_buffers = 1;
    my_rtt.header.max_num_down_buffers = 0;
    memcpy(my_rtt.header.id, "SEGGER RTT", 11);
}
/*
 *
 *
 *
 */
uint32_t LN_RTT_Write(uint32_t bufferIndex, const uint8_t *buffer, uint32_t size)
{
    xAssert(bufferIndex == 0);
    lnRTTChannel *chan = &(my_rtt.channel);
    uint32_t write_offset = chan->write_offset;
    uint32_t done = 0;
    if (chan->write_offset >= chan->read_offset)
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
            if (write_offset >= chan->buffer_size)
            {
                write_offset -= chan->buffer_size;
            }
        }
    }
    // left part
    if (size)
    {
        int avail = chan->read_offset - write_offset;
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
        }
    }
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
