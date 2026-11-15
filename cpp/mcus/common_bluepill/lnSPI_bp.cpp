/*
 *  (C) 2021/2024 MEAN00 fixounet@free.fr
 *  See license file
 */

#include "lnSPI_bp.h"
#include "esprit.h"
#include "lnGPIO.h"
#include "lnPeripheral_priv.h"
#include "lnSPI.h"

struct SpiDescriptor
{
    uint32_t spiAddress;
    LnIRQ spiIrq;
    int dmaEngine;
    int dmaTxChannel;
    Peripherals rcu;
    lnPin mosi, miso, clk;
};

// We assume all the pins of a given SPI are on the same gpio bank
static const SpiDescriptor spiDescriptor[3] = {
    //                       DMA DMAT periph MOSI MISO CLK
    {LN_SPI0_ADR, LN_IRQ_SPI0, 0, 2, pSPI0, PA7, PA6, PA5},
    {LN_SPI1_ADR, LN_IRQ_SPI1, 0, 4, pSPI1, PB15, PB14, PB13},
    {LN_SPI2_ADR, LN_IRQ_SPI2, 1, 1, pSPI2, PB5, PB4, PB3}};
LN_SPI_Registers *aspi0 = (LN_SPI_Registers *)LN_SPI0_ADR;
LN_SPI_Registers *aspi1 = (LN_SPI_Registers *)LN_SPI1_ADR;
LN_SPI_Registers *aspi2 = (LN_SPI_Registers *)LN_SPI2_ADR;

/**
 * @brief
 *
 * @param instance
 * @param pinCs
 * @return lnSPI*
 */

lnSPI *lnSPI::create(uint32_t instance, int pinCs)
{
    return new lnSPI_bp(instance, pinCs);
}

/**
 * switch between RX/TX and TX only : false is txRx,
 * @param adr
 * @param rxTx
 */

void updateMode(LN_SPI_Registers *d, lnSpiDirection dir)
{

    uint32_t reg = d->CTL0;

    reg &= ~(LN_SPI_CTL0_BDOEN | LN_SPI_CTL0_BDEN | LN_SPI_CTL0_RO);
    switch (dir)
    {
    case lnDuplex:
        reg |= 0;
        break; // 0 all bits = full duplex
    case lnTxOnly:
        reg |= LN_SPI_CTL0_BDOEN | LN_SPI_CTL0_BDEN;
        break;
    case lnRxOnly:
        reg |= LN_SPI_CTL0_BDEN;
        break;
    default:
        xAssert(0);
        break;
    }
    d->CTL0 = reg;
}
/**
 *
 * @param adr
 * @param bits
 */
void updateDataSize(LN_SPI_Registers *d, int bits)
{

    uint32_t reg = d->CTL0;
    switch (bits)
    {
    case 8:
        reg &= ~LN_SPI_CTL0_FF16;
        break;
    case 16:
        reg |= LN_SPI_CTL0_FF16;
        break;
    default:
        xAssert(0);
    }
    d->CTL0 = reg;
}
void updateDmaTX(LN_SPI_Registers *d, bool onoff)
{
    if (onoff)
        d->CTL1 |= (uint32_t)(LN_SPI_CTL1_DMATEN);
    else
        d->CTL1 &= ~((uint32_t)(LN_SPI_CTL1_DMATEN));
}

/**
 *
 * @param instance
 * @param pinCs
 */
#define M(x) spiDescriptor[instance].x

lnSPI_bp::lnSPI_bp(int instance, int pinCs)
    : lnSPI(instance, pinCs), txDma(lnDMA::DMA_MEMORY_TO_PERIPH, M(dmaEngine), M(dmaTxChannel), 16, 16)
{
    _instance = instance;

    lnPeripherals::enable((Peripherals)(pSPI0 + instance));
    xAssert(instance < 3);
    const SpiDescriptor *s = spiDescriptor + instance;

    _irq = s->spiIrq;
    lnEnableInterrupt(_irq);
    _regs = (LN_SPI_Registers *)s->spiAddress;
    lnPeripherals::enable(s->rcu);
    // setup miso/mosi & clk
    lnPinMode(s->mosi, lnALTERNATE_PP);
    lnPinMode(s->miso, lnFLOATING);
    lnPinMode(s->clk, lnALTERNATE_PP);

    if (pinCs != -1)
    {
        lnPinMode((lnPin)pinCs, lnOUTPUT);
        lnDigitalWrite((lnPin)pinCs, true);
    }
    sdisable();
}
/**
 *
 */
