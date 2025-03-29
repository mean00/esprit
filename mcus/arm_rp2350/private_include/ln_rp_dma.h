/**
 * @file ln_rp_dma.h
 * @author mean00
 * @brief
 * @version 0.1
 * @date 2023-11-01
 *
 * @copyright Copyright (c) 2023
 *
 */
#pragma once
#include "lnArduino.h"
/**
 * @brief
 *
 */

struct LN_RP_DMA_channelx;
typedef volatile LN_RP_DMA_channelx LN_RP_DMA_channel;

class lnRpDMA
{
  public:
    typedef void(doneCallback)(void *cookie);

    //
    // Source of IRQ to DMA
    //

#define DREQ_SPLAT(x, count, start) LN_DMA_DREQ_##x##count##_TX = start, LN_DMA_DREQ_##x##count##_RX

    enum LN_RP_DMA_DREQ // 12.6.4.1
    {
        DREQ_SPLAT(SPI, 0, 24),  // LN_DMA_DREQ_SPI0_TX=24
        DREQ_SPLAT(SPI, 1, 26),  // LN_DMA_DREQ_SPI1_TX=26
        DREQ_SPLAT(UART, 0, 28), // 28 & 29
        DREQ_SPLAT(UART, 1, 30), // 30 & 31
        DREQ_SPLAT(I2C, 0, 44),
        DREQ_SPLAT(I2C, 1, 46),
        LN_DMA_DREQ_ADC = 48,
    };
    enum DmaTransferType
    {
        DMA_MEMORY_TO_PERIPH,
        DMA_PERIPH_TO_MEMORY
    };

    enum DmaPriority
    {
        DMA_PRIORITY_LOW = 0,
        DMA_PRIORITY_HIGH = 2,
    };

    lnRpDMA(DmaTransferType type, LN_RP_DMA_DREQ treq, int transferWidth, DmaPriority prio = DMA_PRIORITY_LOW);
    ~lnRpDMA();

    void attachCallback(doneCallback *cb, void *cookie);
    void detachCallback();

    bool doMemoryToPeripheralTransferNoLock(int count, const uint32_t *source, const uint32_t *target,
                                            bool repeat = false);
    bool doPeripheralToMemoryTransferNoLock(int count, const uint32_t *source, const uint32_t *target,
                                            bool wrap = false);

    bool continuePeripheralToMemoryTransferNoLock(int count, const uint32_t *target);
    bool continueMemoryToPeripheralTransferNoLock(int count, const uint32_t *target);

    void setTransferSize(int size);

    void beginTransfer();
    void endTransfer();
    void cancelTransfer();
    void disableInterrupt();
    void enableInterrupt();

    uint32_t getCurrentCount();
    void invokeCallback()
    {
        xAssert(_cb);
        _cb(_cookie);
    }

  protected:
    int _channel;
    DmaTransferType _type;
    doneCallback *_cb;
    void *_cookie;
    DmaPriority _priority;
    int _transferWidth;
    LN_RP_DMA_DREQ _req;

    uint32_t _control; // initial value of the control register
    LN_RP_DMA_channel *_dma;
};
