/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */
#include "lnTimer.h"
#include "esprit.h"
#include "lnBarrier.h"
#include "lnPinMapping.h"
#include "lnTimer_priv.h"
LN_Timers_Registers *aTimer0 = (LN_Timers_Registers *)(LN_TIMER0_ADR);
LN_Timers_Registers *aTimer1 = (LN_Timers_Registers *)(LN_TIMER1_ADR);
LN_Timers_Registers *aTimer2 = (LN_Timers_Registers *)(LN_TIMER2_ADR);
LN_Timers_Registers *aTimer3 = (LN_Timers_Registers *)(LN_TIMER3_ADR);
LN_Timers_Registers *aTimer4 = (LN_Timers_Registers *)(LN_TIMER4_ADR);

#define PMW_RANGE 100

#define aTimer(x) abTimers[x]

#define READ_CHANNEL_CTL(channel) (((t->CHCTLs[channel >> 1]) >> (8 * (channel & 1))) & 0xFF)
#define WRITE_CHANNEL_CTL(channel, val)                                                                                \
    {                                                                                                                  \
        int shift = 8 * (channel & 1);                                                                                 \
        uint32_t r = t->CHCTLs[channel >> 1];                                                                          \
        r &= ~(0xff << shift);                                                                                         \
        r |= (val & 0xff) << shift;                                                                                    \
        t->CHCTLs[channel >> 1] = r;                                                                                   \
    }

LN_Timers_Registers *abTimers[5] = {(LN_Timers_Registers *)(LN_TIMER0_ADR), (LN_Timers_Registers *)(LN_TIMER1_ADR),
                                    (LN_Timers_Registers *)(LN_TIMER2_ADR), (LN_Timers_Registers *)(LN_TIMER3_ADR),
                                    (LN_Timers_Registers *)(LN_TIMER4_ADR)};

#define aTimers(x) abTimers[x]

/**
 *
 * @param timer
 * @param channel
 */
lnTimer::lnTimer(uint32_t timer, uint32_t channel)
{
    _timer = timer;
    _channel = channel;
    lnPeripherals::enable((Peripherals)(pTIMER0 + _timer));
}
/**
 *
 * @param pin
 */
lnTimer::lnTimer(lnPin pin)
{
    const LN_PIN_MAPPING *pins = pinMappings;
    while (1)
    {
        xAssert(pins->pin != -1);
        if (pins->pin == pin)
        {
            _timer = pins->timer;
            _channel = pins->timerChannel;
            lnPeripherals::enable((Peripherals)(pTIMER0 + _timer));
            return;
        }
        pins++;
    }
}

/**
 */
lnTimer::~lnTimer()
{
}

/**
 *
 * @param timer
 */
void lnTimer::setPwmFrequency(uint32_t fqInHz)
{
    //--
    LN_Timers_Registers *t = aTimers(_timer);
    ;
    Peripherals per = pTIMER1;
    per = (Peripherals)((int)per + _timer - 1);
    uint32_t clock = lnPeripherals::getClock(per);
    // If ABP1 prescale=1, clock*=2 ???? see 5.2 in GD32VF103
    // disable
    t->CTL0 &= ~LN_TIMER_CTL0_CEN;

    int divider = (2 * clock + (fqInHz * PMW_RANGE / 2)) / (fqInHz * PMW_RANGE);

    if (!divider)
        divider = 1;
    t->PSC = divider - 1;
    t->CAR = PMW_RANGE - 1;
}
/**
 *
 * @param fqInHz
 */
void lnTimer::setTickFrequency(uint32_t fqInHz)
{
    LN_Timers_Registers *t = aTimers(_timer);
    ;
    Peripherals per = pTIMER1;
    per = (Peripherals)((int)per + _timer - 1);
    uint32_t clock = lnPeripherals::getClock(per);
    // If ABP1 prescale=1, clock*=2 ???? see 5.2 in GD32VF103
    // disable
    t->CTL0 &= ~LN_TIMER_CTL0_CEN;

    int divider = (clock + fqInHz / 2) / (fqInHz);
    divider *= 2;
    while (divider > 65535)
    {
        divider = divider / 2;
    }

    if (!divider)
        divider = 1;
    t->PSC = divider - 1;
}

void lnTimer::setMode(lnTimerMode mode)
{
    LN_Timers_Registers *t = aTimers(_timer);
    ;
    uint32_t chCtl = READ_CHANNEL_CTL(_channel);
    chCtl &= LN_TIME_CHCTL0_CTL_MASK;
    switch (mode)
    {
    case lnTimerModePwm1:
        chCtl |= LN_TIME_CHCTL0_CTL_PWM1;
        break;
    case lnTimerModePwm0:
        chCtl |= LN_TIME_CHCTL0_CTL_PWM0;
        break;
    default:
        xAssert(0);
        break;
    }
    chCtl &= LN_TIME_CHCTL0_MS_MASK;
    chCtl |= LN_TIME_CHCTL0_MS_OUPUT;
    WRITE_CHANNEL_CTL(_channel, chCtl)
}

