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

        __asm volatile(
            // ---- DATA COPY from flash to RAM: 8 words (32 B) per iteration ----
            "  mv t0, %0 \n"     // src
            "  mv t1, %1 \n"     // dst
            "  mv t2, %2 \n"     // end
            "  beq t1, t2, 2f\n" // skip if zero-length
            "1:\n"
            "  lw a0, 0(t0)\n"
            "  lw a1, 4(t0)\n"
            "  lw a2, 8(t0)\n"
            "  lw a3, 12(t0)\n"
            "  lw a4, 16(t0)\n"
            "  lw a5, 20(t0)\n"
            "  lw a6, 24(t0)\n"
            "  lw a7, 28(t0)\n"
            "  sw a0, 0(t1)\n"
            "  sw a1, 4(t1)\n"
            "  sw a2, 8(t1)\n"
            "  sw a3, 12(t1)\n"
            "  sw a4, 16(t1)\n"
            "  sw a5, 20(t1)\n"
            "  sw a6, 24(t1)\n"
            "  sw a7, 28(t1)\n"
            "  addi t0, t0, 32\n"
            "  addi t1, t1, 32\n"
            "  bgt t2, t1, 1b\n"
            "2:\n" // tail: remaining words (1-7)
            "  beq t1, t2, 4f\n"
            "3:\n"
            "  lw t3, 0(t0)\n"
            "  sw t3, 0(t1)\n"
            "  addi t0, t0, 4\n"
            "  addi t1, t1, 4\n"
            "  bgt t2, t1, 3b\n"
            "4:\n"

            // ---- BSS ZERO: 8 words (32 B) per iteration ----
            "  mv t0, %3 \n"     // begin
            "  mv t1, %4 \n"     // end
            "  beq t0, t1, 6f\n" // skip if zero-length
            "5:\n"
            "  sw x0, 0(t0)\n"
            "  sw x0, 4(t0)\n"
            "  sw x0, 8(t0)\n"
            "  sw x0, 12(t0)\n"
            "  sw x0, 16(t0)\n"
            "  sw x0, 20(t0)\n"
            "  sw x0, 24(t0)\n"
            "  sw x0, 28(t0)\n"
            "  addi t0, t0, 32\n"
            "  bgt t1, t0, 5b\n"
            "6:\n" // tail: remaining words (1-7)
            "  beq t0, t1, 8f\n"
            "7:\n"
            "  sw x0, 0(t0)\n"
            "  addi t0, t0, 4\n"
            "  bgt t1, t0, 7b\n"
            "8:\n"

            ::"r"((uint32_t *)&_data_lma), // 0 src
            "r"((uint32_t *)&_data_begin), // 1 data
            "r"((uint32_t *)&_data_end),   // 2 end
            "r"((uint32_t *)&_bss_begin),  // 3 zstart
            "r"((uint32_t *)&_bss_end));   // 4 zend

        __libc_init_array(); // call ctor before jumping in the code
        main();
        xAssert(0);
    }
}

// EOF
