
#include "lnUsbStack.h"
#include "device/dcd.h"
#include "esprit.h"
#include "lnUSBD.h"
#include "lnUSDB_tinyUSB.h"
#include "tusb.h"

#if CFG_TUSB_DEBUG != 0
#define USBD_EXTRA_STACK 1024
#else
#define USBD_EXTRA_STACK 0
#endif
#define USB_BASE_STACK_SIZE 1500 // in bytes
#define USBD_STACK_SIZE (USB_BASE_STACK_SIZE + USBD_EXTRA_STACK)
// local prototypes
void dcd_handle_bus_reset(void);
void dcd_ep_ctr_handler(void);
void dcd_ep_ctr_rx_handler(int ep);
void dcd_ep_ctr_tx_handler(int ep);
void cdc_task(void *params);
lnUsbDevice *_usbDevice = NULL;
static lnUsbStack *lnUsbStackInstance = NULL;

//

//--------------------------------------------------------------------+
// Main
//--------------------------------------------------------------------+

class tusbEventHandler : public lnUsbEventHandler
{
  public:
    virtual void event(const lnUsbdEvent ev, const uint32_t endPoint) const
    {
        switch (ev)
        {
        case UsbReset: {
            tusb_speed_t speed = TUSB_SPEED_FULL;
#ifdef LN_USB_SPEED
            speed = LN_USB_SPEED;
#endif
            dcd_handle_bus_reset();

            dcd_event_bus_reset(0, speed, true);
        }
        break;
        case UsbResume:
            dcd_event_bus_signal(0, DCD_EVENT_RESUME, true);
            break;
        case UsbSuspend:
            dcd_event_bus_signal(0, DCD_EVENT_SUSPEND, true);
            break;
        case UsbTransferRxCompleted:
            dcd_ep_ctr_rx_handler(endPoint);
            break;
        case UsbTransferTxCompleted:
            dcd_ep_ctr_tx_handler(endPoint);
            break;
        default:
            xAssert(0);
            break;
        };
    }
};

static const tusbEventHandler myEventHandler;

/**

*/
lnUsbStack::lnUsbStack()
{
    _running = true;
    lnUsbStackInstance = this;
    _qualifier = NULL;
    _hs = NULL;
    _fs = NULL;
    _descDevice = NULL;
    _eventHandler = NULL;
    _cookie = NULL;
}
/**

*/
lnUsbStack::~lnUsbStack()
{
    lnUsbStackInstance = NULL;
}
/**
 * @brief <brief>
 * @param [in] <name> <parameter_description>
 * @return <return_description>
 * @details <details>
 */

void lnUsbStack::init(int nb, const char **desc)
{
    _usbDevice = new lnUsbDevice(0);
    _usbDevice->init();
    _usbDevice->registerEventHandler(&myEventHandler);
    _deviceDescriptor = desc;
    _nbDeviceDescriptor = nb;
}
static void tusbTrampoline(void *a)
{
    lnUsbStack *t = (lnUsbStack *)a;
    t->task();
}
/**

*/
void lnUsbStack::sendEvent(lnUsbStackEvents ev)
{
    if (!_eventHandler)
        return;
    _eventHandler(_cookie, ev);
}

/**

*/
void lnUsbStack::start()
{
    lnCreateTask(tusbTrampoline, "usbd", USBD_STACK_SIZE, this, configMAX_PRIORITIES - 1);
}
/**

*/
void lnUsbStack::stop()
{
    _running = false;
}
/**
 * @brief <brief>
 * @param [in] <name> <parameter_description>
 * @return <return_description>
 * @details <details>
 */

void lnUsbStack::task()
{
    tusb_init();
    while (_running)
    {
        tud_task();
    }
    vTaskDelete(NULL);
}

