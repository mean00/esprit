#include "esprit.h"
#include "hello.h"

extern void lnusb_set_global_configuration(const uint8_t *hsConfiguration, const uint8_t *fsConfiguration,
                                           const tusb_desc_device_t *desc, const tusb_desc_device_qualifier_t *qual,
                                           int udesc_size, const char **udesc);

extern "C" void user_init();

void setup()
{
    lnusb_set_global_configuration(desc_hs_configuration, desc_fs_configuration,
                                   &desc_device, &desc_device_qualifier,
                                   5, (const char **)descriptor);
}

void loop()
{
    user_init();
    while (1)
    {
        lnDelayMs(1000);
    }
}

#ifndef __clang__
extern "C" void _exit(int code)
{
    deadEnd(code);
    while (1)
    {
    }
}
#endif
