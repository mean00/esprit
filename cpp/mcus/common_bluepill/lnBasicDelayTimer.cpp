/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */
#include "esprit.h"
#include "lnBasicTimer.h"
#include "lnBasicTimer_priv.h"
#include "lnPinMapping.h"

#pragma clang diagnostic ignored "-Wextra"

lnBasicDelayTimer *btimerInstances[2] = {NULL, NULL};
extern LN_BTimers_Registers *aBTimers[];

/**
 *
 * @param timer
 * @param channel
 */
lnBasicDelayTimer::lnBasicDelayTimer(int timer) : lnBasicTimer(timer)
{
    xAssert(!btimerInstances[timer]);
    btimerInstances[timer] = this;
    _handler = NULL;
    _cookie = NULL;
}

/**
 */
lnBasicDelayTimer::~lnBasicDelayTimer()
{
    disable();
    disableInterrupt();
    _handler = NULL;
    _cookie = NULL;
    btimerInstances[_timer] = NULL;
}
/**
 *
 */
void lnBasicDelayTimer::enableInterrupt()
{
    lnEnableInterrupt((LnIRQ)(LN_IRQ_TIMER5 + _timer));
    LN_BTimers_Registers *t = aBTimers[_timer];
    t->DMAINTEN |= LN_BTIMER_DMAINTEN_UPIE;
}

/**
 *
 */
void lnBasicDelayTimer::disableInterrupt()
{
    LN_BTimers_Registers *t = aBTimers[_timer];
    t->DMAINTEN &= ~LN_BTIMER_DMAINTEN_UPIE;
    lnDisableInterrupt((LnIRQ)(LN_IRQ_TIMER5 + _timer));
}
/**
 *
 * @param handler
 * @param cookie
 */
void lnBasicDelayTimer::setInterrupt(lnBasicTimerIrq *handler, void *cookie)
{
    _handler = handler;
    _cookie = cookie;
}
/**
 *
 * @param id
 */
void lnBasicDelayTimer::interruptHandler(int id)
{
    lnBasicDelayTimer *instance = btimerInstances[id];
    xAssert(instance);
    instance->irq();
}
/**
 *
 */
void lnBasicDelayTimer::irq(void)
{
    disableInterrupt();
    disable();
    LN_BTimers_Registers *t = aBTimers[_timer];
    t->INTF &= ~LN_BTIMER_INTF_UPIF;
    xAssert(_handler);
    _handler(_cookie);
}
/**
 *
 */
void lnBasicDelayTimer::arm(int durationUs)
{
    disableInterrupt();
    disable();
    // Compute # of ticks
    LN_BTimers_Registers *t = aBTimers[_timer];
    Peripherals per = pTIMER5;
    per = (Peripherals)((int)per + _timer);

    int clock = lnPeripherals::getClock(per) / 1000000; // In Mhz => In us

    int cycles = durationUs * clock;
    int psc = cycles >> 16;
    cycles &= 0xffff;

    t->CTL0 = LN_BTIMER_CTL0_SPM + LN_BTIMER_CTL0_UPS;
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
extern "C" void TIMER5_IRQHandler() LN_INTERRUPT_TYPE;
extern "C" void TIMER6_IRQHandler() LN_INTERRUPT_TYPE;

void TIMER5_IRQHandler()
{
    lnBasicDelayTimer::interruptHandler(0);
}
/**
 *
 */
void TIMER6_IRQHandler()
{
    lnBasicDelayTimer::interruptHandler(1);
}
// EOF
