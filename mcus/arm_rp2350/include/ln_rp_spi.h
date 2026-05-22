/**
 * @file ln_rp_spi_priv.h
 * @author mean00
 * @brief
 * @version 0.1
 * @date 2023-12-26
 *
 * @copyright Copyright (c) 2023
 *
 */
#pragma once
#include "esprit.h"
#include "lnSPI.h"
class lnRpDMA;
class LN_RP_SPIx;
typedef volatile LN_RP_SPIx LN_RP_SPI;
/**
 *
 * @param instance
 * @param pinCs
 */
class rpSPI : public lnSPI
{
  public:
    rpSPI(uint32_t instance, int pinCs = -1);
    virtual ~rpSPI();

    virtual void setBitOrder(spiBitOrder order);
    virtual void setDataMode(spiDataMode mode);
    void setSpeed(uint32_t speed);       // speed in b/s
    void setDataSize(uint32_t dataSize); // 8 or 16

    void begin(uint32_t wordsize = 8);
    void end(void);
    void set(lnSPISettings &settings);

    void setSSEL(int ssel)
    {
        _internalSettings.pinCS = (ssel);
    };
    // async API
    // begin()
    // async write
    // async next
    // finish
    // end()
    virtual bool asyncWrite8(uint32_t nbBytes, const uint8_t *data, lnSpiCallback *cb, void *cookie, bool repeat = false);
    virtual bool nextWrite8(uint32_t nbBytes, const uint8_t *data, lnSpiCallback *cb, void *cookie, bool repeat = false);
    virtual bool asyncWrite16(uint32_t nbWords, const uint16_t *data, lnSpiCallback *cb, void *cookie, bool repeat = false);
    virtual bool nextWrite16(uint32_t nbWords, const uint16_t *data, lnSpiCallback *cb, void *cookie, bool repeat = false);
    virtual bool finishAsyncDma();
    virtual bool waitForAsync();

    // simple writes, it is slow,   you need to do
    // begin
    // write
    // ....
    // end
    virtual bool write8(const uint8_t data);
    virtual bool write16(const uint16_t data);

    // block write
    // same thing
    // begin
    // blockWrite
    // ...
    // end
    virtual bool blockWrite16(uint32_t nbWord, const uint16_t *data);
    virtual bool blockWrite16Repeat(uint32_t nbWord, const uint16_t data);
    virtual bool blockWrite8(uint32_t nbBytes, const uint8_t *data);
    virtual bool blockWrite8Repeat(uint32_t nbBytes, const uint8_t data);
    virtual void waitForCompletion() const;

    // slow read/write
    virtual bool transfer(uint32_t nbBytes, uint8_t *dataOut, uint8_t *dataIn);
    virtual bool read1wire(uint32_t nbRead, uint8_t *rd);

    //-

  public:
    void irqHandler();
    void dmaHandler();

  protected:
    bool blockWrite_all(uint32_t wordSize, uint32_t nbExchange, const uint32_t *data, bool repeat);
    uint32_t _cr0, _cr1, _prescaler;
    uint32_t _instance;
    int _wordSize; // 8 or 16
    lnPin _cs;
    LN_RP_SPI *_spi;

    lnBinarySemaphore _txDone;
    lnRpDMA *_txDma;
};
