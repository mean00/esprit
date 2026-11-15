
#include "esprit.h"

extern "C" void _write()
{
    xAssert(0);
}
extern "C" void *_sbrk(ptrdiff_t incr)
{
    xAssert(0);
    return 0;
}
extern "C" void _putchar(char) {};

#undef printf
#undef scanf

extern "C" int printf(const char *__restrict, ...)
{
    return 0;
}
extern "C" int scanf(const char *__restrict, ...)
{
    return 0;
}

extern "C" int _read()
{
    return 0;
}
extern "C" int _fstat()
{
    return 0;
}
extern "C" int _isatty()
{
    return 0;
}
extern "C" int _fini()
{
    return 0;
}
extern "C" int _close()
{
    return 0;
}
extern "C" int _lseek()
{
    return 0;
}
