/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 *  Each bit in the WS2812B_timer stream becomes a byte
 *
 * As far as the callback is concerned, it takes ~ 2.5 us on a VF103, 4 us on F303 and ~ 6 us on aF103
 */
#include "lnWS2812B_timer.h"
#include "esprit.h"

/**
 *
 * @param nbLeds
 * @param s
 */
WS2812B_timer::WS2812B_timer(int nbLeds, lnPin pin) : WS2812B_base(nbLeds)
{
    _pin = pin;
    _timer = new lnDmaTimer(8, pin);
}
/**
 *
 */
void WS2812B_timer::begin()
{
    lnPinMode(_pin, lnPWM, 5);
    if (!_timer->pwmSetup(830000))
    {
        xAssert(0);
    }
    _one = (_timer->rollover() * 4 + 3) / 6;
    _zero = (_timer->rollover() * 2 + 3) / 6;
    xAssert(_one < 256);
    xAssert(_zero < 256);
    _timer->attachDmaCallback(this);
    uint8_t *target = (uint8_t *)lookup;
    // Pre compute PWM for 4 bits value
    // Each value will come out as  4 bytes = uint32_t
    for (int value = 0; value < 16; value++)
    {
        int copy = value;
        for (int i = 0; i < 4; i++)
        {
            if (copy & 0x8)
                *target = _one;
            else
                *target = _zero;
            target++;
            copy <<= 1;
        }
    }
}

/**
 */
WS2812B_timer::~WS2812B_timer()
{
}
/**
 *
 * @param hilow
 * @param rgb
 */
uint32_t delta;
void WS2812B_timer::convertRgb(int hilow, uint8_t *rgb)
{

    uint32_t *t32 = (uint32_t *)_timerPwmValue;
    if (hilow)
        t32 += 6;             // 6xuint32_t = 24 bytes = 1 led worth
    xAssert(!((int)t32 & 3)); // 32 bits aligned
    int high = (rgb[0] >> 4) & 0xf;
    int low = rgb[0] & 0xf;

    t32[0] = lookup[high];
    t32[1] = lookup[low];
    high = (rgb[1] >> 4) & 0xf;
    low = rgb[1] & 0xf;
    t32[2] = lookup[high];
    t32[3] = lookup[low];
    high = (rgb[2] >> 4) & 0xf;
    low = rgb[2] & 0xf;
    t32[4] = lookup[high];
    t32[5] = lookup[low];
}

/**
 *
 */
void WS2812B_timer::update()
{
    convertRgb(false, _ledsColor);
    if (_nbLeds == 1)
    {
        _nextLed = 1;
        memset(_timerPwmValue + 24, 0, 24); // zero the 2nd part
    }
    else
    {
        _nextLed = 2;
        convertRgb(true, _ledsColor + 3);
    }
    // start PWM
    _timer->start(48, _timerPwmValue);
    _sem.take(100);
    _timer->stop();
}
/**
 *
 * @param half
 * @return
 */
bool WS2812B_timer::timerCallback(bool half)
{
    if (_nbLeds == 1 && half == true)
    {
        _sem.give();
        _nextLed++;
        return false;
    }
    if (_nextLed < _nbLeds)
    {
        convertRgb(!half, _ledsColor + 3 * _nextLed);
        _nextLed++;
        return true;
    }

    if (_nextLed == _nbLeds)
    {
        // ok we just wrote the last one, it is not sent yet
        _nextLed++;
        return true;
    }
    if (_nextLed == _nbLeds + 1)
    {
        _sem.give();
        _nextLed++;
        return false;
    }
    return true;
}
// EOF
