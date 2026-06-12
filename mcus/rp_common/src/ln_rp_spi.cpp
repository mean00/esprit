/**
 * @file ln_rp_spi.cpp
 * @author mean00
 * @brief
 * @version 0.1
 * @date 2023-12-26
 *
 * @copyright Copyright (c) 2023
 *
 */
// clang-format off
#include "ln_rp.h" 
#include "ln_rp_spi.h"
#include "esprit.h"
#include "lnGPIO.h"
#include "lnSPI.h"
#include "lnSPIBidir.h"
#include "ln_rp_clocks.h"
#include "ln_rp_dma.h"
#include "ln_rp_memory_map.h"
#include "ln_rp_spi_priv.h"
// clang-format on

// #define NO_DMA 1

class rpSPI;
#define RP_SPI_USE_DMA 1
typedef struct
{
    LN_RP_SPI *spi;
    LnIRQ irq;
    lnRpDMA::LN_RP_DMA_DREQ txDma;
    lnRpDMA::LN_RP_DMA_DREQ rxDma;
} SPI_DESC;

const SPI_DESC spis[2] = {
    {(LN_RP_SPI *)LN_RP_SPI0, (LnIRQ)18, lnRpDMA::LN_DMA_DREQ_SPI0_TX, lnRpDMA::LN_DMA_DREQ_SPI0_RX},
    {(LN_RP_SPI *)LN_RP_SPI1, (LnIRQ)19, lnRpDMA::LN_DMA_DREQ_SPI1_TX, lnRpDMA::LN_DMA_DREQ_SPI1_RX},
};

typedef void lnSpiCallback(void *cookie);
static rpSPI *instances[2] = {NULL, NULL};

static void spiHandler(int x)
{
    rpSPI *s = instances[x];
    xAssert(s);
    s->irqHandler();
}
static void dmaCb(void *cookie)
{
    rpSPI *spi = (rpSPI *)cookie;
    spi->dmaHandler();
}
static void dmaRxCb(void *cookie)
{
    rpSPI *spi = (rpSPI *)cookie;
    spi->dmaRxHandler();
}
static void spiHandler0()
{
    spiHandler(0);
}
static void spiHandler1()
{
    spiHandler(1);
}

/**
 * @brief Construct a new rp S P I::rp S P I object
 *
 * @param instance
 * @param pinCs
 */
rpSPI::rpSPI(uint32_t instance, int pinCs) : lnSPIBidir(instance, pinCs)
{
    xAssert(!instances[instance]);
    instances[instance] = this;
    _cs = (lnPin)pinCs;
    _instance = instance;
    _spi = (LN_RP_SPI *)spis[instance].spi;
    _cr0 = LN_RP_SPI_CR0_DIVIDER(25) + LN_RP_SPI_CR0_MODE(0) + LN_RP_SPI_CR0_FORMAT_MOTOROLA;
    _cr1 = 0;
    _prescaler = 250;
    _wordSize = 8;
    _dummyFF = 0xFFFFFFFF;

    _txDma = new lnRpDMA(lnRpDMA::DMA_MEMORY_TO_PERIPH, spis[_instance].txDma, _wordSize);
    _txDma->attachCallback(dmaCb, this);
    _rxDma = new lnRpDMA(lnRpDMA::DMA_PERIPH_TO_MEMORY, spis[_instance].rxDma, _wordSize);
    _rxDma->attachCallback(dmaRxCb, this);
    if (!_instance)
        lnSetInterruptHandler(spis[_instance].irq, spiHandler0);
    else
        lnSetInterruptHandler(spis[_instance].irq, spiHandler1);
    _txDma->doMemoryToPeripheralTransferNoLock(1, (const uint32_t *)NULL, (const uint32_t *)&(_spi->DR));
}
/**
 * @brief Destroy the rp S P I::rp S P I object
 *
 */
rpSPI::~rpSPI()
{
    end();
    delete _txDma;
    _txDma = NULL;
    delete _rxDma;
    _rxDma = NULL;
    instances[_instance] = NULL;
}
/**
 * @brief
 *
 * @param wordsize
 */
