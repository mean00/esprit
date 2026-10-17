#include "include/lnUsbStack.h"
#include "esprit.h"
extern "C"
{
#include "lnUSB_c.h"
}

#define WRAPME(x) ((lnUsbStack *)x)

// this is done by the C side
const uint8_t *usb_hsConfiguration = NULL;
const uint8_t *usb_fsConfiguration = NULL;
const tusb_desc_device_t *usb_des_device = NULL;
const tusb_desc_device_qualifier_t *usb_qualifier = NULL;
const char **usb_descriptor = NULL;

int usb_descriptor_size = 0;

void lnusb_set_global_configuration(const uint8_t *hsConfiguration, const uint8_t *fsConfiguration,
                                    const tusb_desc_device_t *desc, const tusb_desc_device_qualifier_t *qual,
                                    int desc_size, const char **udesc);

void lnusb_set_global_configuration(const uint8_t *hsConfiguration, const uint8_t *fsConfiguration,
                                    const tusb_desc_device_t *desc, const tusb_desc_device_qualifier_t *qual,
                                    int udesc_size, const char **udesc)
{
    usb_hsConfiguration = hsConfiguration;
    usb_fsConfiguration = fsConfiguration;
    usb_des_device = desc;
    usb_qualifier = qual;
    usb_descriptor_size = udesc_size;
    usb_descriptor = udesc;
}

/**
 */
lnusb_c *lnusb_create(uint32_t instance)
{
    xAssert(usb_descriptor);
    xAssert(usb_descriptor_size);

    lnUsbStack *s = new lnUsbStack();
    s->init(usb_descriptor_size, (const char **)usb_descriptor);
    return (lnusb_c *)s;
}
/**
 */
void lnusb_delete(lnusb_c *c)
{
    delete WRAPME(c);
    c = NULL;
}
/**
 */
void lnusb_init(lnusb_c *inst, int nbDescriptorLine, const char **deviceDescriptor)
{
    WRAPME(inst)->init(nbDescriptorLine, deviceDescriptor);
}
/**
 */
void lnusb_setConfiguration(lnusb_c *inst)
{
    xAssert(usb_hsConfiguration);
    xAssert(usb_fsConfiguration);
    xAssert(usb_des_device);
    xAssert(usb_qualifier);

    WRAPME(inst)->setConfiguration(usb_hsConfiguration, usb_fsConfiguration, usb_des_device, usb_qualifier);
}
/**
 */
void lnusb_setEventHandler(lnusb_c *inst, const void *cookie, lnUsbStackEventHandler *const ev)
{
    lnUsbStack::lnUsbStackEventHandler *ev_c = (lnUsbStack::lnUsbStackEventHandler *)ev;

    WRAPME(inst)->setEventHandler((void *)cookie, ev_c);
}
/**
 */
void lnusb_start(lnusb_c *inst)
{

    WRAPME(inst)->start();
}
/**
 */
void lnusb_stop(lnusb_c *inst)
{
    WRAPME(inst)->stop();
}

#include "tusb.h"

bool lnusb_is_mounted(void)
{
    return tud_mounted();
}

bool lnusb_is_suspended(void)
{
    return tud_suspended();
}

bool lnusb_is_ready(void)
{
    return tud_ready();
}

// EOF