/*
 *  (C) 2022/2023 MEAN00 fixounet@free.fr
 *  See license file
 *  PFIC is configured with 2 nested level, 1 bit for preemption
 *  That means interrupt priority between 0..7 ignoring preemption
 *
 *
 */

#ifdef USE_CH32v3x_HW_IRQ_STACK
#define HANDLER_DESC(x)                                                                                                \
    extern "C" void x();                                                                                               \
    extern "C" void x##_relay() LN_INTERRUPT_TYPE;
#define HANDLER_DESC_C(y)                                                                                              \
    extern "C" void y();                                                                                               \
    extern "C" void y##_relay() LN_INTERRUPT_TYPE;
extern "C" void unsupported_relay();
#define LOCAL_LN_INTERRUPT_TYPE
#define WCH_HW_STACK CH32_SYSCR_HWSTKEN
#else
#define LOCAL_LN_INTERRUPT_TYPE LN_INTERRUPT_TYPE
#define HANDLER_DESC(x) extern "C" void x() LOCAL_LN_INTERRUPT_TYPE;
#define HANDLER_DESC_C(y) extern "C" void y() LOCAL_LN_INTERRUPT_TYPE;
#define WCH_HW_STACK 0
#endif

#include "ch32v30x_isr_helper.h"
#include "ch32v3x_interrupt_table.h"
#include "esprit.h"
#include "lnIRQ.h"
#include "lnIRQ_riscv_priv_ch32v3x.h"
#include "lnRCU.h"

extern "C"
{

    extern const char _data_begin, _data_end;
    extern const char _bss_begin, _bss_end;
    extern const char _data_lma;

    int main();
    void __libc_init_array(void);

    /**
     * @brief
     *
     */
    ISR_CODE void __attribute__((noreturn)) start_c()
    {
        // Build symbol addresses via %%hi()/%%lo() assembler relocations.
        // No input operands needed — the asm constructs addresses directly.
        // BSS begin/end are reloaded AFTER the data burst, so they can't be corrupted.
        __asm volatile("lui  t0, %%hi(_data_lma)            \n"
                       "addi t0, t0, %%lo(_data_lma)        \n" // t0 = src
                       "lui  t1, %%hi(_data_begin)          \n"
                       "addi t1, t1, %%lo(_data_begin)      \n" // t1 = dst
                       "lui  t2, %%hi(_data_end)            \n"
                       "addi t2, t2, %%lo(_data_end)        \n" // t2 = end
                       "  beq t1, t2, 4f \n"                    // skip if zero-length
                       "  li t4, 32 \n"                         // burst size constant
                       "  sub t3, t2, t1 \n"                    // remaining bytes
                       "  blt t3, t4, 2f \n"                    // skip main loop if <32 bytes remain
                       "1: \n"                                  // main loop: copy 32 bytes via a0..a7
                       "  lw a0, 0(t0) \n"
                       "  lw a1, 4(t0) \n"
                       "  lw a2, 8(t0) \n"
                       "  lw a3, 12(t0) \n"
                       "  lw a4, 16(t0) \n"
                       "  lw a5, 20(t0) \n"
                       "  lw a6, 24(t0) \n"
                       "  lw a7, 28(t0) \n"
                       "  sw a0, 0(t1) \n"
                       "  sw a1, 4(t1) \n"
                       "  sw a2, 8(t1) \n"
                       "  sw a3, 12(t1) \n"
                       "  sw a4, 16(t1) \n"
                       "  sw a5, 20(t1) \n"
                       "  sw a6, 24(t1) \n"
                       "  sw a7, 28(t1) \n"
                       "  addi t0, t0, 32 \n"
                       "  addi t1, t1, 32 \n"
                       "  sub t3, t2, t1 \n" // recompute remaining
                       "  bge t3, t4, 1b \n" // loop while remaining >= 32
                       "2: \n"               // tail: copy 1-7 words one at a time
                       "  beq t1, t2, 4f \n"
                       "3: \n"
                       "  lw t3, 0(t0) \n"
                       "  sw t3, 0(t1) \n"
                       "  addi t0, t0, 4 \n"
                       "  addi t1, t1, 4 \n"
                       "  blt t1, t2, 3b \n" // while dst < end
                       "4: \n"

                       // ---- BSS ZERO: 8 words (32 B) per iteration ----
                       // Reload addresses here — a0..a7 were trashed by the burst copy above.
                       "lui  t0, %%hi(_bss_begin)          \n"
                       "addi t0, t0, %%lo(_bss_begin)      \n" // t0 = begin
                       "lui  t1, %%hi(_bss_end)            \n"
                       "addi t1, t1, %%lo(_bss_end)        \n" // t1 = end
                       "  beq t0, t1, 8f \n"                   // skip if zero-length
                       "  li t4, 32 \n"                        // burst size constant
                       "  sub t2, t1, t0 \n"                   // remaining bytes
                       "  blt t2, t4, 6f \n"                   // skip main loop if <32 bytes remain
                       "5: \n"                                 // main loop: zero 32 bytes
                       "  sw x0, 0(t0) \n"
                       "  sw x0, 4(t0) \n"
                       "  sw x0, 8(t0) \n"
                       "  sw x0, 12(t0) \n"
                       "  sw x0, 16(t0) \n"
                       "  sw x0, 20(t0) \n"
                       "  sw x0, 24(t0) \n"
                       "  sw x0, 28(t0) \n"
                       "  addi t0, t0, 32 \n"
                       "  sub t2, t1, t0 \n" // recompute remaining
                       "  bge t2, t4, 5b \n" // loop while remaining >= 32
                       "6: \n"               // tail: zero 1-7 words
                       "  beq t0, t1, 8f \n"
                       "7: \n"
                       "  sw x0, 0(t0) \n"
                       "  addi t0, t0, 4 \n"
                       "  blt t0, t1, 7b \n" // while cur < end
                       "8: \n"

                       :
                       :                                                                              /* no inputs */
                       : "a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7", "t0", "t1", "t2", "t3", "t4" // clobbers
        );
        __libc_init_array();
        main();
        xAssert(0);
    }
}

// EOF
