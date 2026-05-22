/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 *
 *  DMA-based multi-pulse generator.
 *
 *  Uses a single timer channel in toggle mode with DMA refilling CHCV
 *  on each compare event. The DMA buffer contains cumulative tick counts:
 *
 *      buffer[0] = pulse1_ticks
 *      buffer[1] = pulse1_ticks + gap_ticks
 *      buffer[2] = pulse1_ticks + gap_ticks + pulse2_ticks
 *
 *  Timeline (pure hardware, zero CPU after fire()):
 *      t=0:           CEN=1, output=high, CHCV=buffer[0]
 *      t=pulse1:      CNT=CHCV → output toggles low
 *                     DMA writes CHCV = buffer[1]
 *      t=pulse1+gap:  CNT=CHCV → output toggles high
 *                     DMA writes CHCV = buffer[2]
 *      t=total:       CNT=CHCV → output toggles low (end)
 *                     DMA completes, timer stops (SPM)
 */
#include "lnMultiPulse.h"
#include "esprit.h"
#include "lnBarrier.h"
#include "lnDma.h"
#include "lnPinMapping.h"
#include "lnTimer.h"
#include "lnTimer_priv.h"
extern LN_Timers_Registers *abTimers[5];
#define aTimer(x) abTimers[x]

// Channel control register access macros (same as lnTimer.cpp)
#define READ_CHANNEL_CTL(channel) (((t->CHCTLs[channel >> 1]) >> (8 * (channel & 1))) & 0xFF)
#define WRITE_CHANNEL_CTL(channel, val)                                                                                \
    {                                                                                                                  \
        int shift = 8 * (channel & 1);                                                                                 \
        uint32_t r = t->CHCTLs[channel >> 1];                                                                          \
        r &= ~(0xff << shift);                                                                                         \
        r |= (val & 0xff) << shift;                                                                                    \
        t->CHCTLs[channel >> 1] = r;                                                                                   \
    }

/**
 * Look up DMA engine/channel for a given timer+channel.
 */
static bool searchDma(int timer, int channel, int &dmaEngine, int &dmaChannel)
{
    const LN_TIMER_MAPPING *m = timerMappings;
    int key = 10 * timer + channel;
    while (m->TimerChannel != -1)
    {
        if (m->TimerChannel == key)
        {
            dmaEngine = m->dmaEngine;
            dmaChannel = m->dmaChannel;
            return true;
        }
        m++;
    }
    xAssert(0);
    return false;
}

/**
 * DMA completion callback — signals the pulse sequence is done.
 */
static void _multiDmaCallback(void *cookie, lnDMA::DmaInterruptType typ)
{
    (void)cookie;
    (void)typ;
}

/**
 * Create a multi-pulse generator on the given pin.
 *
 * @param pin  The output pin (must have a timer mapping).
 * @param tickFqHz  Timer tick frequency in Hz (e.g. 100000 = 10 us ticks).
 */
lnMultiPulse::lnMultiPulse(lnPin pin, int tickFqHz)
    : lnTimer(pin)
{
    int dmaEngine = -1;
    int dmaChannel = -1;
    searchDma(_timer, _channel, dmaEngine, dmaChannel);

    // DMA: 16-bit memory → 16-bit peripheral, normal mode
    _dma = new lnDMA(lnDMA::DMA_MEMORY_TO_PERIPH, dmaEngine, dmaChannel, 16, 16);

    // Configure timer tick frequency
    LN_Timers_Registers *t = aTimer(_timer);
    t->CTL0 &= ~LN_TIMER_CTL0_CEN; // disable

    Peripherals per = pTIMER1;
    per = (Peripherals)((int)per + _timer - 1);
    uint32_t clock = lnPeripherals::getClock(per);
#if LN_ARCH == LN_ARCH_ARM
    clock *= 2; // APB1 prescaler compensation
#endif
    int divider = (clock + tickFqHz / 2) / tickFqHz;
    if (divider < 1)
        divider = 1;
    t->PSC = divider - 1;

    // Set CAR to max so the counter never wraps during our pulse sequence
    t->CAR = 0xFFFF;

    // Channel in toggle mode
    uint32_t chCtl = READ_CHANNEL_CTL(_channel);
    chCtl &= LN_TIME_CHCTL0_CTL_MASK;
    chCtl |= LN_TIME_CHCTL0_CTL_TOGGLE;
    chCtl &= LN_TIME_CHCTL0_MS_MASK;
    chCtl |= LN_TIME_CHCTL0_MS_OUPUT;
    WRITE_CHANNEL_CTL(_channel, chCtl);

    // Active high polarity
    t->CHCTL2 &= ~(LN_TIMER_CHTL2_CHxP(_channel));

    // Enable the channel (output connected to pin)
    t->CHCTL2 |= LN_TIMER_CHTL2_CHxEN(_channel);

    // Force output high initially (before CEN)
    chCtl = READ_CHANNEL_CTL(_channel);
    chCtl &= LN_TIME_CHCTL0_CTL_MASK;
    chCtl |= LN_TIME_CHCTL0_CTL_FORCE_HIGH;
    WRITE_CHANNEL_CTL(_channel, chCtl);

    // Switch back to toggle mode (pin stays high due to output latch)
    chCtl = READ_CHANNEL_CTL(_channel);
    chCtl &= LN_TIME_CHCTL0_CTL_MASK;
    chCtl |= LN_TIME_CHCTL0_CTL_TOGGLE;
    WRITE_CHANNEL_CTL(_channel, chCtl);

    // Enable DMA request on channel compare event
    t->DMAINTEN |= LN_TIMER_DMAINTEN_CHxDEN(_channel);

    // Enable DMA burst mode (DMAS) — DMA request on each compare event
    t->CTL1 |= LN_TIMER_CTL1_DMAS;

    _tickFqHz = tickFqHz;
    _running = false;
}

