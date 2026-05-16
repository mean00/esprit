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

    ln_timing_adc_c *ln_timing_adc_create(int instance);
    bool ln_timing_adc_delete(ln_timing_adc_c *in);
    bool ln_timing_adc_set_source(ln_timing_adc_c *instance, int timer, int channel, int fq, int nbPins,
                                  const lnPin *pin);
    bool ln_timing_adc_multi_read(ln_timing_adc_c *instance, int nbSamplePerChannel, uint16_t *output);

#ifdef __cplusplus
}
#endif
