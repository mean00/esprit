
#pragma once
#ifndef __cplusplus
extern void Logger_C(const char *fmt, ...);
#undef printf
#define printf Logger_C
#else
extern "C" void Logger(const char *fmt, ...);
#undef printf
#define printf Logger
#endif
