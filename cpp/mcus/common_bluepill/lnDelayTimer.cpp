/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 *
 * This is basically a copy/paste of basicDelayTimer
 * It is most probably incomplete
 *
 */
#include "esprit.h"
#include "lnPinMapping.h"
#include "lnTimer.h"
#include "lnTimer_priv.h"

#pragma clang diagnostic ignored "-Wextra"

#warning PROPERLY INITIALIZE IT
#warning PROPERLY INITIALIZE IT
#warning PROPERLY INITIALIZE IT
#warning PROPERLY INITIALIZE IT

lnDelayTimer *timerInstances[5] = {NULL, NULL, NULL, NULL, NULL};
extern LN_Timers_Registers *abTimers[];

/**
 *
 * @param timer
 * @param channel
 */
lnDelayTimer::lnDelayTimer(int timer, int channel) : lnTimer(timer, channel)
{
    if (timer > 3 || !timer)
        xAssert(0);
    xAssert(!timerInstances[timer]);
    timerInstances[timer] = this;
    _handler = NULL;
    _cookie = NULL;
}

/**
 */
lnDelayTimer::~lnDelayTimer()
{
    disable();
    disableInterrupt();
    _handler = NULL;
    _cookie = NULL;
    timerInstances[_timer] = NULL;
}
/**
 *
 */
void lnDelayTimer::enableInterrupt()
{
    lnEnableInterrupt((LnIRQ)(LN_IRQ_TIMER1 + _timer - 1));
    LN_Timers_Registers *t = abTimers[_timer];
    t->DMAINTEN |= LN_TIMER_DMAINTEN_UPIE;
}

/**
 *
 */
void lnDelayTimer::disableInterrupt()
{
    LN_Timers_Registers *t = abTimers[_timer];
    t->DMAINTEN &= ~LN_TIMER_DMAINTEN_UPIE;
    lnDisableInterrupt((LnIRQ)(LN_IRQ_TIMER1 + _timer - 1));
}
/**
 *
 * @param handler
 * @param cookie
 */
void lnDelayTimer::setInterrupt(lnTimerIrq *handler, void *cookie)
{
    _handler = handler;
    _cookie = cookie;
}
/**
 *
 * @param id
 */
void lnDelayTimer::interruptHandler(int id)
{
    lnDelayTimer *instance = timerInstances[id];
    xAssert(instance);
    instance->irq();
}
/**
 *
 */
void lnDelayTimer::irq(void)
{
    disableInterrupt();
    disable();
    LN_Timers_Registers *t = abTimers[_timer];
    t->INTF &= ~LN_TIMER_INTF_UPIF;
    xAssert(_handler);
    _handler(_cookie);
}
/**
 *
 */
void lnDelayTimer::arm(int durationUs)
{
    disableInterrupt();
    disable();
    // Compute # of ticks
    LN_Timers_Registers *t = abTimers[_timer];
    Peripherals per = pTIMER0;
    per = (Peripherals)((int)per + _timer);

    int clock = lnPeripherals::getClock(per) / 1000000; // In Mhz => In us

    int cycles = durationUs * clock;
    int psc = cycles >> 16;
    cycles &= 0xffff;

    t->CTL0 = LN_TIMER_CTL0_SPM + LN_TIMER_CTL0_UPS;
    t->CTL1 = 0;
    uint32_t car = cycles;
    if (car)
        car--;
    t->DMAINTEN = 0;
    t->INTF = 0;
    // t->SWEV=LN_BTIMER_SWEVG_UPG;
    t->PSC = psc;
    t->CAR = car;
    t->CNT = 0;

    // t->CTL1=(2<<4); // issue TRGO
    enableInterrupt();
    enable();
}
/**
 *
 */

extern "C" void TIMER1_IRQHandler() LN_INTERRUPT_TYPE;
extern "C" void TIMER2_IRQHandler() LN_INTERRUPT_TYPE;
extern "C" void TIMER3_IRQHandler() LN_INTERRUPT_TYPE;
extern "C" void TIMER4_IRQHandler() LN_INTERRUPT_TYPE;

void TIMER1_IRQHandler()
{
    lnDelayTimer::interruptHandler(1);
}
void TIMER2_IRQHandler()
{
    lnDelayTimer::interruptHandler(2);
}
void TIMER3_IRQHandler()
{
    lnDelayTimer::interruptHandler(3);
}
void TIMER4_IRQHandler()
{
    lnDelayTimer::interruptHandler(4);
}
// EOF
