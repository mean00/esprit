/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */

#include "lnI2C_impl.h"
#include "esprit.h"
#include "lnI2C.h"
#include "lnI2C_priv.h"

static const LN_I2C_Registers *aI2C0 = (LN_I2C_Registers *)LN_I2C0_ADR;
static const LN_I2C_Registers *aI2C1 = (LN_I2C_Registers *)LN_I2C1_ADR;

volatile uint32_t *LN_I2C0_FMPCFG = (volatile uint32_t *)(LN_I2C0_ADR + 0x90);
volatile uint32_t *LN_I2C1_FMPCFG = (volatile uint32_t *)(LN_I2C1_ADR + 0x90);

#define I2C_USES_INTERRUPT

struct LN_I2C_DESCRIPTOR
{
    LN_I2C_Registers *adr;
    lnPin _scl;
    lnPin _sda;
    LnIRQ _irqEv;
    LnIRQ _irqErr;
    int _dmaEngine;
    int _dmaChannelRx;
    int _dmaChannelTx;
};

static const LN_I2C_DESCRIPTOR i2c_descriptors[2] = {
    {(LN_I2C_Registers *)LN_I2C0_ADR, PB6, PB7, LN_IRQ_I2C0_EV, LN_IRQ_I2C0_ER, 0, 6, 5},
    {(LN_I2C_Registers *)LN_I2C1_ADR, PB10, PB11, LN_IRQ_I2C1_EV, LN_IRQ_I2C1_ER, 0, 4, 3}};
static uint32_t i2c_stray_interrupt[2];
volatile uint32_t lastI2cStat;
volatile uint32_t stat1;
//
#define READ_STAT0()                                                                                                   \
    {                                                                                                                  \
        lastI2cStat = (_d->adr->STAT0);                                                                                \
        if (!lastI2cStat)                                                                                              \
            return;                                                                                                    \
    }
#define READ_STAT1() stat1 = (_d->adr->STAT1);

#if 1
#define PEDANTIC()                                                                                                     \
    {                                                                                                                  \
    }
#define SET_STATE(x)                                                                                                   \
    {                                                                                                                  \
        _state = x;                                                                                                    \
    }
#define SET_DONE(x)                                                                                                    \
    {                                                                                                                  \
        _result = x;                                                                                                   \
        lnDisableInterrupt(_d->_irqErr);                                                                               \
        lnDisableInterrupt(_d->_irqEv);                                                                                \
        _sem.give();                                                                                                   \
    }
#else
#define PEDANTIC()                                                                                                     \
    {                                                                                                                  \
        xAssert(0);                                                                                                    \
    }
#define SET_STATE(x)                                                                                                   \
    {                                                                                                                  \
        _state = x;                                                                                                    \
    }
#define SET_DONE(x)                                                                                                    \
    {                                                                                                                  \
        _result = x;                                                                                                   \
        if (!x)                                                                                                        \
            xAssert(0);                                                                                                \
        lnDisableInterrupt(_d->_irqErr);                                                                               \
        lnDisableInterrupt(_d->_irqEv);                                                                                \
        _sem.give();                                                                                                   \
    }
#endif

// #define LN_I2C_STAT0_ERROR_MASK (LN_I2C_STAT0_LOSTARB | LN_I2C_STAT0_AERR | LN_I2C_STAT0_BERR | LN_I2C_STAT0_OUERR)
#warning FIXME
#define LN_I2C_STAT0_ERROR_MASK (0x1F << 8)

lnTwoWire *irqHandler[2] = {NULL, NULL};
/*
 *
 */
lnI2C *lnI2C::create(int instance, int speed)
{
    lnPeripherals::enable((Peripherals)(pI2C0 + instance));
    return new lnTwoWire(instance, speed);
}
/**
 *
 * @param eventEnabled
 * @param dmaEnabled
 * @param txEmptyEnabled
 * @return
 */
