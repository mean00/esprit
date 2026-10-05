/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */
#pragma once
// #include "esprit.h"
#include "stdint.h"
enum lnPin : int;
enum LnIRQ : int;
/**
 *
 * \brief If you do a dma transfer

 * call beginTransfer() from the task, that will lock the DMA for you
 * call doMemoryToPeripheralTransferFromInterrupt() from the task or the interrupt
 * call endTransfer() when you will not use the dma any longer until the next begin()
 * If you are starting a DMA from interrupt, you should do the begin() out of the interrupt and the end when that
 transfer is 100% done
 * If you are starting a DMA from a task you can do begin()/ doMemoryToPeripheral/end() as many times as you want, that
 will leave
 * the dma channel usable by another peripheral in between at the expense of more overhead

 *
 */
class DMA_structx;
typedef volatile DMA_structx DMA_struct;

class lnDMA
{
  public:
    enum DmaInterruptType
    {
        DMA_INTERRUPT_HALF,
        DMA_INTERRUPT_FULL
    };
    typedef void(doneCallback)(void *, DmaInterruptType);

    enum DmaTransferType
    {
        DMA_MEMORY_TO_PERIPH,
        DMA_PERIPH_TO_MEMORY
    };

    enum DmaPriority
    {
        DMA_PRIORITY_LOW = 0,
        DMA_PRIORITY_NORMAL = 1,
        DMA_PRIORITY_HIGH = 2,
        DMA_PRIORITY_ULTRA_HIGH = 3,
    };

    lnDMA(DmaTransferType type, int dmaEngine, int dmaChannel, int sourceWith, int targetWidth);
    ~lnDMA();
    void attachCallback(doneCallback *cb, void *cookie);
    void detachCallback();

    bool doMemoryToPeripheralTransferNoLock(int count, const uint16_t *source, const uint16_t *target, bool repeat,
                                            bool circularMode = false, bool bothInterrutps = false);
    bool doPeripheralToMemoryTransferNoLock(int count, const uint16_t *target, const uint16_t *source,
                                            bool circularMode = false, bool bothInterrutps = false);
    bool setInterruptMask(bool full, bool half);
    void setPriority(DmaPriority prio); // prio between 0 (low) to 3 (ultra high)
    void beginTransfer();
    uint32_t getCurrentCount();
    void endTransfer();
    void cancelTransfer();
    void setWordSize(int sourceWordSize, int targetWordSize);
    void invokeCallback(uint32_t pending);
    void pause();
    void resume();
    void enableInterrupt();
    void disableInterrupt();

  protected:
    DMA_struct *_dma;
    int _channelInt;
    int _dmaInt;
    DmaTransferType _type;
    LnIRQ _irq;

    doneCallback *_cb;
    void *_cookie;
    uint32_t _control;
    uint32_t _priority;
    uint32_t _interruptMask;

    int _sourceWidth, _targetWidth;
};

extern "C" {
void lnDmaAttachRawCallback(int dma, int channel, void (*cb)(bool half, void *cookie), void *cookie);
void lnDmaDetachRawCallback(int dma, int channel);
}
