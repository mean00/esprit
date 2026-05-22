// clang-format off
#include "ln_rp.h" 
#include "ln_rp_gpio.h"
#include "esprit.h"
#include "lnGPIO.h"
// clang-format on
/**
 * \brief This is a rp2040 specific function to configure the pin in PIO mode
 */
void lnPinModePIO(const lnPin pin, const int instance, const bool pullup)
{
    uint32_t fun;
    if (!instance)
    {
        fun = LN_RP_GPIO_CONTROL_FUNC(PIO0);
    }
    else
    {
        fun = LN_RP_GPIO_CONTROL_FUNC(PIO1);
    }
    uint32_t pad = LN_RP_PADS_DRIVE(12MA) | LN_RP_PADS_INPUT_ENABLE | LN_RP_PADS_SLEW_FAST |
                   LN_RP_PADS_SCHMITT_FAST; // default is input / output
    if (pullup)
        pad |= LN_RP_PADS_PULLUP;
    uint32_t control = LN_RP_GPIO_CONTROL_OE(NORMAL); // 12 mA
    lnGpio->PINS[pin].control = fun + control;
    lnPads->PADS[pin] = pad;
}
/**
    \fn
*/
void lnPinMode(const lnPin pin, const lnGpioMode mode, const uint32_t speedInMhz)
{
    uint32_t fun = LN_RP_GPIO_CONTROL_FUNC(SIO);
    uint32_t pad, control;
    switch (mode)
    {
    case lnSPI_MODE:
        lnPads->PADS[pin] &= 0xff; // RP2350 clear iso
        fun = LN_RP_GPIO_CONTROL_FUNC(SPI);
        lnGpio->PINS[pin].control = fun + LN_RP_GPIO_CONTROL_OE(NORMAL);
        return;
    case lnADC_MODE:
        lnPads->PADS[pin] = LN_RP_PADS_SLEW_FAST | LN_RP_PADS_OUTPUT_DISABLE;                      //
        lnGpio->PINS[pin].control = LN_RP_GPIO_CONTROL_OE(DISABLE) + LN_RP_GPIO_CONTROL_FUNC(ADC); // FIXME NOT SURE
        return;
#ifdef USE_RP2350
    case lnUART_Alt:
        lnPads->PADS[pin] &= 0xff; // RP2350 clear iso
        fun = LN_RP_GPIO_CONTROL_FUNC(UART2);
        lnGpio->PINS[pin].control = fun + LN_RP_GPIO_CONTROL_OE(NORMAL);
        return;
#endif
    case lnUART:
        lnPads->PADS[pin] &= 0xff; // RP2350 clear iso
        fun = LN_RP_GPIO_CONTROL_FUNC(UART);
        lnGpio->PINS[pin].control = fun + LN_RP_GPIO_CONTROL_OE(NORMAL);
        return;
    case lnFLOATING:
        //
        pad = LN_RP_PADS_OUTPUT_DISABLE; // input disabled by default
        control = LN_RP_GPIO_CONTROL_OE_DISABLE;
        break;
    case lnINPUT_FLOATING:
        pad = LN_RP_PADS_INPUT_ENABLE + LN_RP_PADS_OUTPUT_DISABLE; //
        control = LN_RP_GPIO_CONTROL_OE_DISABLE;
        break;
    case lnINPUT_PULLUP:
        pad = LN_RP_PADS_INPUT_ENABLE + LN_RP_PADS_OUTPUT_DISABLE + LN_RP_PADS_PULLUP; //
        control = LN_RP_GPIO_CONTROL_OE_DISABLE;
        break;
    case lnINPUT_PULLDOWN:
        pad = LN_RP_PADS_INPUT_ENABLE + LN_RP_PADS_OUTPUT_DISABLE + LN_RP_PADS_PULLDOWN; //
        control = LN_RP_GPIO_CONTROL_OE_DISABLE;
        break;
    case lnOUTPUT_OPEN_DRAIN:
        lnSio->GPIO_OUT_CLR[0] = 1 << pin; // set output to 0
        pad = LN_RP_PADS_OUTPUT_DISABLE;
        control = LN_RP_GPIO_CONTROL_OE_DISABLE;
        break;
    case lnOUTPUT:
        pad = LN_RP_PADS_DRIVE(12MA);
        control = LN_RP_GPIO_CONTROL_OE(ENABLE); // 12 mA
        break;
    default:
        xAssert(0);
        break;
    }
    lnPads->PADS[pin] = pad | LN_RP_PADS_SLEW_FAST;
    lnGpio->PINS[pin].control = fun + control;
    /*
    lnOUTPUT_OPEN_DRAIN = 4,
    lnALTERNATE_PP,
    lnALTERNATE_OD,
    lnPWM,
    lnADC_MODE,
    lnDAC_MODE
    */
}
/**
 * @brief
 *
 */
