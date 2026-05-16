// this is not thread safe, but it should not matter much
// the returned object ARE thread sage, it's just the creation/deletion that is not
extern "C"
{
#include "lnTimer_c.h"
}

#include "lnTimer.h"

/**
 *
 * @param timer
 * @param channel
 * @return
 */
ln_timer_c *ln_timer_create(uint32_t timer, uint32_t channel)
{
    lnTimer *t = new lnTimer(timer, channel);
    return (ln_timer_c *)t;
}

/**
 *
 * @param pin
 * @return
 */
ln_timer_c *ln_timer_create_from_pin(lnPin pin)
{
    lnTimer *t = new lnTimer(pin);
    return (ln_timer_c *)t;
}

/**
 *
 * @param timer
 */
void ln_timer_delete(ln_timer_c *timer)
{
    delete (lnTimer *)timer;
}

/**
 *
 * @param timer
 * @param durationMs
 * @param down
 */
void ln_timer_single_shot(ln_timer_c *timer, uint32_t durationMs, bool up)
{
    ((lnTimer *)timer)->singleShot(durationMs, up);
}
// EOF
