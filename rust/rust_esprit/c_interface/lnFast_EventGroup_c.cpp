
#include "esprit.h"
extern "C"
{
#include "lnFast_EventGroup_c.h"
}
#include "lnFreeRTOS_pp.h"
#include <new> // placement new for the .bss event groups

#define WRAPME(x) ((lnFastEventGroup *)x)

/* The probe creates one event group per USB interface (the GDB and the bridge
 * one today), they live as long as that interface, and they are the *event
 * path* of both tasks: a .bss pool keeps them off the heap, out of reach of
 * fragmentation (malloc_used.md §8.1 B6).  A create beyond the pool still works,
 * it just falls back to the heap. */
#define LN_FAST_EVENT_GROUP_SLOTS 4

alignas(lnFastEventGroup) static uint8_t _evgStorage[LN_FAST_EVENT_GROUP_SLOTS][sizeof(lnFastEventGroup)];
static lnFastEventGroup *_evgPool[LN_FAST_EVENT_GROUP_SLOTS]; // NULL = free slot

lnfast_event_group_c *lnfast_event_group_create()
{
    for (int i = 0; i < LN_FAST_EVENT_GROUP_SLOTS; i++)
    {
        if (!_evgPool[i])
        {
            _evgPool[i] = new (_evgStorage[i]) lnFastEventGroup;
            return (lnfast_event_group_c *)_evgPool[i];
        }
    }
    return (lnfast_event_group_c *)new lnFastEventGroup; // pool exhausted
}
void lnfast_event_group_delete(const lnfast_event_group_c *f)
{
    if (!f)
        return;
    for (int i = 0; i < LN_FAST_EVENT_GROUP_SLOTS; i++)
    {
        if ((const void *)_evgPool[i] == (const void *)f)
        {
            _evgPool[i]->~lnFastEventGroup();
            _evgPool[i] = NULL;
            return;
        }
    }
    delete WRAPME(f); // came from the heap
}

void lnfast_event_group_takeOwnership(const lnfast_event_group_c *inst)
{
    WRAPME(inst)->takeOwnership();
}
void lnfast_event_group_set_events(const lnfast_event_group_c *inst, uint32_t event)
{
    WRAPME(inst)->setEvents(event);
}
uint32_t lnfast_event_group_wait_events(const lnfast_event_group_c *inst, uint32_t mask, int32_t timeout)
{
    return WRAPME(inst)->waitEvents(mask, timeout);
}
uint32_t lnfast_event_group_read_events(const lnfast_event_group_c *inst, uint32_t mask)
{
    return WRAPME(inst)->readEvents(mask);
}
// EOF
