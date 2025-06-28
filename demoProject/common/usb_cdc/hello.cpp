#include "hello.h"
#include "include/lnUsbCDC.h"
#include "include/lnUsbStack.h"
#include "lnArduino.h"

#define LED LN_SYSTEM_LED

#if defined(USE_RP2040) || defined(USE_RP2350)
#define LED2 GPIO10
#define LED3 GPIO11
#else
#define LED2 PB8
#define LED3 PB9
#endif
/**
 */
void setup()
{
    lnPinMode(LED, lnOUTPUT);
    lnPinMode(LED2, lnOUTPUT);
    lnPinMode(LED3, lnOUTPUT);
}
#define MEVENT(x)                                                                                                      \
    case lnUsbStack::USB_##x:                                                                                          \
        Logger(#x);                                                                                                    \
        break;

void helloUsbEvent(void *cookie, lnUsbStack::lnUsbStackEvents event)
{
    switch (event)
    {
        MEVENT(CONNECT)
        MEVENT(DISCONNECT)
        MEVENT(SUSPEND)
        MEVENT(RESUME)
    default:
        xAssert(0);
        break;
    }
}
lnUsbCDC *cdc = NULL;
uint8_t buffer[100];
lnFastEventGroup *event_group;
void cdcEventHandler(void *cookie, int interface, lnUsbCDC::lnUsbCDCEvents event, uint32_t payload)
{
    event_group->setEvents(1 << event);
}

void loop()
{
    Logger("Starting lnTUSB Test\n");

    event_group = new lnFastEventGroup();
    event_group->takeOwnership();

    lnUsbStack *usb = new lnUsbStack;
    usb->init(5, descriptor);
    usb->setConfiguration(desc_hs_configuration, desc_fs_configuration, &desc_device, &desc_device_qualifier);
    usb->setEventHandler(NULL, helloUsbEvent);

    cdc = new lnUsbCDC(0);
    cdc->setEventHandler(cdcEventHandler, NULL);
    usb->start();
    while (1)
    {
        uint32_t t = event_group->waitEvents(0xffffffff, 1000);
        if (!t)
        {
            lnDigitalToggle(LED);
            lnDigitalToggle(LED2);
            lnDigitalToggle(LED3);
            continue;
        }
        if (t & (1 << lnUsbCDC::CDC_SESSION_START))
        {
            Logger("CDC SESSION START\n");
        }
        if (t & (1 << lnUsbCDC::CDC_SESSION_END))
        {
            Logger("CDC SESSION END\n");
        }
        if (t & (1 << lnUsbCDC::CDC_SET_SPEED))
        {
            Logger("CDC SET SPEED\n");
        }
        if (t & (1 << lnUsbCDC::CDC_DATA_AVAILABLE))
        {
            Logger("Dt\n");
            int n = cdc->read(buffer, 100);
            if (n)
            {
                cdc->write((uint8_t *)">", 1);
                cdc->write(buffer, n);
                cdc->flush();
            }
        }
    }
}
// EOF
