/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */

#pragma once
#include "esprit.h"

class lnStopWatch
{
  public:
    lnStopWatch(uint32_t ms);
    bool restart(uint32_t durationMs);
    bool elapsed();

  protected:
    uint32_t _start;
    uint32_t _end;
};
/**
 *
 */
class lnCycleClock
{
  public:
    lnCycleClock();
    void restart();
    uint32_t elapsed();

  protected:
    uint32_t _start;
};
