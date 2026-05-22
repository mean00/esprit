#include "stdbool.h"
#include "stdint.h"
#ifdef __cplusplus
extern "C"
{
#endif

    typedef struct // opaque
    {
        void *dummy;
    } ln_timing_adc_c;

    // enum lnPin : int;
    enum lnPin : int;

    typedef void (*ln_timing_adc_async_callback_t)(void *);

    ln_timing_adc_c *ln_timing_adc_create(int instance);
    bool ln_timing_adc_delete(ln_timing_adc_c *in);
    bool ln_timing_adc_set_source(ln_timing_adc_c *instance, uint32_t timer, uint32_t channel, uint32_t fq,
                                  uint32_t nbPins, const lnPin *pin);
    bool ln_timing_adc_multi_read(ln_timing_adc_c *instance, uint32_t nbSamplePerChannel, uint16_t *output);
    bool ln_timing_adc_async_read(ln_timing_adc_c *instance, uint32_t nbSamplePerChannel, uint16_t *output,
                                  ln_timing_adc_async_callback_t cb, void *ctx);

#ifdef __cplusplus
}
#endif