void rpSPI::begin(uint32_t wordsize)
{
    _wordSize = wordsize;
    xAssert(_wordSize == 8 || _wordSize == 16);
    _cr0 &= ~LN_RP_SPI_CR0_BITS_MASK;
    switch (_wordSize)
    {
    case 8:
        _cr0 |= LN_RP_SPI_CR0_8_BITS;
        _dummyFF = 0xFF;
        break;
    case 16:
        _cr0 |= LN_RP_SPI_CR0_16_BITS;
        _dummyFF = 0xFFFF;
        break;
    default:
        xAssert(0);
        break;
    }

    _spi->CR0 = _cr0;
    _spi->CR1 = _cr1;
    _spi->CPSR = _prescaler;
    _spi->IMSC = 0;
    _spi->DMACR = 0; /*LN_RP_SPI_DMACR_RX |  LN_RP_SPI_DMACR_TX */
    ;                // enable DMA, activating the correct DMA *or* IRQ will select DMA or not

    _spi->CR1 |= LN_RP_SPI_CR1_ENABLE;
}
/**
 * @brief
 *
 */
void rpSPI::end(void)
{
    _spi->CR1 &= ~LN_RP_SPI_CR1_ENABLE;
    _spi->DMACR = 0;
    _spi->IMSC = 0;
}
/**
 * @brief
 *
 * @param speed
 */
void rpSPI::setSpeed(uint32_t speed)
{

    uint32_t fq_in = clock_get_hz(clk_peri);

#define RATIO 128

    int div = (fq_in) / (speed);
    int org = (div / RATIO);
    _prescaler = org & ~1; // must be even, starting at 2, see 4.4.4
    if (!_prescaler)
        _prescaler = 2;

    // the actual clock is SPI_CLOCK/(prescaler*scaler)
    // do scaler = SPI_CLOCK/(wanted_clock*presaler)
    // which is the same as sclae = (spi_CLOK/wanter) / prescaler
    // div/prescaler with round up / down

    int scaler = ((div + (_prescaler >> 1)) / _prescaler) - 1;
    if (scaler > 255)
        scaler = 255;
    _cr0 &= ~LN_RP_SPI_CR0_DIVIDER_MASK;
    _cr0 |= LN_RP_SPI_CR0_DIVIDER(scaler);

    _spi->CR0 = _cr0;
}

/**
 * @brief
 *
 * @param order
 */
void rpSPI::setBitOrder(spiBitOrder order)
{
    switch (order)
    {
    case SPI_LSBFIRST:
        xAssert(0);
        break;
    case SPI_MSBFIRST:
        break;
    default:
        xAssert(0);
        break;
    }
}

/**
 * @brief
 *
 * @param dataSize
 */
void rpSPI::setDataSize(uint32_t dataSize)
{
    _cr0 &= ~LN_RP_SPI_CR0_BITS_MASK;
    switch (dataSize)
    {
    case 8:
        _cr0 |= LN_RP_SPI_CR0_8_BITS;
        _dummyFF = 0xFF;
        break;
    case 16:
        _cr0 |= LN_RP_SPI_CR0_16_BITS;
        _dummyFF = 0xFFFF;
        break;
    default:
        xAssert(0);
        break;
    }
    _spi->CR0 = _cr0;
}

/**
 * @brief
 *
 * @param mode
 */
void rpSPI::setDataMode(spiDataMode mode)
{
    _cr0 &= ~LN_RP_SPI_CR0_MODE(3);
    uint32_t r = 0;
    switch (mode)
    {
    case SPI_MODE0:
        r = 0;
        break;
    case SPI_MODE1:
        r = 2;
        break;
    case SPI_MODE2:
        r = 1;
        break;
    case SPI_MODE3:
        r = 3;
        break;
    default:
        xAssert(0);
        break;
    }
    _cr0 |= LN_RP_SPI_CR0_MODE(r);
    _spi->CR0 = _cr0;
}
/**
 * @brief
 *
 */
