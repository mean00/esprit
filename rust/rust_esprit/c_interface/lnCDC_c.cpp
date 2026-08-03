
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
 * Non-blocking: writes only what the TX FIFO accepts right now.
 */
int lncdc_write_noblock(lncdc_c *instance, const uint8_t *buffer, int maxSize)
{
    return WRAPME(instance)->writeNoBlock(buffer, maxSize);
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
void lncdc_set_event_handler(lncdc_c *instance, cdc_event_handler *handler, void *cookie)
{
    lnUsbCDC::lnUsbCDCEventsHandler *e_handler = (lnUsbCDC::lnUsbCDCEventsHandler *)handler;
    return WRAPME(instance)->setEventHandler(e_handler, cookie);
}

// EOF