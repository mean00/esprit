#pragma once
/**
 * @brief
 *
 */
/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */

#include "esprit.h"
#include "lnDma.h"
#include "lnPeripheral_priv.h"
#include "lnSerial.h"
#include "lnSerialBpCore.h"
#include "lnSerial_priv.h"

class lnSerialBpTxOnlyInterrupt : public lnSerialBpCore, public lnSerialTxOnly
{
  public:
    lnSerialBpTxOnlyInterrupt(int instance);
    virtual bool _programTx();
    bool init()
    {
        return lnSerialBpCore::init();
    }
    bool setSpeed(int speed)
    {
        return lnSerialBpCore::setSpeed(speed);
    }
    virtual bool transmit(int size, const uint8_t *buffer);
    virtual int transmitNoBlock(int size, const uint8_t *buffer);
    void txInterruptHandler(void);
    virtual void _interrupt(void);
    virtual bool rawWrite(int count, const uint8_t *buffer)
    {
        LN_USART_Registers *d = (LN_USART_Registers *)_adr;
        return ln_serial_rawWrite(d, count, buffer);
    }
};