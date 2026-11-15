/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */
#pragma once
#include "esprit.h"

struct LN_TIMER_MAPPING
{
    int8_t TimerChannel; // timer*10+channel
    int8_t dmaEngine;
    int8_t dmaChannel;
};

struct LN_PIN_MAPPING
{
    int8_t pin;
    int8_t adc;
    int8_t timer;
    int8_t timerChannel;
    int8_t dac;
    int8_t filler;
};

extern const LN_PIN_MAPPING pinMappings[];
extern const LN_TIMER_MAPPING timerMappings[];
