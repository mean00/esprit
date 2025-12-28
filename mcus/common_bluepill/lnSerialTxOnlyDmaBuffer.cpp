

/**
 */
#include "esprit.h"
#include "lnSerialBpCore.h"
#include "lnSerialTxOnlyDma.h"

#define M(x) usartMapping[instance].x
/**
 * @brief
 *
 */

/**
 * @brief
 *
 * @return true
 * @return false
 */
bool lnSerialBpTxOnlyBufferedDma::_programTx()
{
    LN_USART_Registers *d = (LN_USART_Registers *)_adr;
    // d->CTL0 &= ~LN_USART_CTL0_UEN;
    ENTER_CRITICAL();
    switch (_txState)
    {
    case txTransmittingDMA:
        disableInterrupt();
        d->CTL0 |= (LN_USART_CTL0_TBIE + LN_USART_CTL0_TCIE);
        d->CTL0 |= LN_USART_CTL0_TEN;    // enable TX
        d->CTL2 |= LN_USART_CTL2_DMA_TX; // enable TX DMA
        enableInterrupt(false, false);
        break;
    default:
        xAssert(0);
        break;
    }
    EXIT_CRITICAL();
    // d->CTL0 |= LN_USART_CTL0_UEN;
    return true;
}

/**
 * @brief Construct a new ln Serial Bp Tx Only Dma object
 *
 * @param instance
 */
lnSerialBpTxOnlyBufferedDma::lnSerialBpTxOnlyBufferedDma(int instance, int txBufferSize)
    : lnSerialTxOnly(instance), lnSerialBpCore(instance),
      _txDma(lnDMA::DMA_MEMORY_TO_PERIPH, M(dmaEngine), M(dmaTxChannel), 8, 32), _txRingBuffer(txBufferSize)
{
    _txDma.beginTransfer();
}
/**
 * @brief Destroy the ln Serial Bp Tx Only Buffered Dma::ln Serial Bp Tx Only Buffered Dma object
 *
 */
lnSerialBpTxOnlyBufferedDma::~lnSerialBpTxOnlyBufferedDma()
{
    _txDma.endTransfer();
}
/**
 * @brief
 *
 * @param c
 * @param it
 */

void lnSerialBpTxOnlyBufferedDma::_dmaCallback2(void *c, lnDMA::DmaInterruptType it)
{
    lnSerialBpTxOnlyBufferedDma *i = (lnSerialBpTxOnlyBufferedDma *)c;
    i->txDmaCb2(it);
}
/**
 * @brief
 *
 * @param it
 */
void lnSerialBpTxOnlyBufferedDma::txDmaCb2(lnDMA::DmaInterruptType it)
{
    uint8_t *to;
    _txRingBuffer.consume(_inFlight);
    _inFlight = 0;
    if (_cb == NULL)
        _txRingSem.give(); // space freed, wake up thread

    int nb = _txRingBuffer.getReadPointer(&to);
    if (!nb) // done !
    {
        _txing = false;
        if (_cb != NULL)
            _cb(_cbCookie, lnSerialCore::txDone);
        return;
    }
    // send remainer
    LN_USART_Registers *d = (LN_USART_Registers *)_adr;
    _inFlight = nb;
    _txDma.doMemoryToPeripheralTransferNoLock(nb, (uint16_t *)to, (uint16_t *)&(d->DATA), false);
}
/**
 * @brief
 *
 * @param size
 * @param buffer
 * @return int
 */
int lnSerialBpTxOnlyBufferedDma::transmitNoBlock(int size, const uint8_t *buffer)
{
    int processed = 0;
    while (size != 0)
    {
        ENTER_CRITICAL();
        int nb = _txRingBuffer.free();
        if (nb == 0)
        {
            EXIT_CRITICAL();
            return processed;
        }
        if (nb > size)
            nb = size;
        int inc = _txRingBuffer.put(nb, buffer);
        buffer += inc;
        size -= inc;
        processed += inc;
        if (!_txing)
            igniteTx();
        EXIT_CRITICAL();
    }
    return processed;
}

/**
 * @brief
 *
 * @param size
 * @param buffer
 * @return true
 * @return false
 */
bool lnSerialBpTxOnlyBufferedDma::transmit(int size, const uint8_t *buffer)
{
    while (size)
    {
        ENTER_CRITICAL();
        int nb = _txRingBuffer.free();
        if (!nb)
        {
            _txRingSem.tryTake(); // wait for a fresh sem post, not an old one
            EXIT_CRITICAL();
            _txRingSem.take();
            continue;
        }
        if (nb > size)
            nb = size;
        int inc = _txRingBuffer.put(nb, buffer);
        buffer += inc, size -= inc;

        if (!_txing)
            igniteTx();
        EXIT_CRITICAL();
    }
    return true;
}
/**
 * @brief
 *
 */
void lnSerialBpTxOnlyBufferedDma::igniteTx() // if we get here, no active transer if going on
{
    uint8_t *to = NULL;
    int nb = _txRingBuffer.getReadPointer(&to);
    xAssert(nb);
    _inFlight = nb;
    _txing = true;
    LN_USART_Registers *d = (LN_USART_Registers *)_adr;

    ENTER_CRITICAL();
    _txState = txTransmittingDMA;
    _programTx();
    d->STAT &= ~LN_USART_STAT_TC;
    _txDma.attachCallback(_dmaCallback2, this);
    EXIT_CRITICAL();

    _txDma.doMemoryToPeripheralTransferNoLock(nb, (uint16_t *)to, (uint16_t *)&(d->DATA), false);
}
// EOF
