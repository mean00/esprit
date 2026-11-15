
/*

 */
#include "esprit.h"
#include "lnPinMapping.h"
#include "lnUSBD.h"
#include "lnUSBD_priv.h"

lnUsbDevice *_usbInstance = NULL;

#define USB_DPLUS PB7
#define USB_DMINUS PB6

/**
 */
lnUsbDevice::lnUsbDevice(int port)
{
    xAssert(port == 0);
    _handler = NULL;
    _usbInstance = this;
    _remoteWakeCountdown = 4;
}
/**
 */
lnUsbDevice::~lnUsbDevice()
{
    power(false);
    _usbInstance = NULL;
}
/**
 */
bool lnUsbDevice::init()
{
    // Only do low level hw setup here
    // the follow up will be in hwReset
    // first disable interrupt
    irqEnabled(false);
    // 48 Mhz input to usb
    lnPeripherals::enableUsbHS48Mhz_ch32v3x();
    // Switch pins... D+ and D-
    lnPinMode(USB_DPLUS, lnINPUT_FLOATING);  // D+
    lnPinMode(USB_DMINUS, lnINPUT_FLOATING); // D-
    power(true);
    return true;
}
/**
 */
bool lnUsbDevice::power(bool onoff)
{
    if (onoff)
    {
        // lnPeripherals::enable(Peripherals::pUSBFS_OTG_CH32v3x);
        // lnPeripherals::reset(Peripherals::pUSBFS_OTG_CH32v3x);

        lnPeripherals::enable(Peripherals::pUSBHS_CH32v3x);
        lnPeripherals::reset(Peripherals::pUSBHS_CH32v3x);
    }
    else
    {
        // lnPeripherals::disable(Peripherals::pUSBFS_OTG_CH32v3x);
        lnPeripherals::disable(Peripherals::pUSBHS_CH32v3x);
    }
    return true;
}
/**
 */
bool lnUsbDevice::irqEnabled(bool onoff)
{

    if (onoff)
    {
        lnEnableInterrupt(LN_IRQ_USBHS);
        // lnEnableInterrupt(LN_IRQ_USB_LP_CAN_RX0);
        lnEnableInterrupt(LN_IRQ_USBHS_WKUP);
    }
    else
    {
        lnDisableInterrupt(LN_IRQ_USBHS);
        // lnDisableInterrupt(LN_IRQ_USB_LP_CAN_RX0);
        lnDisableInterrupt(LN_IRQ_USBHS_WKUP);
    }
    return true;
}
/**
 */
bool lnUsbDevice::registerEventHandler(const lnUsbEventHandler *h)
{
    _handler = h;
    return true;
}

/**

*/
void lnUsbDevice::hwReset()
{
#if 0 // According to GD doc...
    1. Clear the CLOSE bit in USBD_CTL register, then clear the SETRST bit.
    2. Clear USBD_INTF register to remove any spurious pending interrupt.
    3. Program USBD_BADDR register to set endpoint buffer base address.
    4. Set USBD_CTL register to enable interrupts.
    5. Wait for the reset interrupt (RSTIF).
    6. In the reset interrupt, initialize default control endpoint 0 to start enumeration process and
    program USBD_ADDR to set the device address to 0 and enable USB module function.
    7. Configure endpoint 0 and prepare to receive SETUP packet.
#endif

#if 0
    // Force host reset by asserting D+ low for 10 ms
    lnDigitalWrite(USB_DPLUS,0);
    lnPinMode(USB_DPLUS, lnOUTPUT); // D+
    lnDigitalWrite(USB_DPLUS,0);
    lnDelay(10);
    lnPinMode(USB_DPLUS, lnINPUT_FLOATING); // D+
#endif
}
// EOF
