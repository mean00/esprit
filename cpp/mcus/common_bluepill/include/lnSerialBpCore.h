#pragma once

#include "esprit.h"
#include "lnDma.h"
#include "lnPeripheral_priv.h"
#include "lnSerial.h"
#include "lnSerial_priv.h"

/**
 * @brief
 *
 */
class lnSerialBpCore : public lnSerialCore
{
  public:
    enum txState
    {
        txTransmittingIdle,
        txTransmittingInterrupt,
        txTransmittingDMA,
        txTransmittingLast
    };

    // public API
    lnSerialBpCore(uint32_t instance);
    bool init();
    bool setSpeed(uint32_t speed);

    // implementation
    virtual void _interrupt(void) = 0;

    static void interrupts(int instance);

  protected:
    virtual bool _programTx() = 0;
    void disableInterrupt();
    void enableInterrupt(bool tx, bool rx);
    bool _enableTx(txState mode);

    //--
    void _setCallback(lnSerialCallback *cb, void *cookie)
    {
        _cb = cb;
        _cbCookie = cookie;
    }

    lnSerialCallback *_cb;
    void *_cbCookie;

    LnIRQ _irq;
    uint32_t _adr;
    // tx
    volatile const uint8_t *_cur, *_tail;
    txState _txState;
    lnBinarySemaphore _txDone;
    lnMutex _txMutex;
};

/**
 * @brief
 *
 * @param d
 * @param count
 * @param c
 * @return true
 * @return false
 */
bool ln_serial_rawWrite(LN_USART_Registers *d, uint32_t count, unsigned char const *c);

/**
 * @brief
 *
 */
struct UsartMapping
{
    uint32_t usartEngine;
    const uint8_t dmaEngine;
    const uint8_t dmaTxChannel;
    const uint8_t dmaRxChannel;
    const uint8_t filler;
    const LnIRQ irq;
    const lnPin tx;
    const lnPin rx;
    const Peripherals periph;
};
extern const UsartMapping usartMapping[];

// EOF
