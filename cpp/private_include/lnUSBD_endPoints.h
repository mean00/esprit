#pragma once
#define ENTRY_SIZE_T uint32_t

// Number of CDC interfaces the USB descriptors of this build instantiate (see
// lnBMP_usb_descriptor_2cdc.h / lnBMP_usb_descriptor_3cdc.h). Each one uses a
// notification, an OUT and an IN endpoint, so the highest endpoint row the
// hardware looks up is 2 * LN_USBD_CDC_INTERFACES.
#ifdef USE_3_CDC
#define LN_USBD_CDC_INTERFACES 3
#else
#define LN_USBD_CDC_INTERFACES 2
#endif

struct BTableEntry
{
    volatile ENTRY_SIZE_T TxAdr;
    volatile ENTRY_SIZE_T TxSize;
    volatile ENTRY_SIZE_T RxAdr;
    volatile ENTRY_SIZE_T RxSize;
};

/**
  The Setup of shared ram is  (from bottom)
       8*16   => EndpointBufferdescriptor : one BTableEntry per endpoint
                (the PMA only carries 16 bits per 32 bit slot, so each of the
                 4 fields of an endpoint takes 4 bytes)
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

    // Size, in the PMA addressing unit used by ep_bufferTail, of the buffer
    // descriptor table. A row is sizeof(BTableEntry) bytes - 4 x 32 bit slots
    // of which only 16 bits are used, see the note in lnUsbDevice::copyFromSRAM
    // - so the endpoint data buffers MUST start after the rows the descriptors
    // use. With the old 8 byte per endpoint assumption they started at byte 64,
    // i.e. inside the rows the descriptors use: on the 2-CDC build EP0's OUT
    // buffer is the first allocation (64..127) and so covered the rows of
    // EP4..EP7. Every control-OUT with a data phase - the line coding the host
    // sends when it opens a port - is written there, i.e. on top of EP4's
    // TxAdr/TxSize. EP4 is CDC1's data OUT/IN, the bridge/logger endpoint, so
    // the bridge read its address/count out of those host bytes: it transmitted
    // from whatever PMA offset they happened to encode (GDB reply bytes came out
    // on the log port) and its own data was written to a wrong PMA offset. EP4
    // is the only endpoint of the 2-CDC build affected, since EP5..EP7 are not
    // used - which is why the GDB channel and the target link kept working while
    // the log/bridge port was corrupted.
    static int btableSize()
    {
        // Rows 0..2*LN_USBD_CDC_INTERFACES (80 bytes with 2 CDC, 112 with 3).
        // The 3 CDC descriptors keep their data endpoints at 32 bytes because
        // the pool has to hold those rows + EP0 (2 x 64) + 3 x (8 + 32 + 32):
        // 112 + 128 + 216 = 456 of the 512 bytes of PMA.
        return (2 * LN_USBD_CDC_INTERFACES + 1) * (int)sizeof(BTableEntry);
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
        ep_bufferTail = btableSize();
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
