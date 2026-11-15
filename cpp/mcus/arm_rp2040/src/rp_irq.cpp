
#include "hardware/irq.h"
#include "hardware/regs/intctrl.h"
#include "esprit.h"
#include "ln_rp.h"
#include "stdio.h"

#include "hardware/structs/scb.h"
#if RP_SDK_VERSION == 1
volatile armv6m_scb_t *scb = ((volatile armv6m_scb_t *)(PPB_BASE + M0PLUS_CPUID_OFFSET));
#elif RP_SDK_VERSION == 2
volatile armv6m_scb_hw_t *scb = ((volatile armv6m_scb_hw_t *)(PPB_BASE + M0PLUS_CPUID_OFFSET));
#else
#error no sdk set
#endif
#define NVIC_IPR0_OFFSET M0PLUS_NVIC_IPR0_OFFSET
#undef ARM_CPU_PREFIXED
#define ARM_CPU_PREFIXED(x) M0PLUS_##x

/*
0 msp
1 reset

    SVC => 11
    PEND => 14
    SYSTICK => 15
*/

extern uint32_t ram_vector_table[48]; // this is the vector table in RAM

void lnSetInterruptHandler(const LnIRQ &irq, lnInterruptHandler *handler)
{
    int real_irq = (int)irq + 16;
    ram_vector_table[real_irq] = (uint32_t)handler;
    __asm__("dmb" : : : "memory");
}
/**
 * @brief
 *
 * @param irq
 */
void lnEnableInterrupt(const LnIRQ &irq)
{
    irq_set_enabled(irq, true);
}
/**
 * @brief
 *
 * @param irq
 */
void lnDisableInterrupt(const LnIRQ &irq)
{
    irq_set_enabled(irq, false);
}
/**
 * @brief
 *
 */
extern "C" void dummy_caller(int code)
{
    volatile uint32_t vtor = scb->vtor;
    __asm__("bkpt #0");
}
#if 1

/*
 * Copyright (c) 2020 Raspberry Pi (Trading) Ltd.
 *
 * SPDX-License-Identifier: BSD-3-Clause
 */
// this block is dervied from the original irq.c code

extern "C" void irq_handler_chain_first_slot(void);
extern "C" void irq_handler_chain_remove_tail(void);
extern "C" void irq_add_tail_to_free_list(struct irq_handler_chain_slot *slot);

extern struct irq_handler_chain_slot
{
    // first 3 half words are executable code (raw vtable handler points to one slot, and inst3 will jump to next
    // in chain (or end of chain handler)
    uint16_t inst1;
    uint16_t inst2;
    uint16_t inst3;
    union {
        // when a handler is removed while executing, it needs an extra instruction, which overwrites
        // the link and the priority; this is ok because no one else is modifying the chain, as
        // the chain is effectively core local, and the user code which might still need this link
        // disable the IRQ in question before updating, which means we aren't executing!
        struct
        {
            int8_t link;
            uint8_t priority;
        };
        uint16_t inst4;
    };
    irq_handler_t handler;
} irq_handler_chain_slots[PICO_MAX_SHARED_IRQ_HANDLERS];
static inline bool is_shared_irq_raw_handler(irq_handler_t raw_handler)
{
    return (uintptr_t)raw_handler - (uintptr_t)irq_handler_chain_slots < sizeof(irq_handler_chain_slots);
}
static inline irq_handler_t *get_vtable(void)
{
    return (irq_handler_t *)scb_hw->vtor;
}

static inline void *add_thumb_bit(void *addr)
{
    return (void *)(((uintptr_t)addr) | 0x1);
}

// GCC produces horrible code for subtraction of pointers here, and it was bugging me
static inline int8_t slot_diff(struct irq_handler_chain_slot *to, struct irq_handler_chain_slot *from)
{
    // static_assert(sizeof(struct irq_handler_chain_slot) == 12, "");
    int32_t result = 0xaaaa;
    // return (to - from);
    // note this implementation has limited range, but is fine for plenty more than -128->127 result
    pico_default_asm("subs %1, %2\n"
                     "adcs %1, %1\n" // * 2 (and + 1 if negative for rounding)
                     "muls %0, %1\n"
                     "lsrs %0, %0, #20\n" : "+l"(result),
                     "+l"(to) : "l"(from) :);
    return (int8_t)result;
}

