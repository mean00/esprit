/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */

// clang-format off
#include "esprit.h"
#include "lnSerialBpCore.h"
#include "lnSerialTxOnlyDma.h"
#include "lnSerialRxTxDma.h"
#include "lnSerialRxTx.h"
// clang-format on
/**
 *
 * @param instance
 * @param irq
 */
#define M(x) usartMapping[instance].x
// #define DISABLE_TIMER_COLLECTOR
#define CHECK_OVERFLOW()                                                                                               \
    {                                                                                                                  \
        if ((_rxHead - _rxTail) > _rxBufferSize)                                                                       \
        {                                                                                                              \
            nb_overflow++;                                                                                             \
            _rxTail = _rxHead;                                                                                         \
        }                                                                                                              \
    }
static int nb_overflow = 0;
/*
 */

/**
 * @brief
 *
 */
void lnSerialBpRxTxDma::_interrupt()
{
    xAssert(0);
}
/**
 * @brief Construct a new ln Serial Bp Rx Tx Dma::ln Serial Bp Rx Tx Dma object
 *
 * @param instance
 * @param bufferSize
 */
lnSerialBpRxTxDma::lnSerialBpRxTxDma(uint32_t instance, uint32_t bufferSize)
    : lnSerialBpTxOnlyBufferedDma(instance, bufferSize >> 1), lnSerialRxTx(instance),
      _rxDma(lnDMA::DMA_PERIPH_TO_MEMORY, M(dmaEngine), M(dmaRxChannel), 32, 8), _timer(this)
{
    _rxBufferSize = bufferSize;
    _rxBuffer = new uint8_t[_rxBufferSize];
    _rxEnabled = false;
    _rxDma.attachCallback(_rxDmaCb, this);
    _rxHead = _rxTail = 0;
    _rxMask = _rxBufferSize - 1;
    _timer.init("uart", 20);
    xAssert((_rxBufferSize & (~_rxMask)) == _rxBufferSize); // must be a power of 2
}
/**
 * @brief Destroy the ln Serial Bp Rx Tx::ln Serial Bp Rx Tx object
 *
 */
lnSerialBpRxTxDma::~lnSerialBpRxTxDma()
{
    delete[] _rxBuffer;
    _rxBuffer = NULL;
}
/**
 * @brief
 *
 * @param max
 * @param to
 * @return int
 */
int lnSerialBpRxTxDma::read(uint32_t max, uint8_t *to)
{
    uint8_t *ptr;

    ENTER_CRITICAL();
    int nb = getReadPointer(&ptr);
    EXIT_CRITICAL();
    if (nb > max)
        nb = max;
    memcpy(to, ptr, nb);

    ENTER_CRITICAL();
    consume(nb);
    EXIT_CRITICAL();
    return nb;
}
/**
 * @brief
 *
 */
void lnSerialBpRxTxDma::purgeRx()
{
    LN_USART_Registers *d = (LN_USART_Registers *)_adr;
    ENTER_CRITICAL();
    disableRxDmaInterrupt();
    _rxHead = _rxTail = 0;
    volatile uint32_t stat = d->STAT;
    while (stat & LN_USART_STAT_RBNE)
    {
        lnScratchRegister = d->DATA;
        stat = d->STAT;
    }
    EXIT_CRITICAL();
}
/**
 * @brief
 *
 * @param enabled
 * @return true
 * @return false
 */
bool lnSerialBpRxTxDma::enableRx(bool enabled)
{
    LN_USART_Registers *d = (LN_USART_Registers *)_adr;

    if (enabled)
    {
        _rxHead = _rxTail = 0;
        _rxDma.cancelTransfer();
        ENTER_CRITICAL();
        d->CTL0 |= (LN_USART_CTL0_RBNEIE | LN_USART_CTL0_REN);
        d->CTL2 |= LN_USART_CTL2_DMA_RX;
        _rxEnabled = true;
        EXIT_CRITICAL();
        startRxDma();
        _timer.start();
    }
    else
    {
        disableRxDmaInterrupt();
        _rxDma.pause();
        ENTER_CRITICAL();
        _timer.stop();
        _rxEnabled = false;
        d->CTL0 &= ~(LN_USART_CTL0_REN | LN_USART_CTL0_RBNEIE);
        d->CTL2 &= ~LN_USART_CTL2_DMA_RX;
        EXIT_CRITICAL();
    }
    return true;
}
/**
 * @brief
 *
 */
void lnSerialBpRxTxDma::timerCallback()
{
    ENTER_CRITICAL(); // we are in a task, need to block interrupt and other tasks
#ifndef DISABLE_TIMER_COLLECTOR
    checkForNewData();
#endif
    EXIT_CRITICAL();
}
/**
 * @brief
 *
 * @return true
 * @return false
 */