lnSPI_bp::~lnSPI_bp()
{
    sdisable();
}
/**
 * @brief
 *
 */

void lnSPI_bp::begin(uint32_t bitSize)
{
    setup();
    updateMode(_regs, lnTxOnly);
    updateDataSize(_regs, bitSize);
    senable();
    csOn();
}

/**
 */
void lnSPI_bp::end()
{
    csOff();
    sdisable();
}

/**
 *
 * @param sz
 * @param data
 * @return
 */
bool lnSPI_bp::writeInternal(int sz, int data)
{
    if (_regs->STAT & LN_SPI_STAT_CONFERR)
    {
        Logger("Conf Error\n");
        return false;
    }

    while (sbusy())
    {
    }
    _regs->DATA = data;
    waitForCompletion();
    return true;
}

/**
 * @brief
 *
 * @param sz
 * @param nb
 * @param data
 * @param repeat
 * @return true
 * @return false
 */
bool lnSPI_bp::writesInternal(int sz, uint32_t nb, const uint8_t *data, bool repeat)
{
    switch (sz)
    {
    default:
        xAssert(0);
        break;
    case 8: {
        const uint8_t *p = data;
        int inc = !repeat;
        for (size_t i = 0; i < nb; i++)
        {
            while (sbusy())
            {
            }
            _regs->DATA = *p;
            p += inc;
        }
        break;
    }
    case 16: {
        auto *p = (const uint16_t *)data;
        int inc = !repeat;
        for (size_t i = 0; i < nb; i++)
        {
            while (sbusy())
            {
            }
            _regs->DATA = *p;
            p += inc;
        }
        break;
    }
    }

    waitForCompletion();
    return true;
}
/**
 * @brief
 *
 * @param nbWord
 * @param data
 * @return true
 * @return false
 */
bool lnSPI_bp::write16(uint32_t nbWord, const uint16_t *data)
{
    for (int i = 0; i < nbWord; i++)
        write16(data[i]);
    return true;
}
/**
 * @brief
 *
 * @param nbBytes
 * @param data
 * @return true
 * @return false
 */
bool lnSPI_bp::write8(uint32_t nbBytes, const uint8_t *data)
{
    for (int i = 0; i < nbBytes; i++)
        write8(data[i]);
    return true;
}

/**
 *
 * @param z
 * @return
 */
bool lnSPI_bp::write8(uint8_t z)
{
    return writeInternal(8, z);
}
/**
 *
 * @param z
 * @return
 */
bool lnSPI_bp::write16(uint16_t z)
{
    return writeInternal(16, z);
}

/**
 * @brief
 *
 */
void lnSPI_bp::csOn()
{
    if (_internalSettings.pinCS != -1)
    {
        lnDigitalWrite((lnPin)_internalSettings.pinCS, false);
    }
}
/**
 * @brief
 *
 */
void lnSPI_bp::csOff()
{
    if (_internalSettings.pinCS != -1)
    {
        lnDigitalWrite((lnPin)_internalSettings.pinCS, true);
    }
}
/**
 *  This is needed to be able to toggle the CS when all is done
 */
void lnSPI_bp::waitForCompletion() const
{
    int dir = _regs->CTL0 >> 14;
    switch (dir)
    {
    case 0:
    case 1: // bidir mode
        while (sbusy())
        {
        }
        break;
    case 2: // receive only
        break;
    case 3: //  tx only

        while (txbusy())
        {
            __asm__("nop");
        }

        while (sbusy())
        {
            __asm__("nop");
        }
        break;
    }
}
/**
 * @brief
 *
 * @param nbRead
 * @param rd
 * @return true
 * @return false
 */
bool lnSPI_bp::read1wire(uint32_t nbRead, uint8_t *rd)
{
    updateMode(_regs, lnRxOnly); // 1 Wire RX

    // clear stuff (not sure it is useful)
    for (int i = 0; i < nbRead; i++)
    {
        while (!(_regs->STAT & LN_SPI_STAT_RBNE))
        {
            __asm__("nop");
        }
        rd[i] = _regs->DATA;
    }
    waitForCompletion();
    return true;
}

