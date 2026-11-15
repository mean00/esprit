/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */

//
//.#include "stdarg.h"
//
//.#include "stdint.h"
//
//.#include "stdio.h"

#pragma clang diagnostic ignored "-Wextra"

extern "C"
{
// #include "riscv_encoding.h"
#include "FreeRTOS.h"
}
#include "systemHelper.h"

#include "lnSystemTime.h"
extern "C" void do_assert(const char *a);
#undef stderr
extern void *const stderr;
void *const stderr = NULL;
#undef vfprintf

/**
 */

extern "C"
{
    void _exit(int code)
    {
        deadEnd(code + 0x2000);
        while (1)
        {
        }
    }
    void panic(const char *fmt, ...)
    {
        do_assert("panic");
    }
    int _getpid()
    {
        return 0;
    }
    int _kill()
    {
        return 0;
    }
    int getpid(void)
    {
        do_assert("getpid");
        return 0;
    }
    int kill(int pid, int sig)
    {
        do_assert("kill");
        return 0;
    }
    int getppid(void)
    {
        do_assert("getppid");
        return 0;
    }
    void __cxa_pure_virtual()
    {
        do_assert("__cxa_pure_virtual");
    }
    uint32_t core_exception_handler(unsigned long mcause, unsigned long sp)
    {
        do_assert("core_exception_handler");
        return 0;
    }
    void abort()
    {
        do_assert("abort");
        while (1)
        {
        }
    }
    int vfprintf(void *)
    {
        do_assert("vfprintf");
        return 0;
    }
    int fprintf(void *)
    {
        do_assert("vfprintf");
        return 0;
    }
}

namespace std
{
void __throw_out_of_range(const char *a) //__throw_out_of_range
{
    do_assert(a);
}
void __throw_length_error(const char *a) //__throw_out_of_range
{
    do_assert(a);
}
} // namespace std
/**

*/
// EOF
