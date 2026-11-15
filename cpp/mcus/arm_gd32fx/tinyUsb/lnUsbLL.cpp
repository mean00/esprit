
#include "device/dcd.h"
#include "esprit.h"
#include "lnUSBD.h"
#include "lnUSDB_tinyUSB.h"
#include "lnUsbStack.h"
#include "tusb.h"

#include <new> // placement new for the .bss lnUsbDevice

#if CFG_TUSB_DEBUG != 0
#define USBD_EXTRA_STACK 500
#else
#define USBD_EXTRA_STACK 0
#endif
#define USB_BASE_STACK_SIZE 256
#define USBD_STACK_SIZE (USB_BASE_STACK_SIZE + USBD_EXTRA_STACK)
// local prototypes
void dcd_handle_bus_reset(void);
void dcd_ep_ctr_handler(void);
void dcd_ep_ctr_rx_handler(int ep);
void dcd_ep_ctr_tx_handler(int ep);
void cdc_task(void *params);
extern lnUsbDevice *_usbDevice;

//-----------------------------------------------------------------------------+
// The DCD event bridge: `lnUsbDevice`'s virtual event() -> tinyUSB's dcd_event_*.
//
// Moved here from lnUsbStack.cpp (usb.md, P2). It belongs to this layer: it adapts
// the register-level device driver below tinyUSB, and the stack above tinyUSB is
// Rust now, so nothing in the C++ stack is left to host it.
//-----------------------------------------------------------------------------+
lnUsbDevice *_usbDevice = NULL;

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
        }
    }
};

static const tusbEventHandler myEventHandler;

/**
 * Bring the device controller up and hook it to tinyUSB. Replaces the same work that
 * `lnUsbStack::init()` did on the C side (the object itself still lives in .bss: it is
 * created once, never destroyed, and has a virtual destructor, so a plain static
 * would drag newlib's exit machinery in - see malloc_used.md).
 */
extern "C" void lnUsbDcdInit(void)
{
    alignas(lnUsbDevice) static uint8_t usbDeviceStorage[sizeof(lnUsbDevice)];
    _usbDevice = new (usbDeviceStorage) lnUsbDevice(0);
    _usbDevice->init();
    _usbDevice->registerEventHandler(&myEventHandler);
}

void dcd_remote_wakeup(uint8_t rhport)
{
    (void)rhport;
    _usbDevice->wakeUpHost();
}

void dcd_int_disable(uint8_t rhport)
{
    _usbDevice->irqEnabled(false);
}
void dcd_int_enable(uint8_t rhport)
{
    _usbDevice->irqEnabled(true);
}

bool dcd_init(uint8_t rhport, const tusb_rhport_init_t *rh_init)
{
    // Send a reset
    _usbDevice->hwReset();
    return true;
}

void dcd_edpt_close_all(uint8_t rhport)
{
    (void)rhport;
}

// EOF
