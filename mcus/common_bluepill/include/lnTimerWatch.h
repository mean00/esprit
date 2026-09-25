#pragma once

#include "lnPeripherals.h"
#include "lnTimer_priv.h"

extern LN_Timers_Registers *abTimers[];

/**
 * @brief An inline-heavy timer watcher based on general purpose timers.
 * Designed for highly precise, lock-free timer polling.
 */
class lnTimerWatch
{
  public:
    /**
     * @param timerIndex The general timer index (e.g., 4 for TIMER4 which maps to TIM5).
     */
    lnTimerWatch(int timerIndex) : _timerIndex(timerIndex)
    {
        _t = abTimers[_timerIndex];
    }

    /**
     * @brief Marks the start time for the next wait() calls.
     */
    inline LN_ALWAYS_INLINE void start()
    {
        _start_tick = _t->CNT;
    }

    /**
     * @brief Blocks until target_ticks have elapsed since the last start().
     * @param target_ticks Ticks to wait relative to start().
     */
    inline LN_ALWAYS_INLINE void wait(uint16_t target_ticks)
    {
        while ((uint16_t)(_t->CNT - _start_tick) < target_ticks)
        {
            asm("nop");
        }
    }

    /**
     * @brief Configure the timer as a free-running stopwatch.
     */
    void setup()
    {
        lnPeripherals::enable((Peripherals)(pTIMER0 + _timerIndex));
        _t->CTL0 = 0;
        _t->PSC = 0;
        _t->CAR = 0xFFFF;
        _t->CNT = 0;
        _t->CTL0 = LN_TIMER_CTL0_CEN;
    }

  protected:
    LN_Timers_Registers *_t;
    uint16_t _start_tick;
    int _timerIndex;
};