lnMultiPulse::~lnMultiPulse()
{
    stop();
    if (_dma)
    {
        delete _dma;
        _dma = NULL;
    }
}

/**
 * Fire a double pulse.
 *
 * @param pulse1Ms  Duration of the first pulse in ms.
 * @param gapMs     Gap between pulses in ms.
 * @param pulse2Ms  Duration of the second pulse in ms.
 *
 * Returns immediately. The hardware runs the sequence autonomously.
 * Call waitDone() to block until completion.
 */
void lnMultiPulse::fire(int pulse1Ms, int gapMs, int pulse2Ms)
{
    LN_Timers_Registers *t = aTimer(_timer);

    // Convert ms to ticks
    int ticksPerMs = _tickFqHz / 1000;

    uint16_t p1 = (uint16_t)(pulse1Ms * ticksPerMs);
    uint16_t p1g = (uint16_t)((pulse1Ms + gapMs) * ticksPerMs);
    uint16_t total = (uint16_t)((pulse1Ms + gapMs + pulse2Ms) * ticksPerMs);

    // Prepare DMA buffer
    _buffer[0] = p1;
    _buffer[1] = p1g;
    _buffer[2] = total;

    // Set CAR to just past the total duration so the counter wraps cleanly
    t->CAR = total + 1;

    // Set initial CHCV to pulse1 duration
    t->CHCVs[_channel] = p1;

    // Reset counter
    t->CNT = 0;

    // Start DMA transfer (3 half-words, normal mode, no circular)
    _dma->beginTransfer();
    _dma->attachCallback(_multiDmaCallback, this);
    _dma->doMemoryToPeripheralTransferNoLock(
        3,                          // count
        (uint16_t *)_buffer,        // source
        (const uint16_t *)&(t->CHCVs[_channel]), // target
        false,                      // repeat = false (increment source)
        false,                      // circular = false
        true                        // bothInterrupts = true
    );

    // Enable timer in single-pulse mode
    t->CTL0 |= LN_TIMER_CTL0_SPM;
    t->CTL0 |= LN_TIMER_CTL0_CEN;

    _running = true;
}

/**
 * Block until the pulse sequence completes.
 */
void lnMultiPulse::waitDone()
{
    if (!_running)
        return;

    LN_Timers_Registers *t = aTimer(_timer);

    // Wait for CEN to clear (SPM auto-clears it when CAR is reached)
    while ((t->CTL0 & LN_TIMER_CTL0_CEN) != 0)
    {
        __asm__("nop");
    }

    // Wait for DMA to finish
    while (_dma->getCurrentCount() != 0)
    {
        __asm__("nop");
    }

    _dma->endTransfer();
    _running = false;
}

/**
 * Abort a running pulse sequence.
 */
void lnMultiPulse::stop()
{
    if (!_running)
        return;

    LN_Timers_Registers *t = aTimer(_timer);
    t->CTL0 &= ~LN_TIMER_CTL0_CEN;
    t->DMAINTEN &= ~(LN_TIMER_DMAINTEN_CHxDEN(_channel));
    _dma->endTransfer();
    _running = false;

    // Force output low
    disable();
}