/*
 *  (C) 2021/2024 MEAN00 fixounet@free.fr
 *  See license file
 */

// clang-format off
#include "esprit.h"
#include "lnSerialBpCore.h"
#include "lnSerialTxOnlyDma.h"

#include "lnRingBuffer.h"
// clang-format on

#define M(x) usartMapping[instance].x

/**
 *
 * @brief
 *
 * @return true
 * @return false
 */
bool lnSerialBpTxOnlyDma::_programTx()
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
lnSerialBpTxOnlyDma::lnSerialBpTxOnlyDma(uint32_t instance)
    : lnSerialTxOnly(instance), lnSerialBpCore(instance),
      _txDma(lnDMA::DMA_MEMORY_TO_PERIPH, M(dmaEngine), M(dmaTxChannel), 8, 32)
{
}

/**
 * @brief
 *
 * @param c
 * @param it
 */

void lnSerialBpTxOnlyDma::_dmaCallback(void *c, lnDMA::DmaInterruptType it)
{
    lnSerialBpTxOnlyDma *i = (lnSerialBpTxOnlyDma *)c;
    i->txDmaCb(it);
}
/**
 * @brief
 *
 * @param it
 */
void lnSerialBpTxOnlyDma::txDmaCb(lnDMA::DmaInterruptType it)
{
    LN_USART_Registers *d = (LN_USART_Registers *)_adr;
    // disable TC & TBE
    d->CTL0 &= ~(LN_USART_CTL0_TBIE + LN_USART_CTL0_TCIE);
    // clear TC
    // nope d->STAT&=~(LN_USART_STAT_TC);
    // clear DMA
    d->CTL2 &= ~(LN_USART_CTL2_DMA_TX);
    _txState = txTransmittingIdle;
    _txDone.give();
}
/**
 * @brief
 *
 * @param size
 * @param buffer
 * @return true
 * @return false
 */
bool lnSerialBpTxOnlyDma::transmit(uint32_t size, const uint8_t *buffer)
{
    LN_USART_Registers *d = (LN_USART_Registers *)_adr;
    _txMutex.lock();        // lock uart
    _txDma.beginTransfer(); // lock dma
    ENTER_CRITICAL();
    _txState = txTransmittingDMA;
    _programTx();
    d->STAT &= ~LN_USART_STAT_TC;
    _txDma.attachCallback(_dmaCallback, this);
    EXIT_CRITICAL();

    _txDma.doMemoryToPeripheralTransferNoLock(size, (uint16_t *)buffer, (uint16_t *)&(d->DATA), false);

    _txDone.take();
    _txDma.endTransfer();

    // Wait busy bit to clear out
    while (!(d->STAT & LN_USART_STAT_TC))
    {
        __asm__("nop" ::);
    }
    d->STAT &= ~(LN_USART_STAT_TC);

    _txMutex.unlock();
    return true;
}
// EOF
