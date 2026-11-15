/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */

#pragma once
extern "C"
{
    void __attribute__((noreturn)) do_assert(const char *a);
}
#define xAssert(a)                                                                                                     \
    if (!(a))                                                                                                          \
    {                                                                                                                  \
        do_assert(#a);                                                                                                 \
    }
