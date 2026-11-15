#include "FreeRTOS.h"
#include "pico/runtime_init.h"
#include <stdint.h>

uint8_t ucHeap[configTOTAL_HEAP_SIZE];

extern void runtime_run_initializers(void);
extern void rp_rust_main(void);
extern uint32_t ram_vector_table[];

#define INITIAL_STACK 1024
#define VTOR (*(volatile uint32_t *)0xE000ED08)

int main(void)
{
    runtime_run_initializers();
    extern void (*__init_array_start)(void);
    extern void (*__init_array_end)(void);
    for (void (**p)(void) = &__init_array_start; p < &__init_array_end; ++p)
    {
        (*p)();
    }
    uint32_t *dummyStack = (uint32_t *)pvPortMalloc(INITIAL_STACK * sizeof(uint32_t));
    uint32_t top = (uint32_t)&dummyStack[INITIAL_STACK - 1];
    ram_vector_table[0] = top;
    VTOR = (uint32_t)ram_vector_table;
    __asm__("mov sp,%0\n" ::"r"(top) :);
    rp_rust_main();
    return 0;
}