/**
 * @brief
 *
 * @param nbBytes
 * @param dataOut
 * @param dataIn
 * @return true
 * @return false
 */
bool lnSPI_bp::transfer(uint32_t nbBytes, uint8_t *dataOut, uint8_t *dataIn)
{
    for (size_t i = 0; i < nbBytes; i++)
    {
        while (!(_regs->STAT & LN_SPI_STAT_TBE))
        {
        }
        _regs->DATA = dataOut[i];

        while (!(_regs->STAT & LN_SPI_STAT_RBNE))
        {
        }
        dataIn[i] = _regs->DATA;
    }
    waitForCompletion();
    return true;
}
/**
 * @brief
 *
 */
void lnSPI_bp::txDone()
{
    _done.give();
}
/**
 * \brief program the peripheral with the current settings
 */
void lnSPI_bp::setup()
{
    sdisable();
    _regs->STAT &= LN_SPI_STAT_MASK;
    _regs->CTL0 &= LN_SPI_CTL0_MASK;
    _regs->CTL1 &= LN_SPI_CTL1_MASK;

    uint32_t ctl0 = _regs->CTL0;
    ctl0 |= LN_SPI_CTL0_MSTMODE;
    // Drive the NSS by sw, pull it up
    // there does not seem to be a way to completely disconnect NSS management
    ctl0 |= LN_SPI_CTL0_SWNSS;
    ctl0 |= LN_SPI_CTL0_SWNSSEN;

    switch (_internalSettings.bOrder)
    {
    case SPI_LSBFIRST:
        ctl0 |= LN_SPI_CTL0_LSB;
        break;
    case SPI_MSBFIRST:
        ctl0 &= ~LN_SPI_CTL0_LSB;
        break;
    default:
        xAssert(0);
        break;
    }
    uint32_t set = 0;
    switch (_internalSettings.dMode)
    {
    // https://en.wikipedia.org/wiki/Serial_Peripheral_Interface
    case SPI_MODE0: // Pol0, Phase 0/Edge 1
        set = 0;    // Low 1 edge
        break;
    case SPI_MODE1:             // Pol 0 Phase 1 Edge 0
        set = LN_SPI_CTL0_CKPH; // Low 2 edge
        break;
    case SPI_MODE2:             // POL 1 PHA 0 Edge 1
        set = LN_SPI_CTL0_CKPL; // high , 1 edge
        break;
    case SPI_MODE3:                                // POL 1 PHA 1 Edge 0
        set = LN_SPI_CTL0_CKPL | LN_SPI_CTL0_CKPH; // high , 2 edge
        break;
    default:
        xAssert(0);
        break;
    }
    ctl0 |= set;
    _regs->CTL0 = ctl0;
    uint32_t prescale = 0;
    uint32_t speed = _internalSettings.speed;
    xAssert(speed);

    auto periph = (Peripherals)((int)pSPI0 + _instance);
    uint32_t apb = lnPeripherals::getClock(periph);
    prescale = (apb + speed / 2) / speed;
    // prescale can only go from 2 to 256, and prescale=2^(psc+1) actually

    uint32_t psc = 0;
    if (prescale <= 2)
        psc = 0;
    else if (prescale >= 256)
        psc = 7;
    else
    {
        psc = 0;
        uint32_t toggle = 2;
        while (prescale > toggle)
        {
            toggle *= 2;
            psc++;
        }
    }
    uint32_t sp = _regs->CTL0;
    sp &= ~(7 << 3);
    sp |= psc << 3;
    _regs->CTL0 = sp;
    updateMode(_regs, lnTxOnly); // Tx only by default
}

/**
 * @brief
 *
 * @param order
 */
void lnSPI_bp::setBitOrder(spiBitOrder order)
{
    _internalSettings.bOrder = order;
    setup();
}
/**
 * @brief
 *
 * @param mode
 */
void lnSPI_bp::setDataMode(spiDataMode mode)
{
    _internalSettings.dMode = mode;
    setup();
}
/**
 * @brief
 *
 * @param speed
 */
void lnSPI_bp::setSpeed(uint32_t speed)
{
    _internalSettings.speed = speed;
    setup();
}
// EOF