/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */

#pragma once
#if 1
// C linkage on purpose: `USE_RUST_LOGGER` replaces lnDebug.cpp with
// rust_esprit/src/logger_c.rs, and a Rust `#[unsafe(no_mangle)]` function cannot
// answer to a C++-mangled name (`_Z10LoggerInitv`).
extern "C" void LoggerInit();
extern "C" void debugLogger(const char *fmt, ...);
#else
#define LoggerInit()                                                                                                   \
    {                                                                                                                  \
    }
#define Logger(...)                                                                                                    \
    {                                                                                                                  \
    }
#endif

extern "C" void Logger(const char *fmt...);
extern "C" int Logger_C(const char *fmt, ...);
extern "C" void Logger_chars(int n, const char *data);
typedef void(lnLoggerFunction)(int n, const char *data);
extern "C" void setLogger(lnLoggerFunction *f);

