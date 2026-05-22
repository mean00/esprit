#include "lnMultiPulse.h"
#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C"
{
#endif

    typedef struct
    {
        void *dummy;
    } ln_multi_pulse_c;

    /**
     * Create a multi-pulse generator on the given pin.
     *
     * @param pin       The output pin (must have a timer mapping).
     * @param tickFqHz  Timer tick frequency in Hz (e.g. 100000 = 10 us ticks).
     * @return          Opaque pointer to the multi-pulse object, or NULL on failure.
     */
    ln_multi_pulse_c *ln_multi_pulse_create(lnPin pin, int tickFqHz);

    /**
     * Destroy a multi-pulse generator.
     */
    void ln_multi_pulse_delete(ln_multi_pulse_c *mp);

    /**
     * Fire a double pulse.
     *
     * @param mp        The multi-pulse generator.
     * @param pulse1Ms  Duration of the first pulse in ms.
     * @param gapMs     Gap between pulses in ms.
     * @param pulse2Ms  Duration of the second pulse in ms.
     *
     * Returns immediately. The hardware runs the sequence autonomously.
     */
    void ln_multi_pulse_fire(ln_multi_pulse_c *mp, int pulse1Ms, int gapMs, int pulse2Ms);

    /**
     * Block until the pulse sequence completes.
     */
    void ln_multi_pulse_wait_done(ln_multi_pulse_c *mp);

    /**
     * Abort a running pulse sequence.
     */
    void ln_multi_pulse_stop(ln_multi_pulse_c *mp);

#ifdef __cplusplus
}
#endif