#include "esprit.h"
#include "ln_rp_memory_map.h"
/**
 */
struct LN_RP_GPIO_PIN
{
    volatile uint32_t status; // this is RO
    volatile uint32_t control;
};
/**
 */
struct LN_RP_GPIOx
{
    LN_RP_GPIO_PIN PINS[30]; // 00..29
    uint32_t INTR[4];        // 0xf0
    uint32_t CPU0_INTE[4];   // 0x100
    uint32_t CPU0_INTF[4];   // 0x110
    uint32_t CPU0_INTS[4];   // 0x120

    uint32_t CPU1_INTE[4]; // 0x130
    uint32_t CPU1_INTF[4]; // 0x140
    uint32_t CPU1_INTS[4]; // 0x150

    uint32_t DORMANT_INTE[4]; // 0x160
    uint32_t DORMANT_INTF[4]; // 0x170
    uint32_t DORMANT_INTS[4]; // 0x180
};

typedef volatile LN_RP_GPIOx LN_RP_GPIO;

// Status
#define LN_RP_GPIO_STATUS_RAW_OUTPUT (1 << 8)     // raw output
#define LN_RP_GPIO_STATUS_OUTPUT (1 << 9)         // output after override
#define LN_RP_GPIO_STATUS_RAW_OE (1 << 12)        // raw output enable
#define LN_RP_GPIO_STATUS_OE (1 << 13)            // output enable after override
#define LN_RP_GPIO_STATUS_RAW_INPUT (1 << 17)     // input before override
#define LN_RP_GPIO_STATUS_INPUT (1 << 19)         // input on pad
#define LN_RP_GPIO_STATUS_RAW_INTERRUPT (1 << 24) // interrupt before override
#define LN_RP_GPIO_STATUS_INTERRUPT (1 << 26)     // interrupt after override

// Control Function
#define LN_RP_GPIO_CONTROL_FUNC_XIP (0)
#define LN_RP_GPIO_CONTROL_FUNC_SPI (1)
#define LN_RP_GPIO_CONTROL_FUNC_UART (2)
#define LN_RP_GPIO_CONTROL_FUNC_I2C (3)
#define LN_RP_GPIO_CONTROL_FUNC_PWM (4)
#define LN_RP_GPIO_CONTROL_FUNC_SIO (5)
#define LN_RP_GPIO_CONTROL_FUNC_PIO0 (6)
#define LN_RP_GPIO_CONTROL_FUNC_PIO1 (7)
#define LN_RP_GPIO_CONTROL_FUNC_USB (9)
#define LN_RP_GPIO_CONTROL_FUNC_UART2 (0)
#define LN_RP_GPIO_CONTROL_FUNC_ADC (0x1f)

#define LN_RP_GPIO_CONTROL_FUNC(x) (LN_RP_GPIO_CONTROL_FUNC_##x << 0) //
#define LN_RP_GPIO_CONTROL_FUNC_MASK (0x1F)                           //

// Control-OUTPUT

#define LN_RP_GPIO_CONTROL_OUTPUT_NORMAL 0
#define LN_RP_GPIO_CONTROL_OUTPUT_INVERTED 1
#define LN_RP_GPIO_CONTROL_OUTPUT_LOW 2
#define LN_RP_GPIO_CONTROL_OUTPUT_HIGH 3

#define LN_RP_GPIO_CONTROL_OUTPUT(x) (LN_RP_GPIO_CONTROL_OUTPUT_##x << 8) //
#define LN_RP_GPIO_CONTROL_OUTPUT_MASK (3 << 8)                           //

// Control-OUTPUT ENABLE
#define LN_RP_GPIO_CONTROL_OE_NORMAL 0
#define LN_RP_GPIO_CONTROL_OE_INVERTED 1
#define LN_RP_GPIO_CONTROL_OE_DISABLE 2
#define LN_RP_GPIO_CONTROL_OE_ENABLE 3

#define LN_RP_GPIO_CONTROL_OE(x) (LN_RP_GPIO_CONTROL_OE_##x << 12) //
#define LN_RP_GPIO_CONTROL_OE_MASK (3 << 12)                       //

// PADs 0x4001c000
struct LN_RP_PADSx
{
    uint32_t VOLTAGE_SELECT; // 0x00 0=3.3v, 1=1.8v
    uint32_t PADS[30];       // 0xxx
    uint32_t SWCLK;          // 0x7c
    uint32_t SWIO;           // 0x80
};

#define LN_RP_PADS_SLEW_FAST (1 << 0)
#define LN_RP_PADS_SLEW_SLOW (0 << 0)
#define LN_RP_PADS_SLEW_MASK (1 << 0)

#define LN_RP_PADS_SCHMITT_FAST (1 << 0)
#define LN_RP_PADS_SCHMITT_SLOW (0 << 0)
#define LN_RP_PADS_SCHMITT_MASK (1 << 0)

#define LN_RP_PADS_PULLUP (2 << 2)
#define LN_RP_PADS_PULLDOWN (1 << 2)
#define LN_RP_PADS_NO_PULL (0 << 2)
#define LN_RP_PADS_PULLMASK (3 << 2)

#define LN_RP_PADS_INPUT_ENABLE (1 << 6)
#define LN_RP_PADS_OUTPUT_DISABLE (1 << 7)

#define LN_RP_PADS_DRIVE_2MA 0
#define LN_RP_PADS_DRIVE_4MA 1
#define LN_RP_PADS_DRIVE_8MA 2
#define LN_RP_PADS_DRIVE_12MA 3
#define LN_RP_PADS_DRIVE(x) (LN_RP_PADS_DRIVE_##x << 4)

typedef volatile LN_RP_PADSx LN_RP_PADS;

// LN_SIO_BASE_ADR 0xd000000
struct LN_RP_SIOx
{
    uint32_t CPUID;           // 0x00
    uint32_t GPIO_IN[1];      // 0x04
    uint32_t GPIO_HI_IN;      // 0x08
    uint32_t GPIO_HI_IN2;     // 0x0C
    uint32_t GPIO_OUT;        // 0x10
    uint32_t GPIO_OUT_SET[1]; // 0x14
    uint32_t GPIO_OUT_CLR[1]; // 0x18
    uint32_t GPIO_OUT_XOR[1]; // 0x1C
    uint32_t GPIO_OE;         // 0x20
    uint32_t GPIO_OE_SET;     // 0x24
    uint32_t GPIO_OE_CLR;     // 0x28
    uint32_t GPIO_OE_XOR;     // 0x2C
    uint32_t GPIO_HI_OUT;     // 0x30
    uint32_t GPIO_HI_OUT_SET; // 0x34
    uint32_t GPIO_HI_OUT_CLR; // 0x38
    uint32_t GPIO_HI_OUT_XOR; // 0x3c
    uint32_t GPIO_HI_OE;      // 0x40
    uint32_t GPIO_HI_OE_SET;  // 0x44
    uint32_t GPIO_HI_OE_CLR;  // 0x48
    uint32_t GPIO_HI_OE_XOR;  // 0x4C
};

typedef volatile LN_RP_SIOx LN_RP_SIO;

LN_RP_GPIO *lnGpio = (LN_RP_GPIO *)LN_IO_BANK0_BASE_ADR;
LN_RP_SIO *lnSio = (LN_RP_SIO *)LN_SIO_BASE_ADR;
LN_RP_PADS *lnPads = (LN_RP_PADS *)LN_PADS_BANK0_BASE_ADR;

// EOF
