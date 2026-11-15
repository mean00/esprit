/**
 * @file ln_rp_pio_priv.h
 * @author mean00
 * @brief Programmable IO
 * @version 0.1
 * @date 2024-06-01
 *
 * @copyright Copyright (c) 2024
 *
 */

#pragma once
#include "esprit.h"
#include "ln_rp_memory_map.h"
#include <cmath>

/**         N
 * @brief describe one pio
 *
 */

typedef struct
{
    uint32_t CLKDIV;
    uint32_t EXECCTRL;
    uint32_t SHIFTCTRL;
    uint32_t ADDR;
    uint32_t INSTR;
    uint32_t PINCTRL;
} LN_RP_PIO_SM;

typedef struct
{
    uint32_t INTE;
    uint32_t INTF;
    uint32_t INTS;
} LN_RP_PIO_IRQx;

struct LN_RP_PIOx
{
    uint32_t PIO_CTRL;              // 0x00:control
    uint32_t PIO_FSTAT;             // Fifo stat
    uint32_t PIO_FDEBUG;            // Fifo debug
    uint32_t PIO_LEVEL;             // Fifo level
    uint32_t PIO_TXF[4];            // 0x10:directT write to FSM tx fifo x
    uint32_t PIO_RXF[4];            // direct write to FSM rx fifo x
    uint32_t PIO_IRQ;               // 0x30:
    uint32_t PIO_IRQ_FORCE;         //
    uint32_t PIO_INPUT_SYNC_BYPASS; //
    uint32_t PIO_DBG_PADOUT;        //
    uint32_t PIO_DBG_PADOE;         //
    uint32_t PIO_DBG_CFGINFO;       //
    uint32_t PIO_INSTR_MEM[32];     //  Direct access to instruction X
    LN_RP_PIO_SM PIO_SM[4];         // 0xc8
    uint32_t PIO_INTR;              // 0x128 Raw interrupt:
    LN_RP_PIO_IRQx PIO_IRQs[2];     // 0x140
};

typedef volatile LN_RP_PIOx LN_RP_PIO;

// CONTROL
#define LN_RP_PIO_CTRL_ENABLE_BIT(x) (1 << x)                //  Each bit enable one SM
#define LN_RP_PIO_CTRL_ENABLE_ALL (0xf)                      //  Each bit enable one SM
#define LN_RP_PIO_CTRL_RESTART_BIT(x) (1 << (4 + x))         //  Each bit enable one SM
#define LN_RP_PIO_CTRL_CLKDIV_RESTART_BIT(x) (1 << (11 + x)) //  Each bit enable one SM

// FSTAT
#define LN_RP_PIO_FSTAT_TX_EMPTY_BIT(x) (1 << (x + 24))
#define LN_RP_PIO_FSTAT_TX_FULL_BIT(x) (1 << (x + 16))
#define LN_RP_PIO_FSTAT_RX_EMPTY_BIT(x) (1 << (x + 8))
#define LN_RP_PIO_FSTAT_RX_FULL_BIT(x) (1 << (x + 0))

// FDEBUG
#define LN_RP_PIO_FDEBUG_RXSTALL_BIT(x) (1 << (x))      // write 1 to clear, 4 bits
#define LN_RP_PIO_FDEBUG_RXUNDER_BIT(x) (1 << (x + 8))  // write 1 to clear, 4 bits
#define LN_RP_PIO_FDEBUG_TXOVER_BIT(x) (1 << (x + 16))  // write 1 to clear, 4 bits
#define LN_RP_PIO_FDEBUG_TXSTALL_BIT(x) (1 << (x + 24)) // write 1 to clear, 4 bits

// FLEVEL
#define LN_RP_PIO_FLEVEL_RX(x) (0x0f << (8 * x))     // write 1 to clear, 4 bits
#define LN_RP_PIO_FLEVEL_TX(x) (0x0f << (4 + 8 * x)) // write 1 to clear, 4 bits

