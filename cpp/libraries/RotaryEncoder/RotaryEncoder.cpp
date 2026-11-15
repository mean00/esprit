#include "RotaryEncoder.h"
#include "esprit.h"

static lnRotary *current = NULL;
#define THRESHOLD 3          // 3ms
#define TIME_LONG_PRESS 1000 // 2sec
#define TIME_SHORT_PRESS 8

#define R_START 0x0

#ifdef HALF_STEP
// Use the half-step state table (emits a code at 00 and 11)
#define R_CCW_BEGIN 0x1
#define R_CW_BEGIN 0x2
#define R_START_M 0x3
#define R_CW_BEGIN_M 0x4
#define R_CCW_BEGIN_M 0x5
const unsigned char ttable[6][4] = {
    // R_START (00)
    {R_START_M, R_CW_BEGIN, R_CCW_BEGIN, R_START},
    // R_CCW_BEGIN
    {R_START_M | DIR_CCW, R_START, R_CCW_BEGIN, R_START},
    // R_CW_BEGIN
    {R_START_M | DIR_CW, R_CW_BEGIN, R_START, R_START},
    // R_START_M (11)
    {R_START_M, R_CCW_BEGIN_M, R_CW_BEGIN_M, R_START},
    // R_CW_BEGIN_M
    {R_START_M, R_START_M, R_CW_BEGIN_M, R_START | DIR_CW},
    // R_CCW_BEGIN_M
    {R_START_M, R_CCW_BEGIN_M, R_START_M, R_START | DIR_CCW},
};
#else
// Use the full-step state table (emits a code at 00 only)
#define R_CW_FINAL 0x1
#define R_CW_BEGIN 0x2
#define R_CW_NEXT 0x3
#define R_CCW_BEGIN 0x4
#define R_CCW_FINAL 0x5
#define R_CCW_NEXT 0x6

// Enable this to emit codes twice per step.
// #define HALF_STEP

// Values returned by 'process'
// No complete step yet.
#define DIR_NONE 0x0
// Clockwise step.
#define DIR_CW 0x10
// Counter-clockwise step.
#define DIR_CCW 0x20

const unsigned char ttable[7][4] = {
    // R_START
    {R_START, R_CW_BEGIN, R_CCW_BEGIN, R_START},
    // R_CW_FINAL
    {R_CW_NEXT, R_START, R_CW_FINAL, R_START | DIR_CW},
    // R_CW_BEGIN
    {R_CW_NEXT, R_CW_BEGIN, R_START, R_START},
    // R_CW_NEXT
    {R_CW_NEXT, R_CW_BEGIN, R_CW_FINAL, R_START},
    // R_CCW_BEGIN
    {R_CCW_NEXT, R_START, R_CCW_BEGIN, R_START},
    // R_CCW_FINAL
    {R_CCW_NEXT, R_CCW_FINAL, R_START, R_START | DIR_CCW},
    // R_CCW_NEXT
    {R_CCW_NEXT, R_CCW_FINAL, R_CCW_BEGIN, R_START},
};
#endif

/**
 */

static void myInterrupt(lnPin pin, void *arg)
{
    lnRotary *w = (lnRotary *)arg;
    w->rotaryInterrupt();
}
static void myPushInterrupt(lnPin pin, void *arg)
{
    lnRotary *w = (lnRotary *)arg;
    w->pushInterrupt();
}
/**
 *
 */
void lnRotary::pushInterrupt()
{
    bool state = lnDigitalRead(_pinPush);
    uint32_t m = lnGetMs();
    if ((m - _lastRead) < THRESHOLD)
        return;
    if (!state) // down
    {
        _down = m;
        return;
    }
    else
    {
        uint32_t time = m - _down;
        if (time > TIME_LONG_PRESS)
        {
            _events.setEvents(LONG_PRESS);
        }
        else
        {
            if (time > TIME_SHORT_PRESS)
            {
                _events.setEvents(SHORT_PRESS);
            }
        }
    }
}
/**
 *
 * @return
 */
int lnRotary::process()
{
    // Grab state of input pins.
    unsigned char pinstate = (lnDigitalRead(_pinA) << 1) | lnDigitalRead(_pinB);

    // Determine new state from the pins and state table.
    _state = ttable[_state & 0xf][pinstate];
    // Return emit bits, ie the generated event.
    return _state & 0x30;
}
/**
 */
void lnRotary::rotaryInterrupt()
{
    switch (process())
    {
    case DIR_CCW:
        _count++;
        _events.setEvents(ROTARY_CHANGE);
        break;
    case DIR_CW:
        _count--;
        _events.setEvents(ROTARY_CHANGE);
        break;
    default:
        break;
    }
}
/**
 */
lnRotary::lnRotary(lnPin pinPush, lnPin pinA, lnPin pinB)
{
    current = this;
    _count = 0;
    _event = EVENT_NONE;
    _down = 0;
    _lastRead = 0;
    _pinA = pinA;
    _pinB = pinB;
    _pinPush = pinPush;
    lnPinMode(_pinA, lnINPUT_PULLUP);
    lnPinMode(_pinB, lnINPUT_PULLUP);
    lnPinMode(_pinPush, lnINPUT_PULLUP);
    _state = R_START;
}

/**
 *
 */
void lnRotary::start()
{
    lnNoInterrupt();
    lnExtiAttachInterrupt(_pinA, LN_EDGE_BOTH, myInterrupt, this);
    lnExtiAttachInterrupt(_pinB, LN_EDGE_BOTH, myInterrupt, this);
    lnExtiAttachInterrupt(_pinPush, LN_EDGE_BOTH, myPushInterrupt, this);
    lnExtiEnableInterrupt(_pinA);
    lnExtiEnableInterrupt(_pinB);
    lnExtiEnableInterrupt(_pinPush);
    lnInterrupts();
}
/*
 */
int lnRotary::getCount()
{
    lnNoInterrupt();
    int c = _count;
    _count = 0;
    lnInterrupts();
    return c;
}
/**
 *
 * @return
 */
lnRotary::EVENTS lnRotary::readEvent()
{
    return (EVENTS)_events.readEvents(SHORT_PRESS | LONG_PRESS | ROTARY_CHANGE);
}
/**
 *
 * @return
 */
lnRotary::EVENTS lnRotary::waitForEvent(int timeout)
{
    return (EVENTS)_events.waitEvents(SHORT_PRESS | LONG_PRESS | ROTARY_CHANGE, timeout);
}
// EOF
