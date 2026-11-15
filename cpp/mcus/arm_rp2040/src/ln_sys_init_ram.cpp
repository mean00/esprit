
#include <stdint.h>
extern "C"
{
    void runtime_init();
    int main();
    /**
     * This is the ram initilization when everything is in ram
     *
     */
    void main_ram(void)
    {
        extern void *__etext;
        extern void *__data_start__;
        extern void *__data_end__;
        extern void *__bss_start__;
        extern void *__bss_end__;
        // Copy initialized section
        uint32_t *src = (uint32_t *)&__etext;
        uint32_t *dst = (uint32_t *)&__data_start__;
        uint32_t *limit = (uint32_t *)&__data_end__;
        while (dst < limit)
        {
            *(dst++) = *(src++);
        }
        // zero bass
        dst = (uint32_t *)&__bss_start__;
        limit = (uint32_t *)&__bss_end__;
        while (dst < limit)
        {
            *(dst++) = 0;
        }
        runtime_init();
        main();
    }
}