// PIO_DBG_CFGINFO
#define LN_RP_PIO_DBG_CFGINFO_FIFO_DEPTH(x) (x & 0x3f)
#define LN_RP_PIO_DBG_CFGINFO_SM_COUNT(x) ((x >> 8) & 0xf)
#define LN_RP_PIO_DBG_CFGINFO_IMEM_SIZE(x) ((x >> 16) & 0x3f)

// SM CLOCK DIV
#define LN_RP_PIO_SM_CLKDIV(intpart, fracpart) ((intpart << 16) + (fracpart << 8))

// SM EXECTRL
#define LN_RP_PIO_SM_EXECCTRL_STATUS_N_BIT(x) (x & 0xf)
#define LN_RP_PIO_SM_EXECCTRL_STATUS_SEL_BIT_SET(x) ((x & 1) << 4)
#define LN_RP_PIO_SM_EXECCTRL_STATUS_SEL_BIT_GET(x) (!!(x & (1<<4)) )
#define LN_RP_PIO_SM_EXECCTRL_WRAP_BOTTOM_BIT_SET(x) ((x & 0x1f) << 7) // when top is reached , it jumps to bottom
#define LN_RP_PIO_SM_EXECCTRL_WRAP_TOP_BIT_SET(x) ((x & 0x1f) << 12)
#define LN_RP_PIO_SM_EXECCTRL_OUT_STICKY (1 << 17)
#define LN_RP_PIO_SM_EXECCTRL_INLINE_OUT_EN(x) ((x) << 18)
#define LN_RP_PIO_SM_EXECCTRL_OUT_EN_SEL(x) ((x) << 19)
#define LN_RP_PIO_SM_EXECCTRL_JMP_PIN(x) ((x) << 24)
#define LN_RP_PIO_SM_EXECCTRL_SIDE_PINDIR (1 << 29)
#define LN_RP_PIO_SM_EXECCTRL_SIDE_EN (1 << 30)
#define LN_RP_PIO_SM_EXECCTRL_EXEC_STALLE (1 << 31UL)

// SM SHIFTCTRL
#define LN_RP_PIO_SM_SHIFTCTRL_AUTOPUSH (1 << 16)
#define LN_RP_PIO_SM_SHIFTCTRL_AUTOPULL (1 << 17)
#define LN_RP_PIO_SM_SHIFTCTRL_IN_SHIFTDIR (1 << 18)
#define LN_RP_PIO_SM_SHIFTCTRL_OUT_SHIFTDIR (1 << 19)
#define LN_RP_PIO_SM_SHIFTCTRL_PUSH_THRESH(x) (x << 20)
#define LN_RP_PIO_SM_SHIFTCTRL_PULL_THRESH(x) (x << 25)
#define LN_RP_PIO_SM_SHIFTCTRL_FJOIN_TX (1 << 30)
#define LN_RP_PIO_SM_SHIFTCTRL_FJOIN_RX (1 << 31UL)

// SM PINCTRL
#define LN_RP_PIO_SM_PINCTRL_OUT_BASE_BIT(x) ((x) << 0UL)
#define LN_RP_PIO_SM_PINCTRL_SET_BASE_BIT(x) ((x) << 5UL)
#define LN_RP_PIO_SM_PINCTRL_SIDESET_BASE_BIT(x) ((x) << 10UL)
#define LN_RP_PIO_SM_PINCTRL_IN_BASE_BIT(x) ((x) << 15UL)
#define LN_RP_PIO_SM_PINCTRL_OUT_COUNT_BIT(x) ((x) << 20UL)
#define LN_RP_PIO_SM_PINCTRL_SET_COUNT_BIT(x) ((x) << 26UL)
#define LN_RP_PIO_SM_PINCTRL_SIDESET_COUNT_BIT(x) ((x) << 29UL)

// valid for INTE, INTS &INTF
#define LN_RP_PIO_IRQX_RXNEMPTY(x) (1 << (x + 0))
#define LN_RP_PIO_IRQX_TXNFULL(x) (1 << (x + 4))
#define LN_RP_PIO_IRQX_SMx(x) (1 << (x + 8))

//
#define LN_RP_PIO_MAX_PROGRAM_SPACE 32
// EOF