static struct irq_handler_chain_slot *remove_thumb_bit(void *addr)
{

    // static inline void *remove_thumb_bit(void *addr) {
    return (struct irq_handler_chain_slot *)(((uintptr_t)addr) & (uint)~0x1);
}
static inline void *resolve_branch(uint16_t *inst)
{
    // assert(0x1c == (*inst)>>11u);
    int32_t i_addr = (*inst) << 21u;
    i_addr /= (int32_t)(1u << 21u);
    return inst + 2 + i_addr;
}

static uint16_t make_branch(uint16_t *from, void *to)
{
    uint32_t ui_from = (uint32_t)from;
    uint32_t ui_to = (uint32_t)to;
    int32_t delta = (int32_t)(ui_to - ui_from - 4);
    // assert(delta >= -2048 && delta <= 2046 && !(delta & 1));
    return (uint16_t)(0xe000 | ((delta >> 1) & 0x7ff));
}

static inline int8_t get_slot_index(struct irq_handler_chain_slot *slot)
{
    return slot_diff(slot, irq_handler_chain_slots);
}
#if RP_SDK_VERSION == 1
extern "C" void irq_init_priorities()
{
}
#endif
static int8_t irq_hander_chain_free_slot_head;
/**
 *
 */
void irq_set_mask_enabled(uint32_t mask, bool enabled)
{
    if (enabled)
    {
        // Clear pending before enable
        // (if IRQ is actually asserted, it will immediately re-pend)
        *((io_rw_32 *)(PPB_BASE + M0PLUS_NVIC_ICPR_OFFSET)) = mask;
        *((io_rw_32 *)(PPB_BASE + M0PLUS_NVIC_ISER_OFFSET)) = mask;
    }
    else
    {
        *((io_rw_32 *)(PPB_BASE + M0PLUS_NVIC_ICER_OFFSET)) = mask;
    }
}
/**
 *
 */
extern "C" void irq_set_enabled(uint num, bool enabled)
{
    irq_set_mask_enabled(1u << num, enabled);
}
/**
 *
 */
extern "C" void irq_set_exclusive_handler(uint num, irq_handler_t handler)
{
    lnSetInterruptHandler((LnIRQ)num, handler);
}
void irq_add_tail_to_free_list(struct irq_handler_chain_slot *slot)
{
    irq_handler_t slot_handler = (irq_handler_t)add_thumb_bit(slot);
    // assert(is_shared_irq_raw_handler(slot_handler));

    uint exception = __get_current_exception();
    // assert(exception);
    // spin_lock_t *lock = spin_lock_instance(PICO_SPINLOCK_ID_IRQ);
    // uint32_t save = spin_lock_blocking(lock);
    int8_t slot_index = get_slot_index(slot);
    if (slot_handler == get_vtable()[exception])
    {
        get_vtable()[exception] = 0; //__unhandled_user_irq;
    }
    else
    {
        bool __unused found = false;
        // need to find who points at the slot and update it
        for (uint i = 0; i < count_of(irq_handler_chain_slots); i++)
        {
            if (irq_handler_chain_slots[i].link == slot_index)
            {
                irq_handler_chain_slots[i].link = -1;
                irq_handler_chain_slots[i].inst3 = 0xbd01; // pop {r0, pc}
                found = true;
                break;
            }
        }
        xAssert(found);
    }
    // add slot to free list
    slot->link = irq_hander_chain_free_slot_head;
    irq_hander_chain_free_slot_head = slot_index;
    // spin_unlock(lock, save);
}

