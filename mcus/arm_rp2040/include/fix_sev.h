#ifndef __ASSEMBLER__
#include "hardware/sync.h"
#ifndef __sev
#define __sev() __asm volatile("sev" : : : "memory")
#endif
#ifndef __wfe
#define __wfe() __asm volatile("wfe" : : : "memory")
#endif
#endif
#ifndef exception_is_compile_time_default
#define exception_is_compile_time_default(x) true
#endif
