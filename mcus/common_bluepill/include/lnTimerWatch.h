#pragma once

#include "lnBasicTimer.h"
#include "lnBasicTimer_priv.h"

extern LN_BTimers_Registers *aBTimers[];

/**
 * @brief An inline-heavy timer watcher based on lnBasicTimer.
 * Designed for highly precise, lock-free timer polling.
 */
class lnTimerWatch : public lnBasicTimer
{
  public:
    /**
     * @param timer The basic timer index (e.g., 0 for TIMER5).
     */
    lnTimerWatch(int timer) : lnBasicTimer(timer)
    {
        _t = aBTimers[_timer];
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
        _t->PSC = 0;
        _t->CAR = 0xFFFF;
        _t->CNT = 0;
        _t->CTL0 = LN_BTIMER_CTL0_EN;
    }

  protected:
    LN_BTimers_Registers *_t;
    uint16_t _start_tick;
};
