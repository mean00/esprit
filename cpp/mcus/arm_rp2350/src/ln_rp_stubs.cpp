

extern "C" void mutex_init(void *mtx)
{
}
extern "C" void recursive_mutex_init(void *mtx)
{
}
extern "C" void spin_locks_reset(void)
{
}

extern "C" void watchdog_start_tick(unsigned int cycles)
{
}
extern "C" void hw_claim_lock()
{
}
extern "C" void hw_claim_unlock()
{
}
#include "esprit.h"
extern "C"
{
    void *xrom_func_lookup(uint32_t code)
    {
        return NULL;
    }
}
