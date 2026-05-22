/*
 *  (C) 2021/2022 MEAN00 fixounet@free.fr
 *  See license file
 *
 * Normal usage :
 * --------------
 *    setup()
 *    begin()
 *    ...
 *    ...
 *    beginTransaction() <= Lock on a specific device till endTransaction
 *    beginSession(8)  <= while in beginSession the SPI stays enabled and the CS stays on (if defined),
 *                             also the bus width stays the same (8/16 bits)
 *    write8
 *    write8
 *    read8
 *    endSession()
 *    beginSession(16)
 *    write16
 *    write16
 *    read16
 *    endSession()
 * endTransaction() <= Other devices can be used
 *
 *
 *
 */

#pragma once
#include "esprit.h"
#include "lnDma.h"
#include "lnSPI.h"
#include "lnSPI_priv.h"

/**
 *
 * @param instance
 * @param pinCs
 */
class lnSPI_bp : public lnSPI
{
  public:
    lnSPI_bp(int instance, int pinCs = -1);
    virtual ~lnSPI_bp();

    virtual void begin(uint32_t dataSize);
    virtual void end();
    virtual void setBitOrder(spiBitOrder order);
    virtual void setDataMode(spiDataMode mode);
    virtual void setSpeed(uint32_t speed); // speed in b/s
    void setSSEL(int ssel)
    {
        _internalSettings.pinCS = (ssel);
    };
    // async API
    virtual bool asyncWrite8(uint32_t nbBytes, const uint8_t *data, lnSpiCallback *cb, void *cookie, bool repeat = false)
    {
        return asyncWrite(8, nbBytes, data, cb, cookie, repeat);
    }
    virtual bool nextWrite8(uint32_t nbTransfer, const uint8_t *data, lnSpiCallback *cb, void *cookie, bool repeat = false)
    {
        return nextWrite(nbTransfer, data, cb, cookie, repeat);
    }
    virtual bool asyncWrite16(uint32_t nbWords, const uint16_t *data, lnSpiCallback *cb, void *cookie, bool repeat = false)
    {
        return asyncWrite(16, nbWords, (const uint8_t *)data, cb, cookie, repeat);
    }
    virtual bool nextWrite16(uint32_t nbWords, const uint16_t *data, lnSpiCallback *cb, void *cookie, bool repeat = false)
    {
        return nextWrite(nbWords, (const uint8_t *)data, cb, cookie, repeat);
    }
    virtual bool finishAsyncDma();
    virtual bool waitForAsync();

    // Helper Write functions, make sure you are under a session
    virtual bool write8(const uint8_t z);
    virtual bool write16(const uint16_t z);

    virtual bool write8(uint32_t nbBytes, const uint8_t *data);
    virtual bool write16(uint32_t nbBytes, const uint16_t *data);

    // block write
    // just block write, no need for begin() end()
    virtual bool blockWrite16(uint32_t nbWord, const uint16_t *data)
    {
        return dmaWriteInternal(16, nbWord, (uint8_t *)data, false);
    }
    virtual bool blockWrite16Repeat(uint32_t nbWord, const uint16_t data)
    {
        return dmaWriteInternal(16, nbWord, (uint8_t *)&data, true);
    }
    virtual bool blockWrite8(uint32_t nbBytes, const uint8_t *data)
    {
        return dmaWriteInternal(8, nbBytes, data, false);
    }
    virtual bool blockWrite8Repeat(uint32_t nbBytes, const uint8_t data)
    {
        return dmaWriteInternal(8, nbBytes, (uint8_t *)&data, true);
    }
    virtual void waitForCompletion() const;
    // slow read/write
    virtual bool transfer(uint32_t nbBytes, uint8_t *dataOut, uint8_t *dataIn);

    // wait for everything to be COMPLETELY done
    // virtual void waitForCompletion()=0;
    // This reads over the MOSI pin, i.e. only when only 2 wires are used MOSI + CLK, no MISO
    virtual bool read1wire(uint32_t nbRead, uint8_t *rd); // read, reuse MOSI
                                                     //

  protected:
    bool asyncWrite(int wordSize, uint32_t nbWord, const uint8_t *data, lnSpiCallback *cb, void *cookie, bool repeat);
    bool nextWrite(uint32_t nbBytes, const uint8_t *data, lnSpiCallback *cb, void *cookie, bool repeat = false);
    void setup();
    void csOn();
    void csOff();

  protected:
    lnBinarySemaphore _done;
    LN_SPI_Registers *_regs;
    LnIRQ _irq;

    lnDMA txDma;
    // callbacks
    static void exTxDone(void *c, lnDMA::DmaInterruptType it);
    bool writeInternal(int sz, int data);
    bool writesInternal(int sz, uint32_t nbBytes, const uint8_t *data, bool repeat = false);
    bool dmaWriteInternal(int wordSize, uint32_t nbBytes, const uint8_t *data, bool repeat);

  public:
    void txDone();
    void invokeCallback();
};
// EOF
