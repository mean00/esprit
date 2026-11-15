/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */
#pragma once
#define LN_SYSTEM_LED GPIO4
/**
 */
#define PINRANGE(x)                                                                                                    \
    GPIO##x##0, GPIO##x##1, GPIO##x##2, GPIO##x##3, GPIO##x##4, GPIO##x##5, GPIO##x##6, GPIO##x##7, GPIO##x##8,        \
        GPIO##x##9

enum lnPin : int
{
    GPIO0 = 0,
    GPIO1,
    GPIO2,
    GPIO3,
    GPIO4,
    GPIO5,
    GPIO6,
    GPIO7,
    GPIO8,
    GPIO9,
    PINRANGE(1),
    PINRANGE(2),
    PINRANGE(3),
    NoPin = -1
};
