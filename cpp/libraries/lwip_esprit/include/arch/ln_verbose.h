#pragma once
// #include "arch/cc.h"
extern int Logger_C(const char *fmt, ...);
extern void deadEnd(int a);
#undef LWIP_PLATFORM_DIAG
#undef LWIP_PLATFORM_ASSERT
#define LWIP_PLATFORM_DIAG(x) Logger_C x
#define LWIP_PLATFORM_ASSERT(x) deadEnd(0)
