
/*

 */
#include "esprit.h"
#include "lnPinMapping.h"
#include "lnUSBD.h"
#include "lnUSBD_priv.h"

lnUsbDevice *_usbInstance = NULL;

/**
 */
lnUsbDevice::lnUsbDevice(int port)
{
    xAssert(port == 0);
    _handler = NULL;
    _usbInstance = this;
}
/**
 */
lnUsbDevice::~lnUsbDevice()
{
    _usbInstance = NULL;
}
/**
 */
bool lnUsbDevice::init()
{

    return true;
}
/**
 */
bool lnUsbDevice::power(bool onoff)
{

    return true;
}
/**
 */
bool lnUsbDevice::irqEnabled(bool onoff)
{

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
}
// EOF
