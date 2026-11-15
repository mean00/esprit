#include "esprit.h"
extern "C"
{
#include "hardware/gpio.h"
    void gpio_set_function(unsigned int pin, gpio_function_t pio)
    {
        switch (pio)
        {
        case GPIO_FUNC_PIO0:
            lnPinModePIO((lnPin)pin, 0);
            break;
        case GPIO_FUNC_PIO1:
            lnPinModePIO((lnPin)pin, 1);
            break;
        default:
            xAssert(0);
            break;
        }
    }
}