/**
 *
 * @param timer
 * @param channel
 */
void lnTimer::setPwmMode(uint32_t ratio1000)
{
    LN_Timers_Registers *t = aTimers(_timer);
    ;

    setMode(lnTimerModePwm1);

    t->CHCVs[_channel] = ratio1000; // A/R
#if 0  
  t->CHCTL2 |=LN_TIMER_CHTL2_CHxP(_channel);
#else
    t->CHCTL2 &= ~(LN_TIMER_CHTL2_CHxP(_channel)); // active low
#endif
}
/**
 *
 */
void lnTimer::enable()
{
    LN_Timers_Registers *t = aTimers(_timer);
    ;
    t->CTL0 &= ~LN_TIMER_CTL0_CEN;
    t->CNT = t->CAR - 1;
    t->CHCTL2 |= LN_TIMER_CHTL2_CHxEN(_channel); // basic enable, active high
    t->CTL0 |= LN_TIMER_CTL0_CEN;
}
/**
 *
 */
void lnTimer::disable()
{
    LN_Timers_Registers *t = aTimers(_timer);
    ;
    t->CHCTL2 &= ~(LN_TIMER_CHTL2_CHxEN(_channel)); // basic enable, active high
    t->CTL0 &= ~LN_TIMER_CTL0_CEN;
    t->CNT = 0;
    uint32_t chCtl = READ_CHANNEL_CTL(_channel);
    chCtl &= LN_TIME_CHCTL0_CTL_MASK;
    chCtl |= LN_TIME_CHCTL0_CTL_FORCE_LOW;
    WRITE_CHANNEL_CTL(_channel, chCtl)
}

/**
 *
 * @param ratioBy100
 */
void lnTimer::setChannelRatio(uint32_t ratio1024)
{
    LN_Timers_Registers *t = aTimers(_timer);
    ;
    t->CHCVs[_channel] = ratio1024; // A/R
}
/**
 *
 * @param ratio1024
 */
#if 0
#define SPEEDUP 10
#else
#define SPEEDUP 1
#endif
void lnTimer::singleShot(uint32_t durationMs, bool up)
{
    LN_Timers_Registers *t = aTimers(_timer);

    xAssert(durationMs <= 100);
    // down = !down; // for some reason the code below has an inverted pulse
    //   lnNoInterrupt();
    disable();
    setTickFrequency(10 * 1000 * SPEEDUP); // 1 tick=1ms

    uint32_t duration_tick = 10 * durationMs;
    t->CAR = duration_tick;
    t->CHCVs[_channel] = duration_tick;

    // --- 1. SET THE DEFAULT OUTPUT STATE BEFORE ENABLING THE CHANNEL ---
    // disable() left O0CPRE forced low. For a negative pulse we need the
    // default to be VCC, so force high first, then enable the channel
    // while still in FORCE_HIGH mode so the pin goes to VCC.
    uint32_t ctl = READ_CHANNEL_CTL(_channel);
    ctl &= LN_TIME_CHCTL0_CTL_MASK; // Clear CH0COMCTL bits (Bits 4-6)
    if (up)
    {
        // Negative pulse: default=VCC, pulse=GND for duration, back to VCC
        // Step A: Force high while channel is still disabled (CHxEN=0)
        ctl |= LN_TIME_CHCTL0_CTL_FORCE_HIGH;
        WRITE_CHANNEL_CTL(_channel, ctl);

        // Step B: Enable the channel NOW while still in FORCE_HIGH mode.
        // This connects the timer output to the pin at VCC (default state).
        uint32_t ctl2 = t->CHCTL2;
        ctl2 &= ~(LN_TIMER_CHTL2_CHxP(_channel));
        ctl2 |= (LN_TIMER_CHTL2_CHxEN(_channel));
        t->CHCTL2 = ctl2;

        // Step C: Switch to PWM1 mode. With CEN=0 and CNT=0 < CHCV,
        // PWM1 outputs inactive (low=GND). The pin immediately goes
        // from VCC to GND — this IS the start of the negative pulse.
        ctl &= LN_TIME_CHCTL0_CTL_MASK;
        ctl |= LN_TIME_CHCTL0_CTL_PWM1;
        WRITE_CHANNEL_CTL(_channel, ctl);
    }
    else
    {
        // Positive pulse: default=GND, pulse=VCC for duration, back to GND
        // disable() already forced low, so O0CPRE is already GND (default).
        // Switch to PWM0: CNT < CHCV -> active (high=VCC = pulse),
        // CNT >= CHCV -> inactive (low=GND = default)
        ctl |= LN_TIME_CHCTL0_CTL_PWM0;
        WRITE_CHANNEL_CTL(_channel, ctl);

        // Enable the channel (CHxEN) — pin goes from high-impedance to
        // O0CPRE which is low (GND = default). Then CEN starts the counter.
        uint32_t ctl2 = t->CHCTL2;
        ctl2 &= ~(LN_TIMER_CHTL2_CHxP(_channel));
        ctl2 |= (LN_TIMER_CHTL2_CHxEN(_channel));
        t->CHCTL2 = ctl2;
    }

    t->CTL0 |= LN_TIMER_CTL0_SPM; // Set Bit 3 (SPM), single shot

    t->SWEV |=
        LN_TIMER_SWEVG_UPG; // Set Bit 6 (UPG) to force a software update event, latching the configurations immediately
    t->INTF &= ~(1 << 0);   // Clear UPIF
    //
    // --- 6. TRIGGER THE HARDWARE PULSE ---
    t->CTL0 |= LN_TIMER_CTL0_CEN; // Set Bit 0 (CEN) to start the hardware counter
    // Read-back CTL0 to flush the AHB→APB write buffer, ensuring the
    // CEN=1 write has reached the timer before we poll for CEN to clear.
    (void)t->CTL0;
    LN_DATA_BARRIER();
    LN_SYNC_BARRIER();

    // --- 7. WAIT FOR AUTOMATIC HARDWARE SHUTDOWN ---
    // The hardware automatically sets the output, increments the counter, resets the output, and shuts off CEN.
    while ((t->CTL0 & LN_TIMER_CTL0_CEN) != 0)
    {
        // Polling loop waiting exclusively for the hardware to drop the CEN bit back to 0 on completion
        __asm__("nop");
    }
}
/**
 *
 * @return
 */
