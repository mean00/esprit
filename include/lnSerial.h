/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */

#pragma once
#include "esprit.h"
#include <type_traits>

/**
 * @brief
 *
 */

class lnSerialCore
{
  public:
    enum Event
    {
        dataAvailable,
        txDone,
    };

    lnSerialCore(uint32_t instance)
    {
        _instance = instance;
    }
    virtual ~lnSerialCore()
    {
    }
    virtual bool init() = 0;
    virtual bool setSpeed(uint32_t speed) = 0;

  protected:
    int _instance;
};

typedef void lnSerialCallback(void *cookie, lnSerialCore::Event event);

/**
 * @brief
 *
 */
class lnSerialTxOnly : public lnSerialCore
{
  public:
    lnSerialTxOnly(uint32_t instance) : lnSerialCore(instance)
    {
    }
    virtual ~lnSerialTxOnly()
    {
    }
    virtual bool transmit(uint32_t size, const uint8_t *buffer) = 0;
    virtual bool rawWrite(uint32_t size, const uint8_t *buffer) = 0;
};
/**
 * @brief
 *
 */
class lnSerialRxTx : public lnSerialCore
{
  public:
    lnSerialRxTx(uint32_t instance) : lnSerialCore(instance)
    {
    }
    virtual ~lnSerialRxTx()
    {
    }
    virtual bool transmit(uint32_t size, const uint8_t *buffer) = 0;
    virtual int transmitNoBlock(uint32_t size, const uint8_t *buffer) = 0;
    virtual bool enableRx(bool enabled) = 0;
    virtual void purgeRx() = 0;
    virtual int read(uint32_t max, uint8_t *to) = 0;
    virtual void setCallback(lnSerialCallback *cb, void *cookie) = 0;
    // no copy interface
    virtual int getReadPointer(uint8_t **to) = 0;
    virtual void consume(uint32_t n) = 0;
};

lnSerialTxOnly *createLnSerialTxOnly(uint32_t instance, bool dma, bool buffered);
lnSerialRxTx *createLnSerialRxTx(uint32_t instance, uint32_t rxBufferSize = 128, bool dma = true);

// EOF