void lnTwoWire::setInterruptMode(bool eventEnabled, bool dmaEnabled, bool txEmptyEnabled)
{

    uint32_t stat1 = _d->adr->CTL1;

    if (dmaEnabled)
    {
        xAssert(!eventEnabled);
        xAssert(!txEmptyEnabled);
        stat1 &= ~(LN_I2C_CTL1_EVIE + LN_I2C_CTL1_BUFIE);
        stat1 |= LN_I2C_CTL1_DMAON;
    }
    else
    {
        xAssert(eventEnabled);
        stat1 &= ~LN_I2C_CTL1_DMAON;

        stat1 |= LN_I2C_CTL1_EVIE;
        // if (txEmptyEnabled)
        // stat1 |= LN_I2C_CTL1_BUFIE;
        // else
        stat1 &= ~LN_I2C_CTL1_BUFIE;
    }
    _d->adr->CTL1 = stat1 | LN_I2C_CTL1_ERRIE;
}
/**
 *
 */
void lnTwoWire::startTxIrq()
{

    xAssert(irqHandler[_instance] == NULL);
    irqHandler[_instance] = this;

    setInterruptMode(true, false, false); // event, no dma, no tx empty

    lnScratchRegister = _d->adr->STAT0;
    lnScratchRegister = _d->adr->STAT1;

    _dmaTx.attachCallback(dmaTxDone_, this);

    lnEnableInterrupt(_d->_irqErr);
    lnEnableInterrupt(_d->_irqEv);
}
void lnTwoWire::startRxIrq()
{

    xAssert(irqHandler[_instance] == NULL);
    irqHandler[_instance] = this;

    setInterruptMode(true, false, false); // event, no dma, no tx empty

    lnScratchRegister = _d->adr->STAT0;
    lnScratchRegister = _d->adr->STAT1;

    lnEnableInterrupt(_d->_irqErr);
    lnEnableInterrupt(_d->_irqEv);
}
/**
 *
 */
void lnTwoWire::stopIrq()
{
    lnDisableInterrupt(_d->_irqErr);
    lnDisableInterrupt(_d->_irqEv);
    _d->adr->CTL1 &= ~(LN_I2C_CTL1_EVIE + LN_I2C_CTL1_BUFIE + LN_I2C_CTL1_DMAON);
    irqHandler[_instance] = NULL;
    _dmaTx.detachCallback();
}

/**
 *
 * @param reg
 * @param bit
 * @return
 */
bool waitStat0BitSet(LN_I2C_Registers *reg, uint32_t bit)
{
    while (1)
    {
        uint32_t v = (reg->STAT0);
        if (v & bit)
            return true;
        if (v & (LN_I2C_STAT0_ERROR_MASK))
        {
            Logger("I2C:Stat error 0x%x\n", v);
            // clear error
            v &= ~LN_I2C_STAT0_ERROR_MASK;
            reg->STAT0 = v;
            return false;
        }
    }
}
bool waitStat0BitClear(LN_I2C_Registers *reg, uint32_t bit)
{
    while (1)
    {
        uint32_t v = (reg->STAT0);
        if (!(v & bit))
            return true;
        if (v & (LN_I2C_STAT0_ERROR_MASK))
        {
            Logger("I2C:Stat error 0x%x\n", v);
            // clear error
            v &= ~LN_I2C_STAT0_ERROR_MASK;
            reg->STAT0 = v;
            return false;
        }
    }
}
static bool waitCTL0BitClear(LN_I2C_Registers *reg, uint32_t bit)
{
    uint32_t wait = 10000;
    while (--wait != 0)
    {
        uint32_t v = (reg->CTL0);
        if (!(v & bit))
            return true;
    }
    return false;
}
/**
 *
 * @param instance
 * @param speed
 */
#define M(x) i2c_descriptors[instance].x

