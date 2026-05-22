/*
 *  (C) 2021/2024 MEAN00 fixounet@free.fr
 *  See license file
 */
#include "lnDma.h"
#include "esprit.h"
#include "lnDma_priv.h"
#include "lnPeripheral_priv.h"

#define DMA_HALF_INTERRUPT 2
#define DMA_FULL_INTERRUPT 1

#define CLEAR_ALL_DMA_INTERRUPT()                                                                                      \
    {                                                                                                                  \
        _dma->INTC = 0xF << (4 * _channelInt);                                                                         \
    }

//-----------------------
#define z0(x) LN_IRQ_DMA0_Channel##x
#define z1(x) LN_IRQ_DMA1_Channel##x

static lnDMA *_lnDmas[2][7] = {{NULL, NULL, NULL, NULL, NULL, NULL, NULL}, {NULL, NULL, NULL, NULL, NULL, NULL, NULL}};
static const LnIRQ _dmaIrqs[2][7] = {
    {z0(0), z0(1), z0(2), z0(3), z0(4), z0(5), z0(6)},
    {z1(0), z1(1), z1(2), z1(3), z1(4), LN_IRQ_NONE, LN_IRQ_NONE}}; // Warning DMA CHANNEL5/6 is not available

static DMA_struct *_dmas[2] = {(DMA_struct *)LN_DMA0_ADR, (DMA_struct *)LN_DMA1_ADR};
static lnMutex *dmaMutex[2][7];

struct lnDmaStats
{
    uint16_t error, spurious, missed, total, half, full;
};

static lnDmaStats dmaStats[2][7];

/**
 */
extern const DMA_struct *adma0, *adma1, *dummy_dma;
const DMA_struct *adma0 = (DMA_struct *)LN_DMA0_ADR;
const DMA_struct *adma1 = (DMA_struct *)LN_DMA1_ADR;
const DMA_struct *dummy_dma;

/**
 * @brief
 *
 */
void lnDmaSysInit()
{
    memset(&dmaStats, 0, sizeof(dmaStats));
    for (int i = 0; i < 7; i++)
    {
        dmaMutex[0][i] = new lnMutex;
        dmaMutex[1][i] = new lnMutex;
    }
    dummy_dma = adma0;
}

/**
 *
 * @param v
 * @return
 */
static uint32_t memoryWidth(int v)
{
    switch (v)
    {
    case 8:
        return LN_DMA_CHAN_MWIDTH_8;
        break;
    case 16:
        return LN_DMA_CHAN_MWIDTH_16;
        break;
    case 32:
        return LN_DMA_CHAN_MWIDTH_32;
        break;
    default:
        xAssert(0);
        return 0;
    }
}
/**
 *
 * @param v
 * @return
 */
static uint32_t peripheralWidth(int v)
{
    switch (v)
    {
    case 8:
        return LN_DMA_CHAN_PWIDTH_8;
        break;
    case 16:
        return LN_DMA_CHAN_PWIDTH_16;
        break;
    case 32:
        return LN_DMA_CHAN_PWIDTH_32;
        break;
    default:
        xAssert(0);
        return 0;
    }
}

/**
 *
 * @param type
 * @param dmaEngine
 * @param dmaChannel
 * @param sourceWith
 * @param targetWidth
 */
lnDMA::lnDMA(DmaTransferType type, int dmaEngine, int dmaChannel, int sourceWidth, int targetWidth)
{
    _control = 0;
    _channelInt = dmaChannel;
    _dmaInt = dmaEngine;

    _dma = _dmas[dmaEngine];
    _type = type;
    _irq = _dmaIrqs[dmaEngine][dmaChannel];

    _sourceWidth = sourceWidth;
    _targetWidth = targetWidth;

    _priority = LN_DMA_CHAN_PRIO_HIGH;
}
/**
 *
 * @param prio
 */
void lnDMA::setPriority(DmaPriority prio)
{
    _priority = (int)prio << 12;
}

/**
 *
 * @param sourceWordSize
 * @param targetWordSize
 */
void lnDMA::setWordSize(int sourceWordSize, int targetWordSize)
{
    uint32_t source = 0;
    uint32_t target = 0;
    switch (_type)
    {
    case DMA_MEMORY_TO_PERIPH: {
        source = memoryWidth(sourceWordSize);
        target = peripheralWidth(targetWordSize);
        _control &= LN_DMA_CHAN_WIDTH_MASK;
        _control |= source;
        _control |= target;
    }
    break;
    case DMA_PERIPH_TO_MEMORY: {
        source = peripheralWidth(sourceWordSize);
        target = memoryWidth(targetWordSize);
        _control &= LN_DMA_CHAN_WIDTH_MASK;
        _control |= source;
        _control |= target;
    }
    break;
    default:
        xAssert(0);
        break;
    }
}

/**
 *
 */
lnDMA::~lnDMA()
{

    DMA_channels *channel = _dma->channels + _channelInt;
    channel->CTL = 0;
}

/**
 *
 * @param cb
 * @param cookie
 */
void lnDMA::attachCallback(doneCallback *cb, void *cookie)
{
    _cb = cb;
    _cookie = cookie;
}
/**
 *
 */