// Invoked when usb bus is suspended
// remote_wakeup_en : if host allow us  to perform remote wakeup
// Within 7ms, device must draw an average of current less than 2.5 mA from bus
/**
USB_CONNECT,
USB_DISCONNECT,
*/
void tud_suspend_cb(bool remote_wakeup_en)
{
    xAssert(lnUsbStackInstance);
    lnUsbStackInstance->sendEvent(lnUsbStack::USB_SUSPEND);
}

// Invoked when usb bus is resumed
void tud_resume_cb(void)
{
    xAssert(lnUsbStackInstance);
    lnUsbStackInstance->sendEvent(lnUsbStack::USB_RESUME);
}

void tud_mount_cb(void)
{
    xAssert(lnUsbStackInstance);
    lnUsbStackInstance->sendEvent(lnUsbStack::USB_CONNECT);
}

// Invoked when device is unmounted
void tud_umount_cb(void)
{
    xAssert(lnUsbStackInstance);
    lnUsbStackInstance->sendEvent(lnUsbStack::USB_DISCONNECT);
}
/**
 */
uint16_t const *tud_descriptor_string_cb(uint8_t index, uint16_t langid)
{
    xAssert(lnUsbStackInstance);
    return lnUsbStackInstance->getDeviceDescriptor(index);
}

// Invoked when received GET STRING DESCRIPTOR request
// Application return pointer to descriptor, whose contents must exist long enough for transfer to complete
#define MAX_STRING 64
static uint16_t _desc_str[MAX_STRING];
uint16_t const *lnUsbStack::getDeviceDescriptor(int index)
{

    xAssert(_deviceDescriptor);
    xAssert(_nbDeviceDescriptor);
    uint16_t *p = &(_desc_str[0]);
    if (!index)
    {
        *p++ = (TUSB_DESC_STRING << 8) | 4;
        memcpy(p, _deviceDescriptor[0], 2);
        return _desc_str;
    }
    // Note: the 0xEE index string is a Microsoft OS 1.0 Descriptors.
    // https://docs.microsoft.com/en-us/windows-hardware/drivers/usbcon/microsoft-defined-usb-descriptors
    if (index >= _nbDeviceDescriptor)
        return NULL;
    const char *str = _deviceDescriptor[index];
    // Cap at max char
    int mx = strlen(str);
    if (mx >= MAX_STRING)
        mx = MAX_STRING - 1;

    *p++ = (TUSB_DESC_STRING << 8) | (2 * mx + 2);
    for (int i = 0; i < mx; i++)
        *p++ = str[i];
    return _desc_str;
}
//--//

uint8_t const *tud_descriptor_other_speed_configuration_cb(uint8_t index)
{
    (void)index; // for multiple configurations
    return (tud_speed_get() == TUSB_SPEED_HIGH) ? lnUsbStackInstance->getFSConfiguration()
                                                : lnUsbStackInstance->getHSConfiguration();
}

// Invoked when received GET CONFIGURATION DESCRIPTOR
// Application return pointer to descriptor
// Descriptor contents must exist long enough for transfer to complete
uint8_t const *tud_descriptor_configuration_cb(uint8_t index)
{
    (void)index; // for multiple configurations
    return (tud_speed_get() == TUSB_SPEED_HIGH) ? lnUsbStackInstance->getHSConfiguration()
                                                : lnUsbStackInstance->getFSConfiguration();
}

// Invoked when received GET DEVICE QUALIFIER DESCRIPTOR request
// Application return pointer to descriptor, whose contents must exist long enough for transfer to complete.
// device_qualifier descriptor describes information about a high-speed capable device that would
// change if the device were operating at the other speed. If not highspeed capable stall this request.
uint8_t const *tud_descriptor_device_qualifier_cb(void)
{
    xAssert(lnUsbStackInstance);
    return (uint8_t *)lnUsbStackInstance->getQualifier();
}

uint8_t const *tud_descriptor_device_cb(void)
{
    xAssert(lnUsbStackInstance); // tusb_desc_device_t
    return (uint8_t const *)lnUsbStackInstance->getDeviceDescriptor();
}

// EOF
