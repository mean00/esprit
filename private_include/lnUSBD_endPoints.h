#pragma once
#define ENTRY_SIZE_T uint32_t

struct BTableEntry
{
    volatile ENTRY_SIZE_T TxAdr;
    volatile ENTRY_SIZE_T TxSize;
    volatile ENTRY_SIZE_T RxAdr;
    volatile ENTRY_SIZE_T RxSize;
};

/**
  The Setup of shared ram is  (from bottom)
              8*8 => EndpointBufferdescriptor
       0x40   64 bytes Rx buffer  // 128 bytes per endpoint
       0x40   64 bytes Tx buffer
*/
extern volatile uint32_t *aUSBD0_SRAM;
extern LN_USBD_Registers *aUSBD0;
static BTableEntry *btables = (BTableEntry *)LN_USBD0_RAM_ADR;
class EndPoints
{
  public:
    static int ep_bufferTail;
    static int ep_nbEp;
    class xfer_descriptor
    {
      public:
        uint8_t *buffer;
        uint16_t total_len;
        uint16_t queued_len;
        uint16_t pma_ptr;
        uint16_t max_packet_size;
        uint16_t pma_alloc_size;
    };

  public:
    static BTableEntry *getBTable(uint32_t ep)
    {
        BTableEntry *b = btables + ep;
        return b;
    }

    static xfer_descriptor *getDescriptor(uint32_t ep, uint32_t dir)
    {
        return (xfer_status + 2 * (ep & 0x7f) + (!!dir));
    }

    static void setRxBufferSize(uint32_t ep, uint32_t size)
    {
        uint32_t out;
        if (size > 62)
        {
            int nbBlock = (size + 31) / 32; // 32 bytes block
            out = (1 << 15) + (nbBlock << 10);
        }
        else
        {
            int nbBlock = (size + 1) / 2; // 2 bytes block
            out = nbBlock << 10;
        }
        BTableEntry *b = btables + ep;
        b->RxSize = out;
    }

    static uint32_t initEpRam(uint32_t address, uint32_t size)
    {
        int epnum = address & 0x7f;
        int dir = !!(address & 0x80);
        xfer_descriptor *desc = xfer_status + 2 * epnum + dir;
        // Reuse existing one
        if (desc->pma_alloc_size)
        {
            xAssert(size <= (desc->pma_alloc_size & 0x1ff)) return desc->pma_ptr;
        }
        int ret = ep_bufferTail;
        ep_bufferTail += size;
        xAssert(ep_bufferTail < 512);
        desc->pma_ptr = ret;
        desc->pma_alloc_size = size;
        ep_nbEp++;
        return ret;
    }
    static void freeEndpoint(uint32_t address)
    {
        xAssert(ep_nbEp > 2);
        ep_nbEp--;
        if (ep_nbEp == 2)
        {
            reset(true);
        }
    }
    static void reset(bool partial = false)
    {
        if (!partial)
            ep_nbEp = 2;
        ep_bufferTail = 8 * LN_USBD_MAX_ENDPOINT;
        int start = 0;
        if (partial)
            start = 2;

        xfer_descriptor *desc = xfer_status + start;
        for (int i = start; i < LN_USBD_MAX_ENDPOINT * 2; i++)
        {
            memset(desc, 0, sizeof(*desc));
            desc++;
        }
    }
    static void setBlock(uint32_t ep, bool isTx, uint32_t address, uint32_t size)
    {
        uint16_t *ptr = (uint16_t *)aUSBD0_SRAM;
        ptr += 8 * ep;
        if (!isTx)
        { // RX
            ptr += 4;
            // Plus we need to compute the size differently
            int r = size;
            if (size <= 62)
            {
                int n = ((size - 1) / 2);
                r |= n << 10; // # of 2 bytes block
            }
            else
            {
                int n = (size - 31) / 32;
                r |= (n << 10) + (1 << 15); // # of 32 bytes block
            }
            size = r;
        }
        ptr[0] = address;
        ptr[1] = size;
    }

  public:
    static xfer_descriptor xfer_status[LN_USBD_MAX_ENDPOINT * 2];
};
