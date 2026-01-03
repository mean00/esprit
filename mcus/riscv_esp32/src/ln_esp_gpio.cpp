
#include "driver/gpio.h"
#include "esprit.h"
#include "lnGPIO_pins.h"
extern "C"
{
#include "soc/gpio_periph.h"
#include "hal/gpio_types.h"
#include "soc/gpio_reg.h"
}
/**
 */
void lnPinMode(const lnPin pin, const lnGpioMode mode, const int speedInMhz)
{
    gpio_config_t io_conf = {.pin_bit_mask = (1ULL << pin),
                             .mode = GPIO_MODE_INPUT,
                             .pull_up_en = GPIO_PULLUP_DISABLE,
                             .pull_down_en = GPIO_PULLDOWN_DISABLE,
                             .intr_type = GPIO_INTR_DISABLE};

    switch (mode)
    {
    case lnFLOATING:
    case lnINPUT_FLOATING:
        break;
    case lnINPUT_PULLUP:
        io_conf.pull_up_en = GPIO_PULLUP_ENABLE;
        break;
    case lnINPUT_PULLDOWN:
        io_conf.pull_down_en = GPIO_PULLDOWN_ENABLE;
        break;
    case lnOUTPUT:
        io_conf.mode = GPIO_MODE_OUTPUT;
        break;
    case lnOUTPUT_OPEN_DRAIN:
        io_conf.mode = GPIO_MODE_OUTPUT_OD;
        io_conf.pull_up_en = GPIO_PULLUP_ENABLE; // no sure ???
        break;
    case lnALTERNATE_PP:
    case lnALTERNATE_OD:
    case lnPWM:
    case lnADC_MODE:
    case lnDAC_MODE:
    case lnUART:
    case lnSPI_MODE:
    case lnUART_Alt:
    default:
        Logger("Ignoring pin configuration, assuming you have done it elsewhere");
        break;
    }
    gpio_config(&io_conf);
}
/**
 *
 *
 */
void lnDigitalWrite(const lnPin pin, bool value)
{
    gpio_set_level((gpio_num_t)pin, value);
}
/**
 *
 *
 */
bool lnDigitalRead(const lnPin pin)
{
    return gpio_get_level((gpio_num_t)pin);
}
/**
 *
 *
 */
void lnDigitalToggle(const lnPin pin)
{
    gpio_set_level((gpio_num_t)pin, !gpio_get_level((gpio_num_t)pin));
}
/**
 *
 */
void lnOpenDrainClose(const lnPin pin, const bool close) // if true, the open drain is passing, else it is hiz
{
    xAssert(0);
}
/**
 * Assume we'll only use lower pins (0..31)
 */
 volatile uint32_t* const GPIO_SET_REG = (volatile uint32_t*)GPIO_OUT_W1TS_REG;
 volatile uint32_t* const GPIO_CLR_REG = (volatile uint32_t*)GPIO_OUT_W1TC_REG;
lnFastIO::lnFastIO(lnPin p)
{
     lnPinMode(p, lnOUTPUT);
     _on = GPIO_SET_REG;
     _off = GPIO_CLR_REG;
     _bit =  1<<(uint32_t )p;
}

//  EOF