void lnSerialBpRxTxDma::enableRxDmaInterrupt()
{
    _rxDma.enableInterrupt();
}
/**
 * @brief
 *
 * @return true
 * @return false
 */
void lnSerialBpRxTxDma::disableRxDmaInterrupt()
{
    _rxDma.disableInterrupt();
}
/**
 * @brief
 *
 */
void lnSerialBpRxTxDma::startRxDma()
{
    LN_USART_Registers *d = (LN_USART_Registers *)_adr;
    _rxDma.attachCallback(_rxDmaCb, this);
    _rxDma.beginTransfer(); // lock dma
    _rxDma.doPeripheralToMemoryTransferNoLock(_rxBufferSize, (const uint16_t *)_rxBuffer, (const uint16_t *)&(d->DATA),
                                              true, true);
}

/**
 * @brief Construct a new ln Serial Bp Rx Tx::ln Serial Bp Rx Tx object
 *
 * @param instance
 */
void lnSerialBpRxTxDma::_rxDmaCb(void *me, lnDMA::DmaInterruptType type)
{
    lnSerialBpRxTxDma *serial = (lnSerialBpRxTxDma *)me;
    serial->rxDma(type);
}

/**
 * @brief
 *
 * @param to
 * @return int
 */
int lnSerialBpRxTxDma::getReadPointer(uint8_t **to)
{
    ENTER_CRITICAL();
    int h = _rxHead & _rxMask;
    int t = _rxTail & _rxMask;

    int nb;
    if (h >= t)
    {
        nb = _rxHead - _rxTail;
        CHECK_OVERFLOW();
    }
    else
    {
        nb = _rxBufferSize - t;
    }
    *to = _rxBuffer + t;
    EXIT_CRITICAL();
    return nb;
}
/**
 * @brief
 *
 * @param n
 */
void lnSerialBpRxTxDma::consume(uint32_t n)
{
    ENTER_CRITICAL();
    xAssert(n <= _rxBufferSize);
    _rxTail += n;
    if (_rxTail > _rxBufferSize && _rxHead > _rxBufferSize)
    {
        _rxTail -= _rxBufferSize;
        _rxHead -= _rxBufferSize;
    }
    EXIT_CRITICAL();
}

/**
 * @brief This is not interrupt safe, must be called from interrupt or with interrupt disabled
 *
 */
void lnSerialBpRxTxDma::checkForNewData()
{
    if (!_rxEnabled)
        return;

    uint32_t count = _rxDma.getCurrentCount(); // transfer pending, between N and 1, 0 means transfer done
    count = _rxBufferSize - count;             // this is the # of bytes transfered already
    uint32_t maskedHead = _rxHead & _rxMask;
    if (count == maskedHead)
    {
        return;
    }
    uint32_t t = _rxHead & _rxMask;
    _rxHead = _rxHead & ~_rxMask;
    _rxHead += count;
    CHECK_OVERFLOW();
    if (t >= count)
    {
        _rxHead += _rxBufferSize;
        CHECK_OVERFLOW();
    }
    xAssert(_cb);
    _cb(_cbCookie, lnSerialCore::dataAvailable);
}
/**
 * @brief
 *
 * @param type
 */
int oldHead = 0;
void lnSerialBpRxTxDma::rxDma(lnDMA::DmaInterruptType type)
{
#if 0    
    checkForNewData();
    return;
#endif
    oldHead = _rxHead;
    int current = _rxHead & _rxMask;
    bool newData = false;
    switch (type)
    {
    case lnDMA::DMA_INTERRUPT_HALF:
        if (current != _rxBufferSize >> 1)
        {
            newData = true;
            _rxHead = _rxHead & ~_rxMask;
            _rxHead += _rxBufferSize >> 1;
            CHECK_OVERFLOW();
        }
        break;
    case lnDMA::DMA_INTERRUPT_FULL:
        if (current != 0)
        {
            newData = true;
            _rxHead = _rxHead & ~_rxMask;
            _rxHead += _rxBufferSize;
            CHECK_OVERFLOW();
        }
        break;
    default:
        xAssert(0);
    }
    if (newData)
    {
        _timer.restart();
        xAssert(_cb);
        _cb(_cbCookie, lnSerialCore::dataAvailable);
    }
}
/**
 * @brief
 *
 */
void timerLink::timerCallback()
{
    _parent->timerCallback();
}

/**
 * @brief Create a Ln Serial Rx Tx object
 *
 * @param instance
 * @param rxBufferSize
 * @param dma
 * @return lnSerialRxTx*
 */

lnSerialRxTx *createLnSerialRxTx(uint32_t instance, uint32_t rxBufferSize, bool dma)
{
    if (dma)
        return new lnSerialBpRxTxDma(instance, rxBufferSize);
    return new lnSerialBpRxTx(instance, rxBufferSize);
}

// EOF
