

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
    void *rom_func_lookup(uint32_t code)
    {
        return NULL;
    }
}
extern "C" void panic_unsupported(void) { do_assert("panic_unsupported"); }

extern "C" {
bool irq_is_enabled(uint num) {
    return (num < 32) && ((*(volatile uint32_t *)(0xe000e100)) & (1 << num));
}
void irq_set_priority(uint num, uint8_t hardware_priority) {
    volatile uint8_t *nvic_ipr = (volatile uint8_t *)(0xe000e400);
    nvic_ipr[num] = hardware_priority;
}
}

extern "C" {
void hw_claim_or_assert(uint8_t *bits, uint bit_index, const char *message) {
    bits[bit_index >> 3u] |= (uint8_t)(1u << (bit_index & 7u));
}
}