lnTwoWire::lnTwoWire(int instance, int speed)
    : _dmaTx(lnDMA::DMA_MEMORY_TO_PERIPH, M(_dmaEngine), M(_dmaChannelTx), 8, 16),
      _dmaRx(lnDMA::DMA_PERIPH_TO_MEMORY, M(_dmaEngine), M(_dmaChannelRx), 8, 8)
{
    _instance = instance;
    _speed = speed;
    _d = i2c_descriptors + _instance;
    _session = NULL;
    SET_STATE(I2C_IDLE);
}
/**
 *
 */
lnTwoWire::~lnTwoWire()
{
    _session = NULL;
}
/**
 *
 * @param target
 * @return
 */
bool lnTwoWire::begin(int target)
{
    // reset peripheral
    Peripherals periph = (Peripherals)(pI2C0 + _instance);

    LN_I2C_Registers *adr = _d->adr;

    lnPeripherals::reset(periph);
    adr->CTL0 = LN_I2C_CTL0_SRESET;
    xDelay(1);
    adr->CTL0 = 0;
    if (target)
        _targetAddress = target;

    // set pin
    lnPinMode(_d->_scl, lnALTERNATE_OD);
    lnPinMode(_d->_sda, lnALTERNATE_OD);
    // back to default

    adr->CTL1 = 0;
    setSpeed(_speed);
    adr->CTL0 = LN_I2C_CTL0_I2CEN; // And go!

    return true;
}

/**
 *
 * @param speed
 */
void lnTwoWire::setSpeed(int newSpeed)
{

    LN_I2C_Registers *adr = _d->adr;
    Peripherals periph = (Peripherals)(pI2C0 + _instance);

    uint32_t ctl1 = adr->CTL1;
    ctl1 &= ~(0x3f);
    ctl1 |= lnPeripherals::getClock(periph) / 1000000;
    ; // 54 mhz apb1 max
    adr->CTL1 = ctl1;
    _speed = newSpeed;

    int speed = lnPeripherals::getClock(periph);
    if (!_speed)
        speed = speed / (100 * 1000); // default speed is 100k
    else
        speed = speed / newSpeed;

#define SPDMAX ((1 << 12) - 1)
    if (speed > SPDMAX)
        speed = SPDMAX;
    adr->CKCFG = speed;
}
/**
 */
#ifdef I2C_USES_INTERRUPT
bool lnTwoWire::multiWrite(int target, uint32_t nbSeqn, const uint32_t *seqLength, const uint8_t **seqData)
{
    volatile uint32_t stat1, stat0;
    // Send start
    _dmaTx.beginTransfer(); // lock down i2c also, deals with re-entrency
    clearup_state();
    LN_I2C_Registers *adr = _d->adr;
    lnI2CSession session(target, nbSeqn, seqLength, seqData);
    _session = &session;

    SET_STATE(I2C_TX_START);
    adr->CTL0 |= LN_I2C_CTL0_ACKEN; // send start
    adr->CTL0 |= LN_I2C_CTL0_START; // send start
    startTxIrq();
    bool r = _sem.take(100);
    _dmaTx.endTransfer();
    stopIrq();
    _session = NULL;
    SET_STATE(I2C_IDLE);
    if (!r)
    {
        Logger("multiwrite: I2C write timeout\n");
        return false;
    }
    if (_result)
    {
        if (!waitCTL0BitClear(adr, LN_I2C_CTL0_STOP))
        {
            _result = false;
            PEDANTIC();
        }
    }
    return _result;
}
#endif

/**
 *
 * @param target
 * @param n
 * @param data
 * @return
 */
bool lnTwoWire::write(int target, uint32_t n, const uint8_t *data)
{
    return multiWrite(target, 1, &n, &data);
}
void lnTwoWire::clearup_state()
{
    LN_I2C_Registers *adr = _d->adr;
    volatile uint32_t stat0, state1;
    stat0 = adr->DATA;
    stat0 = adr->DATA; // purge received stuff
                       //
    stat0 = adr->STAT0;
    READ_STAT1();
    // read stat 0 & stat 1
    stat0 = adr->CTL0;
    stat0 &= ~(LN_I2C_CTL0_ACKEN + LN_I2C_CTL0_POAP + LN_I2C_CTL0_STOP + LN_I2C_CTL0_START);
    adr->CTL0 = stat0;

    stat0 = adr->STAT0;
    READ_STAT1();

    stat0 = _d->adr->STAT0;
    stat0 &= ~(LN_I2C_STAT0_ERROR_MASK);
    _d->adr->STAT0 = stat0;
}
/**
 *
 * @param target
 * @param n
 * @param data
 * @return
 */