void lnOpenDrainClose(const lnPin pin, const bool close)
{
    uint32_t fun = LN_RP_GPIO_CONTROL_FUNC(SIO);
    uint32_t pad, control;

    lnSio->GPIO_OUT_CLR[0] = 1 << pin; // set output to 0
    if (close)                         // passing -> reconfigure as output to 0
    {
        pad = LN_RP_PADS_DRIVE(12MA);
        control = LN_RP_GPIO_CONTROL_OE(ENABLE); // 12 mA
    }
    else // hi z
    {
        pad = LN_RP_PADS_OUTPUT_DISABLE;
        control = LN_RP_GPIO_CONTROL_OE_DISABLE;
    }
    lnPads->PADS[pin] = pad;
    lnGpio->PINS[pin].control = fun + control;
}
/**
    \fn
*/

void lnDigitalWrite(const lnPin pin, bool value)
{
    if (value)
        lnSio->GPIO_OUT_SET[0] = 1 << pin;
    else
        lnSio->GPIO_OUT_CLR[0] = 1 << pin;
}
/**
    \fn
*/

bool lnDigitalRead(const lnPin pin)
{
    return !!(lnSio->GPIO_IN[0] & (1 << pin));
}
/**
    \fn
*/

void lnDigitalToggle(const lnPin pin)
{
    lnSio->GPIO_OUT_XOR[0] = 1 << pin;
}
/**
    \fn
    //----------------- This is bluepill common specific !-------------
*/

volatile uint32_t *lnGetGpioOnRegister(int port) // Bop register for port "port" with port A:0, B:1, ...
{
    return &(lnSio->GPIO_OUT_SET[0]);
}
volatile uint32_t *lnGetGpioOffRegister(int port) // Bop register for port "port" with port A:0, B:1, ...
{
    return &(lnSio->GPIO_OUT_CLR[0]);
}
/**
    \fn
     Direction register for the bit 0..7 of port "port" , A=0, B=1, ...
     //----------------- This is bluepill common specific !-------------
*/

volatile uint32_t *lnGetGpioDirectionRegister(int port)
{
    xAssert(0);
    return NULL;
}
/**
    \fn
    //----------------- This is bluepill common specific !-------------
*/

volatile uint32_t *lnGetGpioValueRegister(int port) // Bit value for LOW bits of port "port"
{
    xAssert(0);
    return NULL;
}
/**
    \fn
    //----------------- This is bluepill common specific !-------------
*/

uint32_t lnReadPort(int port)
{
    xAssert(0);
    return 0;
}
/**
 * @brief Construct a new ln Fast I O::ln Fast I O object
 *
 */
lnFastIO::lnFastIO(lnPin pin)
{
    lnPinMode(pin, lnOUTPUT);
    _on = &(lnSio->GPIO_OUT_SET[0]);
    _off = &(lnSio->GPIO_OUT_CLR[0]);
    _bit = 1 << pin;
}

// EOF