extern "C" void irq_add_shared_handler(uint num, irq_handler_t handler, uint8_t order_priority)
{
    check_irq_param(num);
    // hard_assert(irq_hander_chain_free_slot_head >= 0); // we must have a slot
    struct irq_handler_chain_slot *slot = &irq_handler_chain_slots[irq_hander_chain_free_slot_head];
    int8_t slot_index = irq_hander_chain_free_slot_head;
    irq_hander_chain_free_slot_head = slot->link;
    irq_handler_t vtable_handler = get_vtable()[16 + num];
    if (!is_shared_irq_raw_handler(vtable_handler))
    {
        // start new chain
        // hard_assert(vtable_handler == __unhandled_user_irq);
        struct irq_handler_chain_slot slot_data = {
            .inst1 = 0xa100,                                                          // add r1, pc, #0
            .inst2 = make_branch(&slot->inst2, (void *)irq_handler_chain_first_slot), // b irq_handler_chain_first_slot
            .inst3 = 0xbd01,                                                          // pop {r0, pc}
            .link = -1,
            .priority = order_priority,
            .handler = handler};
        *slot = slot_data;
        vtable_handler = (irq_handler_t)add_thumb_bit(slot);
    }
    else
    {
        // assert(!((((uintptr_t)vtable_handler) - ((uintptr_t)irq_handler_chain_slots) - 1)%sizeof(struct
        // irq_handler_chain_slot)));
        struct irq_handler_chain_slot *prev_slot = NULL;
        struct irq_handler_chain_slot *existing_vtable_slot = remove_thumb_bit((void *)vtable_handler);
        struct irq_handler_chain_slot *cur_slot = existing_vtable_slot;
        while (cur_slot->priority > order_priority)
        {
            prev_slot = cur_slot;
            if (cur_slot->link < 0)
                break;
            cur_slot = &irq_handler_chain_slots[cur_slot->link];
        }
        if (prev_slot)
        {
            // insert into chain
            struct irq_handler_chain_slot slot_data = {
                .inst1 = 0x4801, // ldr r0, [pc, #4]
                .inst2 = 0x4780, // blx r0
                .inst3 = prev_slot->link >= 0 ? (uint16_t)make_branch(&slot->inst3, resolve_branch(&prev_slot->inst3))
                                              : // b next_slot
                             (uint16_t)0xbd01,  // pop {r0, pc}
                .link = prev_slot->link,
                .priority = order_priority,
                .handler = handler};
            // update code and data links
            prev_slot->inst3 = make_branch(&prev_slot->inst3, slot), prev_slot->link = slot_index;
            *slot = slot_data;
        }
        else
        {
            // update with new chain head
            struct irq_handler_chain_slot slot_data = {
                .inst1 = 0xa100, // add r1, pc, #0
                .inst2 =
                    make_branch(&slot->inst2, (void *)irq_handler_chain_first_slot), // b irq_handler_chain_first_slot
                .inst3 = make_branch(&slot->inst3, existing_vtable_slot),            // b existing_slot
                .link = get_slot_index(existing_vtable_slot),
                .priority = order_priority,
                .handler = handler};
            *slot = slot_data;
            // fixup previous head slot
            existing_vtable_slot->inst1 = 0x4801; // ldr r0, [pc, #4]
            existing_vtable_slot->inst2 = 0x4780; // blx r0
            vtable_handler = (irq_handler_t)add_thumb_bit(slot);
        }
    }
    // set_raw_irq_handler_and_unlock(num, vtable_handler, save);
    irq_set_exclusive_handler(num, vtable_handler);
}

#ifndef __riscv
static io_rw_32 *nvic_ipr0(void)
{
    return (io_rw_32 *)(PPB_BASE + ARM_CPU_PREFIXED(NVIC_IPR0_OFFSET));
}
#endif

void runtime_init_per_core_irq_priorities(void)
{
#if PICO_DEFAULT_IRQ_PRIORITY != 0
#ifndef __riscv
    // static_assert(!(NUM_IRQS & 3), ""); // this isn't really required - the reg is still 32 bit
    uint32_t prio4 = (PICO_DEFAULT_IRQ_PRIORITY & 0xff) * 0x1010101u;
    io_rw_32 *p = nvic_ipr0();
    for (uint i = 0; i < (NUM_IRQS + 3) / 4; i++)
    {
        *p++ = prio4;
    }
#else
    for (uint i = 0; i < NUM_IRQS; ++i)
    {
        irq_set_priority(i, PICO_DEFAULT_IRQ_PRIORITY);
    }
#endif
#endif
}

#endif