bool lnTwoWire::multiRead(int target, uint32_t nbSeqn, const uint32_t *seqLength, uint8_t **seqData)
{
    volatile uint32_t stat1, stat0;
    LN_I2C_Registers *adr = _d->adr;
    _dmaRx.beginTransfer(); // we dont actually use dma, but we use the dma mutex to protect against re-entrency
                            // ugly, i know
    clearup_state();
    lnI2CSession session(target, nbSeqn, seqLength, (const uint8_t **)seqData);

    //  xAssert(session.total_bytes < 3); // above is buggy, dont care for now

    _session = &session;

    SET_STATE(I2C_RX_START);

    startRxIrq();
    switch (_session->total_bytes)
    {
    case 1:
        break;
    case 0:
        break;
    default:
        adr->CTL0 |= LN_I2C_CTL0_POAP; // send start
        break;
    }
    adr->CTL0 |= LN_I2C_CTL0_ACKEN; // send start, auto ack
    adr->CTL0 |= LN_I2C_CTL0_START; // send start, auto ack
    bool r = _sem.take(100);
    _dmaRx.endTransfer();
    stopIrq();
    _session = NULL;
    SET_STATE(I2C_IDLE);
    if (!r)
    {
        Logger("multiread: I2C write timeout\n");
        return false;
    }
    if (_result)
    {
    }
    return _result;
}
/**
 *
 * @param target
 * @param n
 * @param data
 * @return
 */
bool lnTwoWire::read(int target, uint32_t n, uint8_t *data)
{
    return lnTwoWire::multiRead(target, 1, &n, &data);
}
/**
 *
 * @param c
 */
void lnTwoWire::dmaTxDone_(void *c, lnDMA::DmaInterruptType typ)
{
    lnTwoWire *i = (lnTwoWire *)c;
    i->dmaTxDone();
}

/**
 *
 * @param c
 */
void lnTwoWire::dmaTxDone()
{
    _session->curTransaction++;
    if (!initiateTx())
    {
        SET_STATE(I2C_TX_STOP);
        setInterruptMode(true, false, false); // event, no dma, no tx
        lnScratchRegister = _d->adr->STAT0 + _d->adr->STAT1;
    }
    // done !
}
/**
 * @return  true if we continue, false if the last one has been sent
 * @return
 */
bool lnTwoWire::initiateTx()
{
    if (_session->curTransaction >= _session->nbTransaction)
        return false;

    _session->curIndex = 0; // we are starting a new sequence anyway
    int t = _session->curTransaction;
    const uint8_t *data = _session->transactionData[t];
    int size = _session->transactionSize[t];

    xAssert(size <= 65535);

    if (size < 8) // plain interrupt
    {
        SET_STATE(I2C_TX_DATA);
        setInterruptMode(true, false, true); // event, no DMA, tx interrupt
        return sendNext();                   // dont setup DMA transfer for small transfer
    }
    // DMA ON
    _dmaTx.setWordSize(8, 16);
    SET_STATE(I2C_TX_DATA_DMA);
    setInterruptMode(false, true, false); //   no event, DMA, no tx interrupt,
    _dmaTx.doMemoryToPeripheralTransferNoLock(size, (uint16_t *)data, (uint16_t *)&(_d->adr->DATA), false);
    // all done!
    return true;
}

/**
 *
 * @return  true if we continue, false if the last one has been sent
 */
