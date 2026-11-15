/**
 * @file ln_rp_dma.cpp
 * @author mean00
 * @brief
 * @version 0.1
 * @date 2023-11-01
 *
 * @copyright Copyright (c) 2023
 *
 */
// clang-format off
#include "ln_rp.h" 
#include "hardware/irq.h"
#include "hardware/regs/intctrl.h"
#include "ln_rp_dma.h"
#include "ln_rp_dma_priv.h"
// clang-format on

static lnRpDMA *dmaInstances[LN_RP_DMA_CHANNEL_COUNT] = {NULL, NULL, NULL, NULL, NULL, NULL,
                                                         NULL, NULL, NULL, NULL, NULL, NULL};

LN_RP_DMA *dmactrl = (LN_RP_DMA *)LN_RP_DMA_CONTROL;
/**
 * @brief
 *
 */
void dma_irq0_handler()
{
    uint32_t flags = dmactrl->INTS0;
    dmactrl->INTS0 = flags; // ack the DMA interrupt
    flags &= dmactrl->INTE0;
    uint32_t copy = flags;
    for (int i = 0; i < 12 && copy; i++)
    {
        int bit = 1 << i;
        if (copy & bit)
        {
            xAssert(dmaInstances[i]);
            dmaInstances[i]->invokeCallback();
        }
        copy &= ~bit;
    }
}
/**
 * @brief
 *
 * @return int
 */
static int lookupFreeChannel()
{
    for (int i = 0; i < LN_RP_DMA_CHANNEL_COUNT; i++)
        if (!dmaInstances[i])
            return i;
    xAssert(0);
    return 0;
}

/**
 * @brief
 *
 */
void lnRpDmaSysInit()
{
    LN_RP_DMA_channel *dma;
    irq_set_enabled(DMA_IRQ_0, false);
    for (int i = 0; i < LN_RP_DMA_CHANNEL_COUNT; i++)
    {
        dma = RP_DMA_CHANNEL(i);
        dma->DMA_CONTROL = 0; // dis
    }
    dmactrl->INTE0 = 0;
    dmactrl->INTE1 = 0; // disable all channel's interrupt
    irq_set_exclusive_handler(DMA_IRQ_0, dma_irq0_handler);
    irq_set_enabled(DMA_IRQ_0, true);
}
/**
 * @brief
 *
 * @param size
 */
void lnRpDMA::setTransferSize(int size)
{
    // now build up default control we'll re use later
    _transferWidth = size;
    _control &= ~LN_RP_DMA_CONTROL_SET_DATA_SIZE_MASK;
    switch (_transferWidth)
    {
    case 8:
        _control |= LN_RP_DMA_CONTROL_SET_DATA_SIZE_8;
        break;
    case 16:
        _control |= LN_RP_DMA_CONTROL_SET_DATA_SIZE_16;
        break;
    case 32:
        _control |= LN_RP_DMA_CONTROL_SET_DATA_SIZE_32;
        break;
    default:
        xAssert(0);
        break;
    }
}

/**
 * @brief Construct a new ln Rp D M A::ln Rp D M A object
 *
 * @param type
 * @param dmaChannel
 * @param sourceWith
 * @param targetWidth
 */
lnRpDMA::lnRpDMA(DmaTransferType type, LN_RP_DMA_DREQ req, int transferWidth, DmaPriority prio)
{
    _control = 0;
    _channel = lookupFreeChannel();
    _type = type;
    _priority = prio;
    _req = req;
    // now build up default control we'll re use later
    setTransferSize(transferWidth);
    switch (_type)
    {
    case DMA_MEMORY_TO_PERIPH:
        _control |= LN_RP_DMA_CONTROL_INCR_READ;
        break;
    case DMA_PERIPH_TO_MEMORY:
        _control |= LN_RP_DMA_CONTROL_INCR_WRITE;
        break;
    default:
        xAssert(0);
        break;
    }
    switch (_priority)
    {
    case DMA_PRIORITY_LOW:
        break;
    case DMA_PRIORITY_HIGH:
        _control |= LN_RP_DMA_CONTROL_HIGH_PRIO;
        break;
    default:
        xAssert(0);
        break;
    }
    _control |= LN_RP_DMA_CONTROL_TREQ(_req);
    _control |= LN_RP_DMA_CONTROL_CHAIN_TO(_channel); // chain to self
    //
    dmaInstances[_channel] = this;
    _dma = RP_DMA_CHANNEL(_channel);
    _dma->DMA_CONTROL = _control; // disable it while we are at it
}
/**
 * @brief Destroy the lnRpDMA object
 *
 */
lnRpDMA::~lnRpDMA()
{
    dmaInstances[_channel] = 0;
    _dma->DMA_CONTROL &= ~LN_RP_DMA_CONTROL_ENABLE;
}
/**
 * @brief
 *
 * @param cb
 * @param cookie
 */
void lnRpDMA::attachCallback(doneCallback *cb, void *cookie)
{
    _cb = cb;
    _cookie = cookie;
}
/**
 * @brief
 *
 */
