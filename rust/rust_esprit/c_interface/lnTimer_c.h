#include "lnSystemTime.h"
#include "lnTimer.h"
#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C"
{
#endif

    typedef struct
    {
        void *dummy;
    } ln_timer_c;

    ln_timer_c *ln_timer_create(uint32_t timer, uint32_t channel);
    ln_timer_c *ln_timer_create_from_pin(lnPin pin);
    void ln_timer_delete(ln_timer_c *timer);
    void ln_timer_single_shot(ln_timer_c *timer, uint32_t durationMs, bool up);

#ifdef __cplusplus
}
#endif

extern "C" void lnDelay_C(uint32_t ms);
