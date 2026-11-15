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

#pragma once
#ifdef __cplusplus
extern "C"
{
#endif

#ifdef LN_CUSTOM_FREERTOS
#include "lnFreeRTOS_custom.h"
#elif defined(LN_NO_FREERTOS)
// no freertos
#define TaskHandle_t int
#define EventGroupHandle_t int
#define SemaphoreHandle_t int
#define TimerHandle_t int
/* The C++ wrappers (lnFreeRTOS_pp.h) keep the kernel object's static storage
 * inside the object, so these type names have to exist here too.  Nothing in
 * this flavor ever calls the kernel, so plain dummies are enough: they only
 * need to be named types, their size is never used. */
typedef int StaticSemaphore_t;
typedef int StaticEventGroup_t;
typedef int StaticTimer_t;
typedef int StaticTask_t;
typedef int StackType_t;
#else
#include "lnFreeRTOS_default.h"
#endif

#ifdef __cplusplus
}
#endif
