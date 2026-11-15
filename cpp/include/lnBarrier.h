#pragma once

#ifdef __arm__
#define LN_DATA_BARRIER()                                                                                              \
    {                                                                                                                  \
        __asm__ __volatile__("dmb" : : : "memory");                                                                    \
    }
#define LN_SYNC_BARRIER()                                                                                              \
    {                                                                                                                  \
        __asm__ __volatile__("dsb" : : : "memory");                                                                    \
    }
#elif __riscv
#define LN_DATA_BARRIER()                                                                                              \
    {                                                                                                                  \
        __asm__ __volatile__("fence" ::: "memory");                                                                    \
    }
#define LN_SYNC_BARRIER()                                                                                              \
    {                                                                                                                  \
        __asm__ __volatile__("fence iorw, iorw" ::: "memory");                                                         \
    }
#elif
#error "Unsupported arch"
#endif