bool lnTwoWire::sendNext()
{
    int t = _session->curTransaction;
    const uint8_t *data = _session->transactionData[t];
    int size = _session->transactionSize[t];

    if (_session->curTransaction >= _session->nbTransaction)
    {
        _session->curIndex = 0; // all done
        return false;
    }
    if (_session->curIndex >= size) // next transaction ?
    {
        _session->curTransaction++;
        _session->curIndex = 0;
        return initiateTx();
    }
    _d->adr->DATA = data[_session->curIndex++];
    return true;
}
/**
 *
 * @return
 */
bool lnTwoWire::receiveNext()
{
    // current transaction
    int t = _session->curTransaction;
    uint8_t *data = (uint8_t *)_session->transactionData[t];
    int size = _session->transactionSize[t];

    // all done ?
    if (_session->curTransaction >= _session->nbTransaction)
    {
        return false;
    }

    // store current...
    xAssert(_session->curIndex < size);
    xAssert(t < _session->nbTransaction);

    data[_session->curIndex++] = _d->adr->DATA & 0xff;
    _session->current_offset++;
    if (_session->curIndex < size)
    {
        return true;
    }

    _session->curTransaction++;
    _session->curIndex = 0;
    if (_session->curTransaction >= _session->nbTransaction)
        return false;
    return true;
}
/**
 *
 * evt:
 *      0 : Event
 *      1:  Error
 * @param evt
 * @return
 */
/**
 *
 */
void lnTwoWire::irq(int evt)
{
    xAssert(_session);
    if (evt != 0)
    {
        // ignore stray error interrupts
        switch (_state)
        {
        case I2C_RX_END:
        case I2C_TX_END:
        case I2C_IDLE:
            i2c_stray_interrupt[this->_instance]++;
            return;
            break;
        default:
            break;
        }
        lnDisableInterrupt(_d->_irqErr);
        lnDisableInterrupt(_d->_irqEv);
        uint32_t stat0 = _d->adr->STAT0;
        stat0 &= ~(LN_I2C_STAT0_ERROR_MASK); // clear error
        _d->adr->STAT0 = stat0;
        // SET_DONE(false);
        return;
    }
    if ((uint32_t)_state & LN_RX_I2C_STATE_OFFSET)
        irqRx();
    else
        irqTx();
}
/**
 *
 *
 */
void lnTwoWire::irqRx()
{
    switch (_state)
    {
    default:
        xAssert(0);
        break;
    case I2C_RX_START: {
        READ_STAT0();
        xAssert(lastI2cStat & LN_I2C_STAT0_SBSEND);
        _d->adr->DATA = ((_session->target & 0x7F) << 1) + 1;
        SET_STATE(I2C_RX_ADDR_SENT);
    }
        return;
        break;
    case I2C_RX_ADDR_SENT: {
        if (_session->total_bytes < 3)
        {
            _d->adr->CTL0 &= ~LN_I2C_CTL0_ACKEN;
        }
        READ_STAT0();
        stat1 = _d->adr->STAT1; // Clear bit
        if (!(lastI2cStat & LN_I2C_STAT0_ADDSEND))
        {
            xAssert(0);
        }
        if (_session->total_bytes == 1)
        {
            SET_STATE(I2C_RX_DATA_1_BYTE)
        }
        else
        {
            SET_STATE(I2C_RX_DATA)
        }
        setInterruptMode(true, false, true); // event, no DMA, rx interrupt
        _d->adr->CTL1 |= (LN_I2C_CTL1_BUFIE);
        break;
        return;
    }
    break;
    /*
     * Special case when N=1
     *
     */
    case I2C_RX_DATA_1_BYTE: {
        READ_STAT0();
        READ_STAT1();
        xAssert(lastI2cStat & LN_I2C_STAT0_RBNE);
        receiveNext();
        uint32_t u = _d->adr->CTL0;
        u |= LN_I2C_CTL0_STOP;
        _d->adr->CTL0 = u;
        SET_STATE(I2C_RX_END);
        SET_DONE(true);
        return;
    }
    break;
    case I2C_RX_DATA_2_BYTES: {
        receiveNext();
        SET_STATE(I2C_RX_END);
        SET_DONE(true);
        return;
    }
    break;
    case I2C_RX_DATA: {
        READ_STAT0();
        if (_session->current_offset += 2 == _session->total_bytes)
        {
            READ_STAT0();
            READ_STAT1();
            uint32_t u = _d->adr->CTL0;
            u &= ~LN_I2C_CTL0_ACKEN;
            _d->adr->CTL0 = u;
            u |= LN_I2C_CTL0_STOP;
            _d->adr->CTL0 = u;
            receiveNext();
            SET_STATE(I2C_RX_DATA_2_BYTES); // dont read the byte!
            return;
        }
        xAssert(lastI2cStat & LN_I2C_STAT0_RBNE);
        receiveNext();
        return;
    }
    break;
    }
}
/**
 *
 *
 */