void lnDMA::detachCallback()
{
    _cb = NULL;
    _cookie = NULL;
}

/**
 *
 * @param count
 * @param source
 * @param repeat
 * @return
 */

/**
 *
 */
void lnDMA::beginTransfer()
{
    dmaMutex[_dmaInt][_channelInt]->lock();
    // Ok now we can configure the dma
    DMA_channels *channel = _dma->channels + _channelInt;

    channel->CNT &= LN_DMA_CHAN_KEEP_MASK; // disable

    // Clear interrupts & flags
    CLEAR_ALL_DMA_INTERRUPT();

    uint32_t sw;
    uint32_t source;
    uint32_t target;
    switch (_type)
    {
    case DMA_MEMORY_TO_PERIPH: {
        source = memoryWidth(_sourceWidth);
        target = peripheralWidth(_targetWidth);
        _control &= LN_DMA_CHAN_WIDTH_MASK;
        _control |= source;
        _control |= target;
        _control |= LN_DMA_CHAN_DIR_M2P;
        _control |= _priority; // LN_DMA_CHAN_PRIO_HIGH;
        break;
    }
    case DMA_PERIPH_TO_MEMORY: {
        source = peripheralWidth(_sourceWidth);
        target = memoryWidth(_targetWidth);
        _control &= LN_DMA_CHAN_WIDTH_MASK;
        _control |= source;
        _control |= target;
        _control |= LN_DMA_CHAN_DIR_P2M;
        _control |= _priority; // LN_DMA_CHAN_PRIO_HIGH;
        break;
    }
    default:
        xAssert(0);
        break;
    }

    if (_lnDmas[_dmaInt][_channelInt] != NULL)
        xAssert(0);
    _lnDmas[_dmaInt][_channelInt] = this;
}

/**
 * @brief
 *
 */
void lnDMA::cancelTransfer()
{
    DMA_channels *channel = _dma->channels + _channelInt;
    lnNoInterrupt();
    uint32_t control = channel->CTL;
    control &= ~LN_DMA_CHAN_ENABLE;
    control &= ~(LN_DMA_CHAN_ERRIE + LN_DMA_CHAN_TFTFIE + LN_DMA_CHAN_HTFIE);
    channel->CTL = control;

    _cb = NULL;
    _cookie = NULL;
    CLEAR_ALL_DMA_INTERRUPT();
    _lnDmas[_dmaInt][_channelInt] = NULL;
    lnInterrupts();
}
/**
 * @brief
 *
 */
void lnDMA::endTransfer()
{
    cancelTransfer();
    dmaMutex[_dmaInt][_channelInt]->unlock();
}

/**
 *
 * @param count
 * @param source
 * @param target
 * @param repeat
 * @return
 */
bool lnDMA::doMemoryToPeripheralTransferNoLock(int count, const uint16_t *source, const uint16_t *target, bool repeat,
                                               bool circular, bool bothInterrupts)
{
    // clear pending bits
    CLEAR_ALL_DMA_INTERRUPT();

    DMA_channels *channel = _dma->channels + _channelInt;
    uint32_t control = channel->CTL;
    control &= LN_DMA_CHAN_KEEP_MASK;

    channel->CTL = control; // also disable

    control = _control; // base configuration

    channel->PADDR = (uint32_t)target;
    channel->MADDR = (uint32_t)source;
    channel->CNT = (uint32_t)count;
    if (!repeat)
        control |= LN_DMA_CHAN_MINCREASE; // increase address

    if (circular)
    {
        control |= LN_DMA_CHAN_CMEN; // replay the dma in a loop
    }
    else
    {
        control &= ~LN_DMA_CHAN_CMEN;
    }
    control |= LN_DMA_CHAN_ERRIE + LN_DMA_CHAN_TFTFIE; // error  interrupt + transfer complete
    if (bothInterrupts)
    {
        control |= LN_DMA_CHAN_HTFIE; // half transfer
    }
    _interruptMask = control & LN_DMA_CHAN_ALL_INTERRUPT_MASK;
    channel->CTL = control | LN_DMA_CHAN_ENABLE; // GO!
    lnEnableInterrupt(_irq);
    return true;
}
/**
 * \fn getCurrentCount
 * \brief return the current DMA address
 * @return
 */
uint32_t lnDMA::getCurrentCount()
{
    // clear pending bits
    DMA_channels *channel = _dma->channels + _channelInt;
    uint32_t cnt = channel->CNT;
    return cnt;
}
/**
 *
 */
void lnDMA::pause()
{

    DMA_channels *channel = _dma->channels + _channelInt;
    uint32_t control = channel->CTL;
    CLEAR_ALL_DMA_INTERRUPT(); // clear pending interrupt
    control &= ~LN_DMA_CHAN_ENABLE;
    channel->CTL = control;
}
/**
 *
 */
void lnDMA::resume()
{
    DMA_channels *channel = _dma->channels + _channelInt;
    uint32_t control = channel->CTL;
    control |= LN_DMA_CHAN_ENABLE;
    channel->CTL = control;
}
/**
 * @brief
 *
 */
