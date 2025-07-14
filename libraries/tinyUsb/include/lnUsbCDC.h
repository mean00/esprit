

#pragma once
#include "esprit.h"

class lnUsbCDC
{
  public:
    enum lnUsbCDCEvents
    {
        CDC_DATA_AVAILABLE,
        CDC_WRITE_AVAILABLE,
        CDC_SESSION_START,
        CDC_SESSION_END,
        CDC_SET_SPEED
    };
    typedef void lnUsbCDCEventsHandler(void *cookie, int interface, lnUsbCDCEvents event, uint32_t payload);

  public:
    lnUsbCDC(int instance);
    int read(uint8_t *buffer, int maxSize);
    int write(const uint8_t *buffer, int maxSize);
    int writeNoBlock(const uint8_t *buffer, int maxSize) const; // same as above but does not block
    void flush();
    void clear_input_buffers();
    int writeAvailable();
    void setEventHandler(lnUsbCDCEventsHandler *h, void *cookie)
    {
        _eventHandler = h;
        _eventCookie = cookie;
    }
    // internal API
    void incomingEvent(lnUsbCDCEvents ev, uint32_t payload = 0);
    void encodingChanged(const void *newEncoding); // this is a const  cdc_line_coding_t * in disguise

  protected:
    int _instance;
    lnUsbCDCEventsHandler *_eventHandler;
    void *_eventCookie;
};
