/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */
#include "lnBasicTimer.h"
#include "esprit.h"
#include "lnBasicTimer_priv.h"
#include "lnPinMapping.h"

LN_BTimers_Registers *aTimer5 = (LN_BTimers_Registers *)(LN_TIMER5_ADR);
LN_BTimers_Registers *aTimer6 = (LN_BTimers_Registers *)(LN_TIMER6_ADR);

LN_BTimers_Registers *aBTimers[2] = {(LN_BTimers_Registers *)(LN_TIMER5_ADR), (LN_BTimers_Registers *)(LN_TIMER6_ADR)};
/**
 *
 * @param timer
 * @param channel
 */
lnBasicTimer::lnBasicTimer(int timer)
{
    lnPeripherals::enable((Peripherals)(pTIMER5 + timer));
    _timer = timer;
    LN_BTimers_Registers *t = aBTimers[_timer];
    t->DMAINTEN = 0;
    t->INTF = 0;
    t->CTL0 = 0;
}

/**
 */
lnBasicTimer::~lnBasicTimer()
{
    disable();
}
/**
 *
 * @param prediv
 * @param reload
 */
void lnBasicTimer::setTimerFrequency(int prediv, int reload)
{
    xAssert(prediv);
    xAssert(reload);
    LN_BTimers_Registers *t = aBTimers[_timer];
    t->PSC = prediv - 1;
    t->CAR = reload - 1;
}
/**
 *
 * @param timer
 */
void lnBasicTimer::setTimerFrequency(int fqInHz)
{
    LN_BTimers_Registers *t = aBTimers[_timer];
    Peripherals per = pTIMER5;
    per = (Peripherals)((int)per + _timer);

    int clock = lnPeripherals::getClock(per);
    int divider = clock / (float)fqInHz;

    int psc = 1;
    while ((int)divider > 65535 * psc)
    {
        psc++;
    }
    disable();
    t->CTL0 |= LN_BTIMER_CTL0_ARSE;

    uint32_t car = ((int)divider) / psc;
    if (car)
        car--;
    if (psc)
        psc = psc - 1;
    t->PSC = psc;

    t->CAR = car;

    Logger("PSC=%d CAR=%d  \n", psc + 1, car + 1);
    t->CTL1 = (2 << 4); // issue TRGO
}
/**
 */
int lnBasicTimer::getTimerFrequency()
{
    LN_BTimers_Registers *t = aBTimers[_timer];
    int car = t->CAR + 1;
    int div = t->PSC + 1;
    Peripherals per = pTIMER5;
    per = (Peripherals)((int)per + _timer);
    int fq = lnPeripherals::getClock(per) / (car * div);
    return fq;
}
/**
 *
 */
void lnBasicTimer::enable()
{
    LN_BTimers_Registers *t = aBTimers[_timer];
    t->CTL0 |= LN_BTIMER_CTL0_EN; // basic enable, active high
}
/**
 *
 */
void lnBasicTimer::disable()
{
    LN_BTimers_Registers *t = aBTimers[_timer];
    t->CTL0 &= ~LN_BTIMER_CTL0_EN; // basic enable, active high
}
// EOF