void lnTwoWire::irqTx()
{
    switch (_state)
    {
    default:
        xAssert(0);
        break;
    case I2C_TX_START: {
        READ_STAT0();
        xAssert(lastI2cStat & LN_I2C_STAT0_SBSEND);
        _d->adr->DATA = ((_session->target & 0x7F) << 1);
        _d->adr->CTL1 |= (LN_I2C_CTL1_BUFIE); // enable tx
        SET_STATE(I2C_TX_ADDR_SENT);
    }
        return;
        break;
    case I2C_TX_ADDR_SENT: {
        READ_STAT0();
        stat1 = _d->adr->STAT1; // Clear bit
        if (!(lastI2cStat & LN_I2C_STAT0_ADDSEND))
        {
            xAssert(0);
        }
        xAssert(_d->adr->STAT0 & LN_I2C_STAT0_TBE);
        if (!initiateTx())
        {
            SET_DONE(false);
        }
    }
        return;
        break;
    case I2C_TX_DATA:
        READ_STAT0();
        xAssert(_d->adr->STAT0 & LN_I2C_STAT0_TBE);
        if (!sendNext())
        {
            SET_STATE(I2C_TX_STOP);
            _d->adr->CTL1 &= ~(LN_I2C_CTL1_BUFIE);
        }
        return;
        break;

    case I2C_TX_STOP:
        READ_STAT0();
        xAssert(lastI2cStat & LN_I2C_STAT0_BTC);
        _d->adr->CTL0 |= LN_I2C_CTL0_STOP;
        SET_DONE(true);
        SET_STATE(I2C_TX_END);
        return;
        break;
    }
}

/**
 *
 * @param instance
 * @param error
 */
void i2cIrqHandler(int instance, bool error)
{
    lnTwoWire *i = irqHandler[instance];
    xAssert(i);
    i->irq((int)error);
}

#ifndef I2C_USES_INTERRUPT
bool lnTwoWire::multiWrite(int target, int nbSeqn, int *seqLength, uint8_t **seqData)
{
    volatile uint32_t stat1, stat0;
    // Send start

    LN_I2C_Registers *adr = _d->adr;
    lnI2CSession session(target, nbSeqn, seqLength, seqData);
    _session = &session;
    adr->CTL0 |= LN_I2C_CTL0_START; // send start
    waitStat0BitSet(adr, LN_I2C_STAT0_SBSEND);
    // Send address
    adr->DATA = ((target & 0x7F) << 1);
    if (!waitStat0BitSet(adr, LN_I2C_STAT0_ADDSEND))
        return false;
    stat1 = adr->STAT1;

    // wait for completion
    while (1)
    {
        if (!waitStat0BitSet(adr, LN_I2C_STAT0_TBE))
            xAssert(0);
        if (sendNext())
            continue;
        break;
    }
    // wait for BTC
    if (!waitStat0BitSet(adr, LN_I2C_STAT0_BTC))
        return false;
    // send stop
    adr->CTL0 |= LN_I2C_CTL0_STOP;
    if (!waitStat0BitClear(adr, LN_I2C_CTL0_STOP))
        return false;
    return true;
}
#endif

// EOF
