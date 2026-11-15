#pragma once

#include "esprit.h"

class lnUsbEventHandler
{
  public:
    enum lnUsbdEvent
    {
        UsbReset = 0,
        UsbResume,
        UsbSuspend,
        UsbTransferRxCompleted,
        UsbTransferTxCompleted
    };

  public:
    virtual void event(const lnUsbdEvent ev, const uint32_t endPoint = 0) const = 0;
};

class lnUsbDevice
{
  public:
    lnUsbDevice(int port = 0);
    virtual ~lnUsbDevice();
    bool init();
    bool power(bool onoff);
    bool irqEnabled(bool onoff);
    bool registerEventHandler(const lnUsbEventHandler *h);
    bool setAddress(int address);
    bool wakeUpHost();
    // sram
    void copyFromSRAM(uint8_t *dest, int srcOffset, int bytes);
    void copyToSRAM(int destOffset, uint8_t *src, int bytes);
    // endpoints
    void setEpType(int ep, int type);
    void setEpAddress(int ep, int adr);
    void setEpKind(int ep, bool set);
    void setEpStatus(int ep, bool isTx, int status);
    void toggleDTG(int ep, bool isTx);
    void clearDTG(int ep, bool isTx);
    void clearTxRx(int ep, bool isTx);
    int getEpStatusReg(int ep);
    void resetEps();
    void setEpStatusReg(int ep, int value);
    void hwReset();

  public:
    void irq();

  protected:
    int _instance;
    int _remoteWakeCountdown;
    const lnUsbEventHandler *_handler;
};
