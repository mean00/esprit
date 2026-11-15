/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */
#pragma once
#define LN_SYSTEM_LED PC13
#define PPINS(a, val)                                                                                                  \
    P##a##0 = val, P##a##1, P##a##2, P##a##3, P##a##4, P##a##5, P##a##6, P##a##7, P##a##8, P##a##9, P##a##10,          \
    P##a##11, P##a##12, P##a##13, P##a##14, P##a##15,
/**
 */
enum lnPin : int
{
    PPINS(A, 0) PPINS(B, 16) PPINS(C, 32) PPINS(D, 48) PPINS(E, 64) NoPin = -1
};
