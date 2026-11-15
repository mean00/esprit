// clang-format off
#include "lnSerialTxOnly.h"
// clang-format on

/**
 * @brief Construct a new ln Serial Bp Tx Only Interrupt object
 *
 * @param instance
 */
lnSerialBpTxOnlyInterrupt::lnSerialBpTxOnlyInterrupt(uint32_t instance) : lnSerialTxOnly(instance), lnSerialBpCore(instance)
{
}
int lnSerialBpTxOnlyInterrupt::transmitNoBlock(uint32_t size, const uint8_t *buffer)
{
    xAssert(0);
    return 0;
}
/**
 * @brief
 *
 */
void lnSerialBpTxOnlyInterrupt::_interrupt()
{
    LN_USART_Registers *d = (LN_USART_Registers *)_adr;
#define ALL_INTERRUPTS (LN_USART_STAT_TC + LN_USART_STAT_TBE + LN_USART_STAT_OVERR)
    volatile int stat = d->STAT & d->CTL0 & ALL_INTERRUPTS; // filter enabled interrupt only
    if (stat & (LN_USART_STAT_TC + LN_USART_STAT_TBE))
    {
        if (_txState == txTransmittingInterrupt || _txState == txTransmittingLast)
            txInterruptHandler();
    }
}
/**
 * @brief
 *
 */
void lnSerialBpTxOnlyInterrupt::txInterruptHandler(void)
{
    LN_USART_Registers *d = (LN_USART_Registers *)_adr;
    volatile int stat = d->STAT;
    lnScratchRegister = d->CTL0;
    switch (_txState)
    {
    case txTransmittingInterrupt:
        xAssert(stat & LN_USART_STAT_TBE);
        d->DATA = (uint32_t)*_cur;
        _cur++;
        if (_cur > _tail)
            xAssert(0);
        if (_cur == _tail)
        {
            d->CTL0 |= LN_USART_CTL0_TCIE;
            d->CTL0 &= ~(LN_USART_CTL0_TBIE); // only let the Transmission complete bit
            _txState = txTransmittingLast;
        }
        return;
        break;
    case txTransmittingLast:
        if (!(stat & LN_USART_STAT_TC))
            xAssert(0); // we got here due to RX interrupt...
        d->CTL0 &= ~(LN_USART_CTL0_TBIE + LN_USART_CTL0_TCIE);
        d->STAT &= ~(LN_USART_STAT_TC);
        _txState = txTransmittingIdle;
        _txDone.give();
        return;
        break;
    default:
        xAssert(0);
        break;
    }
}
/**
 *
 * @param size
 * @param buffer
 * @return
 */
bool lnSerialBpTxOnlyInterrupt::transmit(uint32_t size, const uint8_t *buffer)
{
    // return true;
    LN_USART_Registers *d = (LN_USART_Registers *)_adr;
    _txMutex.lock();
    ENTER_CRITICAL();
    _tail = buffer + size;
    _cur = buffer;
    if (size == 1)
    {
        _txState = txTransmittingLast;
    }
    else
    {
        _txState = txTransmittingInterrupt;
    }
    d->STAT &= ~(LN_USART_STAT_TC);
    // enable TB interrupt
    _programTx();
    EXIT_CRITICAL();
    _txDone.take();
    _txMutex.unlock();
    return true;
}
/**
 * @brief
 *
 * @return true
 * @return false
 */
bool lnSerialBpTxOnlyInterrupt::_programTx()
{
    LN_USART_Registers *d = (LN_USART_Registers *)_adr;
    d->CTL0 &= ~LN_USART_CTL0_UEN;
    switch (_txState)
    {
    case txTransmittingLast:
        disableInterrupt();
        // disable TC & TE tx interrupt
        d->CTL0 &= ~(LN_USART_CTL0_TBIE + LN_USART_CTL0_TCIE);
        // enable Tx
        d->CTL2 &= ~LN_USART_CTL2_DMA_TX; // disable TX DMA
        d->CTL0 |= LN_USART_CTL0_TEN;     // enable TX
        d->CTL0 |= LN_USART_CTL0_TCIE;    // enable TBIE
        enableInterrupt(true, false);     // enable tx part of interrupt
        break;
    case txTransmittingInterrupt:
        disableInterrupt();
        // disable TC & TE tx interrupt
        d->CTL0 &= ~(LN_USART_CTL0_TBIE + LN_USART_CTL0_TCIE);
        // enable Tx
        d->CTL2 &= ~LN_USART_CTL2_DMA_TX; // disable TX DMA
        d->CTL0 |= LN_USART_CTL0_TEN;     // enable TX
        d->CTL0 |= LN_USART_CTL0_TBIE;    // enable TBIE
        enableInterrupt(true, false);     // enable tx part of interrupt
        break;
    case txTransmittingIdle:
        disableInterrupt();
        d->CTL0 &= ~LN_USART_CTL0_TEN;    // disable TX
        d->CTL2 &= ~LN_USART_CTL2_DMA_TX; // disable TX DMA
        d->CTL0 &= ~(LN_USART_CTL0_TBIE + LN_USART_CTL0_TCIE);
        enableInterrupt(false, false); // disable tx part of interrupt
        break;
    default:
        xAssert(0);
        break;
    }
    d->CTL0 |= LN_USART_CTL0_UEN;
    return true;
}
// EOF
