/******************************************************************************
 * The MIT License
 *
 * Copyright (c) 2011 LeafLabs, LLC.
 *
 * Permission is hereby granted, free of charge, to any person
 * obtaining a copy of this software and associated documentation
 * files (the "Software"), to deal in the Software without
 * restriction, including without limitation the rights to use, copy,
 * modify, merge, publish, distribute, sublicense, and/or sell copies
 * of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be
 * included in all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
 * EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
 * MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
 * NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS
 * BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN
 * ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 *****************************************************************************/

/*
 * This file is a modified version of a file obtained from
 * CodeSourcery Inc. (now part of Mentor Graphics Corp.), in which the
 * following text appeared:
 *
 * Copyright (c) 2006, 2007 CodeSourcery Inc
 *
 * The authors hereby grant permission to use, copy, modify, distribute,
 * and license this software and its documentation for any purpose, provided
 * that existing copyright notices are retained in all copies and that this
 * notice is included verbatim in any distributions. No written agreement,
 * license, or royalty fee is required for any of the authorized uses.
 * Modifications to this software may be copyrighted by their authors
 * and need not follow the licensing terms described here, provided that
 * the new terms are clearly indicated on the first page of each file where
 * they apply.
 */

#include "esprit.h"
#include <stddef.h>

extern int main(void);

extern "C"
{
    void __libc_init_array(void);
    /* The linker must ensure that these are at least 4-byte aligned. */
    extern const char __data_start__, __data_end__;
    extern const char __bss_start__, __bss_end__;
    extern const char _lm_rom_img_cfgp;

    // stub
    void _init(void)
    {
    }
}

/**
 * \fn start_c
 * \brief Initialize ram & libc runtime before jumping into actual code
 * careful the libc is not compltetely initialized here, some stuff doesnt work yet
 */
extern "C" void __attribute__((noreturn)) start_c(void)
{
    uint32_t srcAdr = *(uint32_t *)(&_lm_rom_img_cfgp);
    volatile uint32_t *src = (volatile uint32_t *)srcAdr;
    volatile uint32_t *dst = (volatile uint32_t *)&__data_start__;
    volatile uint32_t *end = (volatile uint32_t *)&__data_end__;

    while (dst < end)
    {
        *dst++ = *src++;
    }

    /* Zero .bss. */
    volatile uint32_t *zstart = (volatile uint32_t *)&__bss_start__;
    volatile uint32_t *zend = (volatile uint32_t *)&__bss_end__;
    while (zstart < zend)
    {
        *zstart++ = 0;
    }

    /* Run initializers. */
    __libc_init_array();
    main();
    xAssert(0);
    for (;;)
        continue;
}

// EOF