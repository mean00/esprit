/**
 * Very very basic ADC
 */
// clang-format off
#include "ln_rp.h" 
#include "hardware/gpio.h"
#include "hardware/adc.h"
#include "lnADC.h"
#include "lnGPIO.h"
#include "ln_rp_adc.h"
// clang-format on
static bool adc_inited = false;

/**
 *
 * @param instance
 */
lnSimpleADC::lnSimpleADC(int instance, lnPin pin)
{
    if (!adc_inited)
    {
        adc_inited = true;
        // init once if/when needed, for example clock
    }
    xAssert(instance == 0);
    xAssert(pin >= GPIO26 && pin <= GPIO29);
    setPin(pin);
}
/**
 *
 */
lnSimpleADC::~lnSimpleADC()
{
}
/**
 *
 */
lnPin lnSimpleADC::pin()
{
    return _pin;
}
/**
 *
 */
bool lnSimpleADC::setPin(lnPin pin)
{
    _pin = pin;
    lnPinMode(_pin, lnADC_MODE);
    return true;
}
/**
 *
 */
uint32_t lnSimpleADC::simpleRead(uint32_t averaging)
{
    setup();
    uint32_t sum = 0, count = averaging;
    while (count--)
    {
        lnAdc->CS |= LN_RP_ADC_CS_START_ONCE;
        while (true)
        {
            if (lnAdc->CS & LN_RP_ADC_CS_READY)
                break;
        }
        sum += lnAdc->RESULT & LN_RP_ADC_FIFO_VALUE_MASK;
    }
    shutdown();
    return sum / averaging;
}
/**
 *
 */
void lnSimpleADC::setup()
{
    lnAdc->CS = LN_RP_ADC_CS_ENABLE;
    while (!(lnAdc->CS & LN_RP_ADC_CS_READY))
    {
        __asm__("nop");
    }
    lnAdc->CS |= LN_RP_ADC_CS_AINSEL((_pin - GPIO26));
    lnAdc->DIV = LN_RP_ADC_DIV_FRAC(100) + LN_RP_ADC_DIV_INT(100); // randome value
}
/**
 */
void lnSimpleADC::shutdown()
{
    lnAdc->CS &= ~LN_RP_ADC_CS_ENABLE;
}
// -- EOF --
