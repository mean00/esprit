#include "lnCpuID.h"
extern "C"
{
#include "lnCpuID_c.h"
}

uint32_t ln_cpuid_vendor(void)
{
    return (uint32_t)lnCpuID::vendor();
}

uint32_t ln_cpuid_mcu(void)
{
    return (uint32_t)lnCpuID::mcu();
}

const char *ln_cpuid_mcu_as_string(void)
{
    return lnCpuID::mcuAsString();
}

int ln_cpuid_flash_size(void)
{
    return lnCpuID::flashSize();
}

int ln_cpuid_ram_size(void)
{
    return lnCpuID::ramSize();
}

const char *ln_cpuid_id_as_string(void)
{
    return lnCpuID::idAsString();
}

int ln_cpuid_clock_speed(void)
{
    return lnCpuID::clockSpeed();
}

uint32_t ln_cpuid_serial_id(void)
{
    return lnCpuID::getSerialID();
}