void rpSPI::waitForCompletion() const
{
    int countdown = 10 * 1000 * 1000; // 100M/90k =>
    while ((_spi->SR & LN_RP_SPI_SR_BSY))
    {
        __asm__("nop");
        if (--countdown == 0)
        {
            Logger("SPI Timeout !\n");
            return;
        }
    }
}

bool rpSPI::blockWrite_all(uint32_t wordSize, uint32_t nbExchange, const uint32_t *data, bool repeat)
{
    xAssert(_wordSize == wordSize);
    _callback = nullptr;
    _spi->DMACR = LN_RP_SPI_DMACR_RX | LN_RP_SPI_DMACR_TX;
    _spi->IMSC |= LN_RP_SPI_INT_TX;
    _txDone.tryTake();
    _txDma->setTransferSize(wordSize);
    _txDma->doMemoryToPeripheralTransferNoLock(nbExchange, data, (const uint32_t *)&(_spi->DR), repeat);
    _txDma->beginTransfer();
    _txDone.take();
    waitForCompletion();
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
bool rpSPI::blockWrite8(uint32_t nbBytes, const uint8_t *data)
{
    return blockWrite_all(8, nbBytes, (const uint32_t *)data, false);
}
/**
 * @brief
 *
 * @param nbHalfWord
 * @param data
 * @return true
 * @return false
 */
bool rpSPI::blockWrite16(uint32_t nbHalfWord, const uint16_t *data)
{
    return blockWrite_all(16, nbHalfWord, (const uint32_t *)data, false);
}
/**
 * @brief
 *
 * @param nbHalfWord
 * @param data
 * @param repeat
 * @return true
 * @return false
 */
bool rpSPI::blockWrite16Repeat(uint32_t nbHalfWord, const uint16_t data)
{
    return blockWrite_all(16, nbHalfWord, (const uint32_t *)&data, true);
}

/**
 * @brief
 *
 * @param nbBytes
 * @param data
 * @return true
 * @return false
 */
bool rpSPI::blockWrite8Repeat(uint32_t nbBytes, const uint8_t data)
{
    return blockWrite_all(8, nbBytes, (const uint32_t *)&data, true);
}
/**
 * @brief
 *
 */
void rpSPI::irqHandler()
{
    xAssert(0);
#if 0
    switch (_state)
    {
    case TxStateBody: {
        while (_current < _limit)
        {
            if (_spi->SR & LN_RP_SPI_SR_TFN)
            {
                _spi->DR = *_current;
                _current++;
            }
            else
            {
                break;
            }
        }
        if (_current == _limit)
        {
            _state = TxStateLast;
            _spi->IMSC = 0;
            lnDisableInterrupt(spis[_instance].irq);
            _txDone.give();
        }
    }
    break;
    default:
        xAssert(0);
        break;
    }
#endif
}
/**
 * @brief
 *
 */
void rpSPI::dmaHandler()
{
    _spi->IMSC = 0;
    if (_callback)
    {
        _callback(_callbackCookie);
    }
    else
    {
        _txDma->endTransfer();
        _txDone.give();
    }
}

/**
 * @brief RX DMA completion handler
 *
 */
void rpSPI::dmaRxHandler()
{
    if (_callback)
    {
        _callback(_callbackCookie);
    }
    else
    {
        _rxDma->endTransfer();
        _rxDone.give();
    }
}

/**
 * @brief
 *
 * @param settings
 */
void rpSPI::set(lnSPISettings &settings)
{
    setBitOrder(settings.bOrder);
    setDataMode(settings.dMode);
    setSpeed((int)settings.speed);
}
/**
 * @brief
 *
 * @param a
 * @param b
 * @return lnSPI*
 */
lnSPI *lnSPI::create(uint32_t instance, int pinCs)
{
    return new rpSPI(instance, pinCs);
}

/**
 * @brief
 *
 * @param instance
 * @param pinCs
 * @return lnSPIBidir*
 */
lnSPIBidir *lnSPIBidir::createBiDir(uint32_t instance, int pinCs)
{
    return new rpSPI(instance, pinCs);
}

//--- not implemented (old non-const transfer) ---
bool rpSPI::transfer(uint32_t nbBytes, uint8_t *dataOut, uint8_t *dataIn)
{
    return transfer(nbBytes, (const uint8_t *)dataOut, dataIn);
}

//=============================================================================
// lnSPIBidir interface implementation
//=============================================================================

/**
 * @brief Polling fallback for small transfers
 *
 */
bool rpSPI::transferPolling(uint32_t nbBytes, const uint8_t *dataOut, uint8_t *dataIn)
{
    for (uint32_t i = 0; i < nbBytes; i++)
    {
        _spi->DR = dataOut[i];
        while (!(_spi->SR & LN_RP_SPI_SR_RNE))
        {
            __asm__("nop");
        }
        dataIn[i] = (uint8_t)_spi->DR;
    }
    waitForCompletion();
    return true;
}

/**
 * @brief Polling fallback for small reads
 *
 */
bool rpSPI::readPolling(uint32_t nbBytes, uint8_t *dataIn)
{
    for (uint32_t i = 0; i < nbBytes; i++)
    {
        _spi->DR = _dummyFF;
        while (!(_spi->SR & LN_RP_SPI_SR_RNE))
        {
            __asm__("nop");
        }
        dataIn[i] = (uint8_t)_spi->DR;
    }
    waitForCompletion();
    return true;
}

/**
 * @brief Blocking full-duplex transfer
 *
 * Uses DMA for transfers >= RP_SPI_MIN_DMA, polling fallback for smaller ones.
 *
 * @param nbBytes Number of bytes to exchange
 * @param dataOut Data to send (TX)
 * @param dataIn  Buffer for received data (RX)
 * @return true on success
 */
bool rpSPI::transfer(uint32_t nbBytes, const uint8_t *dataOut, uint8_t *dataIn)
{
    if (nbBytes < RP_SPI_MIN_DMA)
    {
        return transferPolling(nbBytes, dataOut, dataIn);
    }

    _callback = nullptr;
    _txDone.tryTake();
    _rxDone.tryTake();

    _spi->DMACR = LN_RP_SPI_DMACR_RX | LN_RP_SPI_DMACR_TX;
    _spi->IMSC |= LN_RP_SPI_INT_TX;

    _txDma->setTransferSize(_wordSize);
    _rxDma->setTransferSize(_wordSize);

    _txDma->doMemoryToPeripheralTransferNoLock(nbBytes, (const uint32_t *)dataOut, (const uint32_t *)&(_spi->DR),
                                               false);
    _rxDma->doPeripheralToMemoryTransferNoLock(nbBytes, (const uint32_t *)&(_spi->DR), (uint32_t *)dataIn, false);

    _txDma->beginTransfer();
    _rxDma->beginTransfer();

    _txDone.take();
    _rxDone.take();
    waitForCompletion();
    return true;
}

/**
 * @brief Blocking receive-only (sends 0xFF/0xFFFF dummy bytes on MOSI)
 *
 * Uses DMA for transfers >= RP_SPI_MIN_DMA, polling fallback for smaller ones.
 *
 * @param nbBytes Number of bytes to receive
 * @param dataIn  Buffer for received data
 * @return true on success
 */
bool rpSPI::read(uint32_t nbBytes, uint8_t *dataIn)
{
    if (nbBytes < RP_SPI_MIN_DMA)
    {
        //_txDone.take();
        bool r = readPolling(nbBytes, dataIn);
        //_txDone.give();
        return r;
    }

    _callback = nullptr;
    _txDone.tryTake();
    _rxDone.tryTake();

    _spi->DMACR = LN_RP_SPI_DMACR_RX | LN_RP_SPI_DMACR_TX;
    _spi->IMSC |= LN_RP_SPI_INT_TX;

    _txDma->setTransferSize(_wordSize);
    _rxDma->setTransferSize(_wordSize);

    // TX: repeat-send dummy byte to generate clocks
    _txDma->doMemoryToPeripheralTransferNoLock(nbBytes, (const uint32_t *)&_dummyFF, (const uint32_t *)&(_spi->DR),
                                               true);
    // RX: capture incoming data
    _rxDma->doPeripheralToMemoryTransferNoLock(nbBytes, (const uint32_t *)&(_spi->DR), (uint32_t *)dataIn, false);

    _txDma->beginTransfer();
    _rxDma->beginTransfer();

    _txDone.take();
    _rxDone.take();
    waitForCompletion();
    return true;
}

/**
 * @brief Async full-duplex transfer
 *
 * @param nbBytes Number of bytes to exchange
 * @param dataOut Data to send (TX)
 * @param dataIn  Buffer for received data (RX)
 * @param cb      Callback on completion
 * @param cookie  User cookie for callback
 * @return true if started successfully
 */
bool rpSPI::asyncTransfer(uint32_t nbBytes, const uint8_t *dataOut, uint8_t *dataIn, lnSpiCallback *cb, void *cookie)
{
    this->_callback = cb;
    this->_callbackCookie = cookie;

    _txDone.tryTake();
    _rxDone.tryTake();

    _spi->DMACR = LN_RP_SPI_DMACR_RX | LN_RP_SPI_DMACR_TX;
    _spi->IMSC |= LN_RP_SPI_INT_TX;

    _txDma->setTransferSize(_wordSize);
    _rxDma->setTransferSize(_wordSize);

    _txDma->attachCallback(dmaCb, this);
    _rxDma->attachCallback(dmaRxCb, this);

    _txDma->doMemoryToPeripheralTransferNoLock(nbBytes, (const uint32_t *)dataOut, (const uint32_t *)&(_spi->DR),
                                               false);
    _rxDma->doPeripheralToMemoryTransferNoLock(nbBytes, (const uint32_t *)&(_spi->DR), (uint32_t *)dataIn, false);

    _txDma->beginTransfer();
    _rxDma->beginTransfer();
    return true;
}

/**
 * @brief Async receive-only (sends 0xFF/0xFFFF dummy bytes on MOSI)
 *
 * @param nbBytes Number of bytes to receive
 * @param dataIn  Buffer for received data
 * @param cb      Callback on completion
 * @param cookie  User cookie for callback
 * @return true if started successfully
 */
bool rpSPI::asyncRead(uint32_t nbBytes, uint8_t *dataIn, lnSpiCallback *cb, void *cookie)
{
    this->_callback = cb;
    this->_callbackCookie = cookie;

    _txDone.tryTake();
    _rxDone.tryTake();

    _spi->DMACR = LN_RP_SPI_DMACR_RX | LN_RP_SPI_DMACR_TX;
    _spi->IMSC |= LN_RP_SPI_INT_TX;

    _txDma->setTransferSize(_wordSize);
    _rxDma->setTransferSize(_wordSize);

    _txDma->attachCallback(dmaCb, this);
    _rxDma->attachCallback(dmaRxCb, this);

    // TX: repeat-send dummy byte to generate clocks
    _txDma->doMemoryToPeripheralTransferNoLock(nbBytes, (const uint32_t *)&_dummyFF, (const uint32_t *)&(_spi->DR),
                                               true);
    // RX: capture incoming data
    _rxDma->doPeripheralToMemoryTransferNoLock(nbBytes, (const uint32_t *)&(_spi->DR), (uint32_t *)dataIn, false);

    _txDma->beginTransfer();
    _rxDma->beginTransfer();
    return true;
}

/**
 * @brief Clean up both TX and RX DMAs after async bidir operation
 *
 * @return true on success
 */
bool rpSPI::finishAsyncDmaBidir()
{
    _txDma->endTransfer();
    _rxDma->endTransfer();
    _txDone.give();
    _rxDone.give();
    return true;
}

/**
 * @brief
 *
 * @return true
 * @return false
 */
bool rpSPI::finishAsyncDma()
{
    _txDma->endTransfer();
    _txDone.give();
    return true;
}
/**
 * @brief
 *
 * @param nbBytes
 * @param data
 * @param cb
 * @param cookie
 * @param repeat
 * @return true
 * @return false
 */
bool rpSPI::asyncWrite8(uint32_t nbBytes, const uint8_t *data, lnSpiCallback *cb, void *cookie, bool repeat)
{
    this->_callback = cb;
    this->_callbackCookie = cookie;
    // source targer
    _spi->DMACR = LN_RP_SPI_DMACR_TX;
    _spi->IMSC |= LN_RP_SPI_INT_TX;
    _txDone.tryTake();
    _txDma->attachCallback(dmaCb, this);
    _txDma->doMemoryToPeripheralTransferNoLock(nbBytes, (const uint32_t *)data, (const uint32_t *)&(_spi->DR), repeat);
    _txDma->beginTransfer();
    return true;
}
/**
 * @brief
 *
 * @param nbBytes
 * @param data
 * @param cb
 * @param cookie
 * @param repeat
 * @return true
 * @return false
 */
bool rpSPI::nextWrite8(uint32_t nbBytes, const uint8_t *data, lnSpiCallback *cb, void *cookie, bool repeat)
{
    this->_callback = cb;
    this->_callbackCookie = cookie;
    _txDma->attachCallback(dmaCb, this);
    return _txDma->continueMemoryToPeripheralTransferNoLock(nbBytes, (const uint32_t *)data);
}
//--- not implemented ---

//--- not implemented ---
//--- not implemented ---

/**
 * @brief
 *
 * @param z
 * @return true
 * @return false
 */
bool rpSPI::write8(const uint8_t data)
{
    return write16(data);
}
/**
 * @brief
 *
 * @param data
 * @return true
 * @return false
 */
bool rpSPI::write16(const uint16_t data)
{
    int countdown = 1000 * 1000 * 1000; // 100M/90k =>
    while (!(_spi->SR & LN_RP_SPI_SR_TFE))
    {
        __asm__("nop");
        if (--countdown == 0)
        {
            // xAssert(0);
            return false;
        }
    }
    _spi->DR = data;
    waitForCompletion();
    return true;
}

// --- not implemented ---
bool rpSPI::read1wire(uint32_t nbRead, uint8_t *rd)
{
    return false;
} // read, reuse MOSI

// -- not implemented --
bool rpSPI::waitForAsync()
{
    bool r = _txDone.take(60 * 1000);
    waitForCompletion();
    //--
    return r;
}

/**
 * @brief
 *
 * @param nbWord
 * @param data
 * @param cb
 * @param cookie
 * @param repeat
 * @return true
 * @return false
 */
bool rpSPI::asyncWrite16(uint32_t nbWord, const uint16_t *data, lnSpiCallback *cb, void *cookie, bool repeat)
{
    this->_callback = cb;
    this->_callbackCookie = cookie;
    // source target
    _spi->DMACR = LN_RP_SPI_DMACR_TX;
    _spi->IMSC |= LN_RP_SPI_INT_TX;
    _txDone.tryTake();
    _txDma->setTransferSize(16);
    _txDma->attachCallback(dmaCb, this);
    _txDma->doMemoryToPeripheralTransferNoLock(nbWord, (const uint32_t *)data, (const uint32_t *)&(_spi->DR), repeat);
    _txDma->beginTransfer();
    return true;
}
/**
 * @brief
 *
 * @param nbBytes
 * @param data
 * @param cb
 * @param cookie
 * @param repeat
 * @return true
 * @return false
 */
bool rpSPI::nextWrite16(uint32_t nbWord, const uint16_t *data, lnSpiCallback *cb, void *cookie, bool repeat)
{
    this->_callback = cb;
    this->_callbackCookie = cookie;
    return _txDma->continueMemoryToPeripheralTransferNoLock(nbWord, (const uint32_t *)data);
}

//--