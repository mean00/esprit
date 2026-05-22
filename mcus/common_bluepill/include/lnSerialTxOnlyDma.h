#pragma once
#include "lnDma.h"
#include "lnRingBuffer.h"

/**
 * @brief
 *
 */
class lnSerialBpTxOnlyDma : public lnSerialBpCore, public lnSerialTxOnly
{
  public:
    lnSerialBpTxOnlyDma(uint32_t instance);

    bool init()
    {
        return lnSerialBpCore::init();
    }
    bool setSpeed(uint32_t speed)
    {
        return lnSerialBpCore::setSpeed(speed);
    }
    virtual bool rawWrite(uint32_t count, const uint8_t *buffer)
    {
        LN_USART_Registers *d = (LN_USART_Registers *)_adr;
        return ln_serial_rawWrite(d, count, buffer);
    }
    virtual bool transmit(uint32_t size, const uint8_t *buffer);
    virtual void _interrupt(void)
    {
        xAssert(0);
    }
    virtual bool _programTx();
    static void _dmaCallback(void *c, lnDMA::DmaInterruptType it);
    void txDmaCb(lnDMA::DmaInterruptType it);
    lnDMA _txDma;
};
/**
 * @brief
 *
 */
class lnSerialBpTxOnlyBufferedDma : public lnSerialBpCore, public lnSerialTxOnly
{
  public:
    lnSerialBpTxOnlyBufferedDma(uint32_t instance, uint32_t txBufferSize);
    virtual ~lnSerialBpTxOnlyBufferedDma();
    bool init()
    {
        return lnSerialBpCore::init();
    }
    bool setSpeed(uint32_t speed)
    {
        return lnSerialBpCore::setSpeed(speed);
    }
    virtual bool rawWrite(uint32_t count, const uint8_t *buffer)
    {
        LN_USART_Registers *d = (LN_USART_Registers *)_adr;
        return ln_serial_rawWrite(d, count, buffer);
    }
    virtual bool transmit(uint32_t size, const uint8_t *buffer);
    virtual int transmitNoBlock(uint32_t size, const uint8_t *buffer);
    virtual void _interrupt(void)
    {
        xAssert(0);
    }
    virtual bool _programTx();
    static void _dmaCallback2(void *c, lnDMA::DmaInterruptType it);
    void txDmaCb2(lnDMA::DmaInterruptType it);
    void igniteTx();
    lnDMA _txDma;

    lnRingBuffer _txRingBuffer;
    lnBinarySemaphore _txRingSem;
    bool _txing;
    int _inFlight;
};

// EOF