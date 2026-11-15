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

#include "lnSPI_defines.h"

/**
 *
 * @param instance
 * @param pinCs
 */
class lnSPI
{
  public:
    static lnSPI *create(uint32_t instance, int pinCs = -1);

    virtual ~lnSPI()
    {
    }

    virtual void begin(uint32_t dataSize = 8) = 0;
    virtual void end() = 0;
    void set(const lnSPISettings &st)
    {
        setSSEL(st.pinCS);
        setSpeed(st.speed);
        setBitOrder(st.bOrder);
        setDataMode(st.dMode);
    }
    virtual void setBitOrder(spiBitOrder order) = 0;
    virtual void setDataMode(spiDataMode mode) = 0;
    virtual void setSpeed(uint32_t speed) = 0; // speed in b/s
    void setSSEL(int ssel) // int for -1 sentinel
    {
        _internalSettings.pinCS = (ssel);
    };
    // async API
    // begin()
    // async write
    // async next
    // finish
    // end()
    virtual bool asyncWrite8(uint32_t nbBytes, const uint8_t *data, lnSpiCallback *cb, void *cookie,
                             bool repeat = false) = 0;
    virtual bool nextWrite8(uint32_t nbBytes, const uint8_t *data, lnSpiCallback *cb, void *cookie, bool repeat = false) = 0;
    virtual bool asyncWrite16(uint32_t nbWords, const uint16_t *data, lnSpiCallback *cb, void *cookie,
                              bool repeat = false) = 0;
    virtual bool nextWrite16(uint32_t nbWords, const uint16_t *data, lnSpiCallback *cb, void *cookie,
                             bool repeat = false) = 0;
    virtual bool finishAsyncDma() = 0;
    virtual bool waitForAsync() = 0;

    // simple writes, it is slow,   you need to do
    // begin
    // write
    // ....
    // end
    virtual bool write8(const uint8_t data) = 0;
    virtual bool write16(const uint16_t data) = 0;

    // block write
    // same thing
    // begin
    // blockWrite
    // ...
    // end
    virtual bool blockWrite16(uint32_t nbWord, const uint16_t *data) = 0;
    virtual bool blockWrite16Repeat(uint32_t nbWord, const uint16_t data) = 0;
    virtual bool blockWrite8(uint32_t nbBytes, const uint8_t *data) = 0;
    virtual bool blockWrite8Repeat(uint32_t nbBytes, const uint8_t data) = 0;
    virtual void waitForCompletion() const = 0;

    // slow read/write
    virtual bool transfer(uint32_t nbBytes, uint8_t *dataOut, uint8_t *dataIn) = 0;

    // wait for everything to be COMPLETELY done
    // virtual void waitForCompletion()=0;
    // This reads over the MOSI pin, i.e. only when only 2 wires are used MOSI + CLK, no MISO
    virtual bool read1wire(uint32_t nbRead, uint8_t *rd) = 0; // read, reuse MOSI
    //
    int getInstance()
    {
        return _instance;
    }

  protected:
    int _instance;
    lnSPISettings _internalSettings;
    lnSpiCallback *_callback;
    void *_callbackCookie;

  protected:
    lnSPI(uint32_t instance, int pinCs = -1)
        : _internalSettings(1000000, SPI_MSBFIRST, SPI_MODE0, -1), _callback(NULL), _callbackCookie(NULL)
    {
    }
};
// EOF
