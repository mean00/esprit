use rust_esprit::AdcTiming;
use rust_esprit::delay_ms;
use rust_esprit::{logger, logger_init};
use rust_esprit::{pin, pin_mode};
const PIN_IN: pin = pin::PA3;

logger_init!();
#[unsafe(no_mangle)]
pub extern "C" fn rnInit() {
    logger!("Setuping up Timing ADC demo...\n");

    //pinMode(PS_PIN_VBAT          ,rn::rn_gpio::rnGpioMode::lnADC_MODE);
}

/**
 * \fn rnLoop
 *
 *
 *
 */

const SAMPLE_SIZE: usize = 32;
#[unsafe(no_mangle)]
pub extern "C" fn rnLoop() {
    logger!("Running Timing ADC demo...\n");

    pin_mode(PIN_IN, rust_esprit::GpioMode::lnADC_MODE);
    let mut output: [u16; SAMPLE_SIZE] = [0; SAMPLE_SIZE];
    let pins: [pin; 1] = [PIN_IN];
    let mut adc = AdcTiming::new(0);
    adc.set_source(3, 3, 10000, &pins);
    loop {
        adc.multi_read(SAMPLE_SIZE as i32, &mut output);
        for i in 0..SAMPLE_SIZE {
            logger!(" {} : {}\n", i, output[i]);
        }
        delay_ms(1000);
    }
}
// EOF
