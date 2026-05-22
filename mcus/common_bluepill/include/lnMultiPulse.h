/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 *
 *  DMA-based multi-pulse generator.
 */
#pragma once
#include "lnDma.h"
#include "lnTimer.h"

/**
 * DMA-based multi-pulse generator.
 *
 * Uses a single timer channel in toggle mode with DMA refilling CHCV
 * on each compare event. The output sequence is:
 *
 *     ┌─────┐         ┌─────┐
 *     │     │         │     │
 *  ───┘     └─────────┘     └──────────
 *     ↑pulse1↑  gap  ↑pulse2↑
 *
 * The entire sequence runs autonomously once fire() is called.
 * No CPU intervention is needed after the initial setup.
 */
class lnMultiPulse : public lnTimer
{
  public:
    /**
     * Create a multi-pulse generator on the given pin.
     *
     * @param pin       The output pin (must have a timer mapping).
     * @param tickFqHz  Timer tick frequency in Hz (e.g. 100000 = 10 us ticks).
     *                  Higher values give better timing resolution but limit
     *                  the maximum total pulse duration (max ~65536 ticks).
     */
    lnMultiPulse(lnPin pin, int tickFqHz);
    virtual ~lnMultiPulse();

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
    void fire(int pulse1Ms, int gapMs, int pulse2Ms);

    /**
     * Block until the pulse sequence completes.
     */
    void waitDone();

    /**
     * Abort a running pulse sequence.
     */
    void stop();

  protected:
    lnDMA *_dma;
    uint16_t _buffer[3];
    int _tickFqHz;
    bool _running;
};