int lnAdcTimer::getPwmFrequency()
{
    return _actualPwmFrequency;
}
/**
 *
 * @param fqInHz
 */
void lnAdcTimer::setPwmFrequency(int fqInHz)
{
    LN_Timers_Registers *t = aTimers(_timer);
    ;
    Peripherals per = pTIMER1;
    per = (Peripherals)((int)per + _timer - 1);
    uint32_t clock = lnPeripherals::getClock(per);
    // If ABP1 prescale=1, clock*=2 ???? see 5.2.1 in GD32VF103
    if (_timer)
    {
        // Timer0,7,8,9 is connected to APB2 with prescaler==1 so no x2
        // timer 1, 2,3,4 are connected to APB1 with prescaler =1/2, so *2
        clock = clock * 2;
    }
    // disable
    t->CTL0 &= ~LN_TIMER_CTL0_CEN;

    int divider = clock / (fqInHz);

    int intDiv = divider / 65536;
    int fracDiv = divider & 65535;

    int totalDivider = (intDiv << 16) + fracDiv;
    _actualPwmFrequency = clock / totalDivider;

    Logger("Adc : Asked for fq=%d got fq=%d\n", fqInHz, _actualPwmFrequency);
    Logger("intDiv:%d intFrac=%d\n", intDiv, fracDiv);
    t->PSC = intDiv; // 0 => not divided
    // Set reload to 0
    t->CAR = fracDiv - 1;

    uint32_t chCtl = READ_CHANNEL_CTL(_channel);

    chCtl &= LN_TIME_CHCTL0_CTL_MASK;
    chCtl |= LN_TIME_CHCTL0_CTL_PWM0;
    chCtl &= LN_TIME_CHCTL0_MS_MASK;
    chCtl |= LN_TIME_CHCTL0_MS_OUPUT;

    WRITE_CHANNEL_CTL(_channel, chCtl)

    t->CHCVs[_channel] = 1;                        // A/R
    t->CHCTL2 &= ~(LN_TIMER_CHTL2_CHxP(_channel)); // active high ?
}

/**
 *
 * @param timer
 */

void lnSquareSignal::setFrequency(uint32_t fqInHz)
{
    LN_Timers_Registers *t = aTimers(_timer);
    ;
    Peripherals per = pTIMER1;

    t->CTL0 &= ~LN_TIMER_CTL0_CEN; // disable
    setMode(lnTimerModePwm1);

    t->CHCVs[_channel] = PMW_RANGE / 2;            // A/R
    t->CHCTL2 &= ~(LN_TIMER_CHTL2_CHxP(_channel)); // active low
                                                   //--

    per = (Peripherals)((int)per + _timer - 1);
    uint32_t clock = lnPeripherals::getClock(per);
    // If ABP1 prescale=1, clock*=2 ???? see 5.2 in GD32VF103
    // disable
    if (_timer)
        clock *= 2;

    int divider = (clock + (fqInHz * PMW_RANGE / 2)) / (fqInHz * PMW_RANGE);

    if (!divider)
        divider = 1;
    t->PSC = divider - 1;
    t->CAR = PMW_RANGE - 1;
}

// EOF
