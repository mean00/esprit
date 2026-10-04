#include "stdbool.h"
#include "stdint.h"
#ifdef __cplusplus
extern "C"
{
#endif

    // RP2040 / RP2350 only: thin bridge between the C++ `lnSimpleADC` driver
    // (mcus/arm_rp2xxx/src/ln_rp_adc.cpp) and the Rust backend rs_adc_rp2xxx.
    //
    // The `ln_simple_adc_*` C ABI itself belongs to the Rust backends
    // (rs_adc_bluepill on GD32/CH32, rs_adc_rp2xxx on RP2xxx), so the shims
    // exposed here use a distinct `ln_rp_` prefix.

    void *ln_rp_simple_adc_create(int instance, uint32_t pin);
    void ln_rp_simple_adc_destroy(void *adc);
    uint32_t ln_rp_simple_adc_read(void *adc, uint32_t averaging);

#ifdef __cplusplus
}
#endif
