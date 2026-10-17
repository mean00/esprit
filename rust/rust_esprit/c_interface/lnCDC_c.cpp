
#include "include/lnUsbCDC.h"
#include "include/lnUsbStack.h"
#include "esprit.h"
extern "C"
{
#include "lnCDC_c.h"
}
#define WRAPME(x) ((lnUsbCDC *)x)
/*

*/
lncdc_c *lncdc_create(uint32_t instance)
{
    return (lncdc_c *)(new lnUsbCDC((int)instance));
}

/*

*/
void lncdc_delete(lncdc_c *instance)
{
    delete WRAPME(instance);
    instance = NULL;
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