void lnDMA::enableInterrupt()
{
    DMA_channels *channel = _dma->channels + _channelInt;
    uint32_t nw = channel->CTL;
    nw &= ~LN_DMA_CHAN_ALL_INTERRUPT_MASK;
    nw |= _interruptMask;
    channel->CTL = nw;
}
/**
 * @brief
 *
 */
void lnDMA::disableInterrupt()
{

    DMA_channels *channel = _dma->channels + _channelInt;
    channel->CTL &= ~LN_DMA_CHAN_ALL_INTERRUPT_MASK;
}
/**
 *
 * @param count
 * @param target
 * @param source
 * @param repeat
 * @param circularMode
 * @return
 */
bool lnDMA::doPeripheralToMemoryTransferNoLock(int count, const uint16_t *target, const uint16_t *source, bool circular,
                                               bool bothInterrupts)
{
    // clear pending bits

    CLEAR_ALL_DMA_INTERRUPT();

    DMA_channels *channel = _dma->channels + _channelInt;
    uint32_t control = channel->CTL;
    control &= LN_DMA_CHAN_KEEP_MASK;

    channel->CTL = control; // also disable

    control = _control; // base configuration

    channel->PADDR = (uint32_t)source;
    channel->MADDR = (uint32_t)target;
    channel->CNT = (uint32_t)count;

    control |= LN_DMA_CHAN_MINCREASE; // increase address

    if (circular)
    {
        control |= LN_DMA_CHAN_CMEN; // replay the dma in a loop
    }
    else
    {
        control &= ~LN_DMA_CHAN_CMEN;
    }
    control |= LN_DMA_CHAN_ERRIE + LN_DMA_CHAN_TFTFIE; // error and transmit complete interrupt
    if (bothInterrupts)
        control |= LN_DMA_CHAN_HTFIE;
    _interruptMask = control & LN_DMA_CHAN_ALL_INTERRUPT_MASK;
    channel->CTL = control | LN_DMA_CHAN_ENABLE; // GO!
    lnEnableInterrupt(_irq);
    return true;
}
/**
 *
 * @param full
 * @param half
 * @return
 */
bool lnDMA::setInterruptMask(bool full, bool half)
{
    // clear pending bits
    DMA_channels *channel = _dma->channels + _channelInt;

    uint32_t control = channel->CTL;
    control &= ~LN_DMA_CHAN_ALL_INTERRUPT_MASK; // reset
    control |= LN_DMA_CHAN_ERRIE;
    if (full)
        control |= LN_DMA_CHAN_TFTFIE;
    if (half)
        control |= LN_DMA_CHAN_HTFIE;

    // clear interrupts if any
    CLEAR_ALL_DMA_INTERRUPT();
    _interruptMask = control & LN_DMA_CHAN_ALL_INTERRUPT_MASK;
    channel->CTL = control;
    return true;
}
/**
 * @brief
 *
 */
void lnDMA::invokeCallback(uint32_t pending)
{
    DmaInterruptType typ;

    lnDmaStats *stats = &(dmaStats[_dmaInt][_channelInt]);
    stats->total++;
    switch (pending)
    {
    case 0: // no interrupt pending
        stats->spurious++;
        return;
        break;
    case DMA_FULL_INTERRUPT:                      // complete
    case DMA_FULL_INTERRUPT + DMA_HALF_INTERRUPT: // complete + half
        typ = DMA_INTERRUPT_FULL;
        stats->full++;
        break;
    case DMA_HALF_INTERRUPT: // half
        typ = DMA_INTERRUPT_HALF;
        stats->half++;
        break;
    default:
        xAssert(0);
        break;
    }
    xAssert(_cb);
    _cb(_cookie, typ);
}
/**
 *
 * @param dma
 * @param channel
 */
void dmaIrqHandler(int dma, int channel)
{
    // First clear the interrupt
    DMA_struct *dm = _dmas[dma];
    DMA_channels *chan = dm->channels + channel;

    uint32_t status = dm->INTF; // pending...
    status >>= (4 * channel);   // ERR DONE HALFDONE GLOBAL
    if ((status & 8) != 0)      // error bit
    {
        xAssert(0);
    }

    // Are we in circular mode ?
    if ((chan->CTL & LN_DMA_CHAN_CMEN) != 0)
    {
        // circular, nothing special to do
    }
    else
    { // since it is one shot, disable DMA channel
        chan->CTL &= ~LN_DMA_CHAN_ENABLE;
        // disable interrupt
        lnDisableInterrupt(_dmaIrqs[dma][channel]);
    }
    // Ack the DMA interrupt
    status &= chan->CTL;                                 // only check enabled interrupt
    status >>= 1;                                        // remove the GIF, we dont care
    status &= (DMA_HALF_INTERRUPT + DMA_FULL_INTERRUPT); // Only keep FT and HT

    dm->INTC = (status << 1) << (4 * channel); // clear pending only

    // the interrupt itself will be acked in invokeCallback
    lnDMA *la = _lnDmas[dma][channel];
    xAssert(la);
    la->invokeCallback(status);
}

// EOF
