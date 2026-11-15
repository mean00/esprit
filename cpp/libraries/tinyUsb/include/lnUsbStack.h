

/*

*/
#pragma once
#include "class/cdc/cdc.h"
#include "tusb_types.h"
#include "usbd.h"

class lnUsbStack
{
  public:
    enum lnUsbStackEvents
    {
        USB_CONNECT = 0,
        USB_DISCONNECT = 1,
        USB_SUSPEND = 2,
        USB_RESUME = 3
    };

    typedef void lnUsbStackEventHandler(void *cookie, const lnUsbStackEvents event);

  public:
    lnUsbStack();
    virtual ~lnUsbStack();
    void init(int nbDescriptorLine, const char **deviceDescriptor);
    void setConfiguration(const uint8_t *hsConfiguration, const uint8_t *fsConfiguration,
                          const tusb_desc_device_t *desc, const tusb_desc_device_qualifier_t *qual)
    {
        _hs = hsConfiguration;
        _fs = fsConfiguration;
        _descDevice = desc;
        _qualifier = qual;
    }
    void setEventHandler(void *cookie, lnUsbStackEventHandler *ev)
    {
        _eventHandler = ev;
        _cookie = cookie;
    }
    const uint8_t *getHSConfiguration()
    {
        return _hs;
    }
    const uint8_t *getFSConfiguration()
    {
        return _fs;
    }
    const tusb_desc_device_t *getDeviceDescriptor()
    {
        return _descDevice;
    }
    const tusb_desc_device_qualifier_t *getQualifier()
    {
        return _qualifier;
    }
    void sendEvent(lnUsbStackEvents ev);
    void start();
    void stop();

  public:
    void task();

  protected:
    bool _running;
    const char **_deviceDescriptor;
    int _nbDeviceDescriptor;
    const tusb_desc_device_qualifier_t *_qualifier;
    const tusb_desc_device_t *_descDevice;
    const uint8_t *_hs, *_fs;
    lnUsbStackEventHandler *_eventHandler;
    void *_cookie;

  public: // dont use these
    uint16_t const *getDeviceDescriptor(int index);
};
