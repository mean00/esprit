// C interface for lnMultiPulse
extern "C"
{
#include "lnMultiPulse_c.h"
}

#include "lnMultiPulse.h"

/**
 * Create a multi-pulse generator on the given pin.
 */
ln_multi_pulse_c *ln_multi_pulse_create(lnPin pin, int tickFqHz)
{
    lnMultiPulse *mp = new lnMultiPulse(pin, tickFqHz);
    return (ln_multi_pulse_c *)mp;
}

/**
 * Destroy a multi-pulse generator.
 */
void ln_multi_pulse_delete(ln_multi_pulse_c *mp)
{
    delete (lnMultiPulse *)mp;
}

/**
 * Fire a double pulse.
 */
void ln_multi_pulse_fire(ln_multi_pulse_c *mp, int pulse1Ms, int gapMs, int pulse2Ms)
{
    ((lnMultiPulse *)mp)->fire(pulse1Ms, gapMs, pulse2Ms);
}

/**
 * Block until the pulse sequence completes.
 */
void ln_multi_pulse_wait_done(ln_multi_pulse_c *mp)
{
    ((lnMultiPulse *)mp)->waitDone();
}

/**
 * Abort a running pulse sequence.
 */
void ln_multi_pulse_stop(ln_multi_pulse_c *mp)
{
    ((lnMultiPulse *)mp)->stop();
}