void lnRpDMA::detachCallback()
{
    _cb = NULL;
    _cookie = NULL;
}
/**
 * @brief
 *
 * @param count
 * @param source
 * @param repeat
 * @return true
 * @return false
 */
bool lnRpDMA::doMemoryToPeripheralTransferNoLock(int count, const uint32_t *source, const uint32_t *target, bool repeat)
{
    xAssert(!(_dma->DMA_CONTROL & LN_RP_DMA_CONTROL_ENABLE));
    _dma->DMA_COUNT = count;
    _dma->DMA_READ = (uint32_t)source;
    _dma->DMA_WRITE = (uint32_t)target;
    uint32_t control = _control;
    control &= ~(LN_RP_DMA_CONTROL_INCR_READ + LN_RP_DMA_CONTROL_INCR_WRITE);
    if (!repeat)
    {
        control |= LN_RP_DMA_CONTROL_INCR_READ;
    }
    _dma->DMA_CONTROL = control;
    return true;
}
/**
 * @brief
 *
 * @param count
 * @param target
 * @return true
 * @return false
 */
bool lnRpDMA::continueMemoryToPeripheralTransferNoLock(int count, const uint32_t *source)
{
    uint32_t control = _control & ~LN_RP_DMA_CONTROL_ENABLE;
    _dma->DMA_CONTROL = control;
    _dma->DMA_COUNT = count;
    _dma->DMA_READ = (uint32_t)source;
    control |= LN_RP_DMA_CONTROL_ENABLE;
    _dma->DMA_CONTROL = control;
    return true;
}
/**
 * @brief
 *
 * @param count
 * @param target
 * @return true
 * @return false
 */
bool lnRpDMA::doPeripheralToMemoryTransferNoLock(int count, const uint32_t *source, const uint32_t *target, bool wrap)
{
    xAssert(!(_dma->DMA_CONTROL & LN_RP_DMA_CONTROL_ENABLE));
    _dma->DMA_COUNT = count;
    _dma->DMA_READ = (uint32_t)source;
    _dma->DMA_WRITE = (uint32_t)target;
    uint32_t control = _control;
    control &= ~(LN_RP_DMA_CONTROL_INCR_READ + LN_RP_DMA_CONTROL_INCR_WRITE);
    control |= LN_RP_DMA_CONTROL_INCR_WRITE;
    _dma->DMA_CONTROL = control;
    return true;
}
/**
 * @brief
 *
 * @param count
 * @param target
 * @return true
 * @return false
 */
bool lnRpDMA::continuePeripheralToMemoryTransferNoLock(int count, const uint32_t *target)
{
    _dma->DMA_CONTROL &= ~LN_RP_DMA_CONTROL_ENABLE;
    _dma->DMA_COUNT = count;
    _dma->DMA_WRITE = (uint32_t)target;
    _dma->DMA_CONTROL |= LN_RP_DMA_CONTROL_ENABLE;
    return true;
}
#warning this does not protect agains interrupt racing with another interrupt
#define BEGIN_ATOMIC()                                                                                                 \
    if (true || !underInterrupt)                                                                                       \
    {                                                                                                                  \
        ENTER_CRITICAL();                                                                                              \
    }
#define END_ATOMIC()                                                                                                   \
    if (true || !underInterrupt)                                                                                       \
    {                                                                                                                  \
        EXIT_CRITICAL();                                                                                               \
    }
/**
 * @brief
 * dma_channel_set_irq0_enabled
 */
void lnRpDMA::beginTransfer()
{
    BEGIN_ATOMIC();
    dmactrl->INTE0 |= (1 << _channel);
    // dmactrl->INTF0 |= (1<<_channel);
    _dma->DMA_CONTROL |= LN_RP_DMA_CONTROL_ENABLE; // this reg is  a trigger, writing it starts DMA
    END_ATOMIC();
}

/**
 * @brief
 *
 * @return uint32_t
 */
uint32_t lnRpDMA::getCurrentCount()
{
    return _dma->DMA_COUNT;
}

void lnRpDMA::disableInterrupt()
{
    BEGIN_ATOMIC();
    dmactrl->INTE0 &= ~(1 << _channel);
    END_ATOMIC();
}
void lnRpDMA::enableInterrupt()
{
    BEGIN_ATOMIC();
    dmactrl->INTE0 |= (1 << _channel);
    END_ATOMIC();
}

/**
 * @brief
 *
 */
void lnRpDMA::endTransfer()
{
    BEGIN_ATOMIC();
    dmactrl->INTE0 &= ~(1 << _channel);
    _dma->DMA_CONTROL &= ~LN_RP_DMA_CONTROL_ENABLE;
    END_ATOMIC();
}
/**
 * @brief
 *
 */
void lnRpDMA::cancelTransfer()
{
    BEGIN_ATOMIC();
    dmactrl->INTE0 &= ~(1 << _channel);
    _dma->DMA_CONTROL &= ~LN_RP_DMA_CONTROL_ENABLE;
    END_ATOMIC();
}
// EOF
