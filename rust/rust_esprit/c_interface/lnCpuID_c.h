#pragma once
#include <stdint.h>

#ifdef __cplusplus
extern "C"
{
#endif

    uint32_t ln_cpuid_vendor(void);
    uint32_t ln_cpuid_mcu(void);
    const char *ln_cpuid_mcu_as_string(void);
    int ln_cpuid_flash_size(void);
    int ln_cpuid_ram_size(void);
    const char *ln_cpuid_id_as_string(void);
    int ln_cpuid_clock_speed(void);
    uint32_t ln_cpuid_serial_id(void);

#ifdef __cplusplus
}
#endif
