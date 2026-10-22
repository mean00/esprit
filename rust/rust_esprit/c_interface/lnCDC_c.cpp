
#include "include/lnUsbCDC.h"
#include "include/lnUsbStack.h"
#include "esprit.h"
extern "C"
{
#include "lnCDC_c.h"
}
#include <new> // placement new for the .bss CDC objects

#define WRAPME(x) ((lnUsbCDC *)x)

/* One CDC object per USB interface, and they are permanent: the constructor
 * registers `this` in lnUsbCDC.cpp's `_cdc_instances[]`, which the TinyUSB
 * callbacks dereference for as long as the probe runs, so they live in .bss
 * instead of the heap (malloc_used.md §8.1 B5).  `instance` is the interface
 * number, so it is also the slot.  The count mirrors `MAX_CDC` of
 * esprit/libraries/tinyUsb/lnSrc/lnUsbCDC.cpp (its ctor asserts it). */
#define LN_CDC_SLOTS 4

static_assert(LN_CDC_SLOTS >= 3, "the probe can expose GDB + bridge + logger CDCs");

alignas(lnUsbCDC) static uint8_t _cdcStorage[LN_CDC_SLOTS][sizeof(lnUsbCDC)];
static bool _cdcUsed[LN_CDC_SLOTS];

/*
 *
 */
lncdc_c *lncdc_create(uint32_t instance)
{
    if (instance >= LN_CDC_SLOTS || _cdcUsed[instance])
    {
        xAssert(0);
        return NULL;
    }
    _cdcUsed[instance] = true;
    return (lncdc_c *)new (_cdcStorage[instance]) lnUsbCDC((int)instance);
}

/*
 *
 */
void lncdc_delete(lncdc_c *instance)
{
    if (!instance)
        return;
    for (uint32_t i = 0; i < LN_CDC_SLOTS; i++)
    {
        if ((const void *)_cdcStorage[i] == (const void *)instance)
        {
            ((lnUsbCDC *)instance)->~lnUsbCDC();
            _cdcUsed[i] = false;
            return;
        }
    }
}

/*

*/
int lncdc_read(lncdc_c *instance, uint8_t *buffer, int maxSize)
{
    return WRAPME(instance)->read(buffer, maxSize);
}

/*

*/
int lncdc_write(lncdc_c *instance, const uint8_t *buffer, int maxSize)
{
    return WRAPME(instance)->write(buffer, maxSize);
}

/*

*/
void lncdc_flush(lncdc_c *instance)
{
    return WRAPME(instance)->flush();
}

/*

*/
void lncdc_clear_input_buffers(lncdc_c *instance)
{
    return WRAPME(instance)->clear_input_buffers();
}

/*

*/
int lncdc_write_no_block(lncdc_c *instance, const uint8_t *buffer, int maxSize)
{
    return WRAPME(instance)->writeNoBlock(buffer, maxSize);
}

int lncdc_write_available(lncdc_c *instance)
{
    return WRAPME(instance)->writeAvailable();
}

int lncdc_read_available(lncdc_c *instance)
{
    return WRAPME(instance)->readAvailable();
}

bool lncdc_is_connected(lncdc_c *instance)
{
    return WRAPME(instance)->isConnected();
}

bool lncdc_peek(lncdc_c *instance, uint8_t *ch)
{
    return WRAPME(instance)->peek(ch);
}

uint8_t lncdc_get_line_state(lncdc_c *instance)
{
    return WRAPME(instance)->getLineState();
}

void lncdc_set_event_handler(lncdc_c *instance, cdc_event_handler *handler, void *cookie)
{
    lnUsbCDC::lnUsbCDCEventsHandler *e_handler = (lnUsbCDC::lnUsbCDCEventsHandler *)handler;
    return WRAPME(instance)->setEventHandler(e_handler, cookie);
}

// Direct port-level operations
#include "tusb.h"

bool lncdc_port_is_connected(uint32_t itf)
{
    return tud_cdc_n_connected(itf);
}

int lncdc_port_read_available(uint32_t itf)
{
    return (int)tud_cdc_n_available(itf);
}

int lncdc_port_write_available(uint32_t itf)
{
    return (int)tud_cdc_n_write_available(itf);
}

int lncdc_port_read(uint32_t itf, uint8_t *buffer, int maxSize)
{
    return (int)tud_cdc_n_read(itf, buffer, (uint32_t)maxSize);
}

int lncdc_port_write_no_block(uint32_t itf, const uint8_t *buffer, int maxSize)
{
    return (int)tud_cdc_n_write(itf, buffer, (uint32_t)maxSize);
}

int lncdc_port_write(uint32_t itf, const uint8_t *buffer, int size)
{
    int sent = 0;
    while (size)
    {
        int n = (int)tud_cdc_n_write(itf, buffer, (uint32_t)size);
        if (n < 0)
            return -1;
        if (!n)
        {
            tud_cdc_n_write_flush(itf);
            lnDelay(1);
            continue;
        }
        buffer += n;
        size -= n;
        sent += n;
    }
    return sent;
}

void lncdc_port_flush(uint32_t itf)
{
    while (tud_cdc_n_write_flush(itf) != 0)
    {
    }
}

void lncdc_port_clear_input(uint32_t itf)
{
    tud_cdc_n_read_flush(itf);
}

uint8_t lncdc_port_get_line_state(uint32_t itf)
{
    return tud_cdc_n_get_line_state(itf);
}

bool lncdc_port_peek(uint32_t itf, uint8_t *ch)
{
    return tud_cdc_n_peek(itf, ch);
}

// EOF