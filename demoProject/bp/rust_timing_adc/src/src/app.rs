use rust_esprit::AdcTiming;
use rust_esprit::delay_ms;
use rust_esprit::{logger, logger_init};
use rust_esprit::{GpioMode, GpioPin, Pin};
const PIN_IN: Pin = Pin::PA3;

logger_init!();
#[unsafe(no_mangle)]
pub extern "C" fn rnInit() {
    logger!("Setuping up Timing ADC demo...\n");
}

/**
 * \fn rnLoop
 *
 *
 *
 */
struct AdcStruct {}
//
const SAMPLE_SIZE: usize = 32;
#[unsafe(no_mangle)]
pub extern "C" fn rnLoop() {
    logger!("Running Timing ADC demo...\n");

    let mut pin = GpioPin::new(PIN_IN);
    pin.set_mode(GpioMode::Adc);
    let mut output: [u16; SAMPLE_SIZE] = [0; SAMPLE_SIZE];
    let pins: [Pin; 1] = [PIN_IN];
    let mut adc = AdcTiming::<AdcStruct>::new(0);
    adc.set_source(3, 3, 10000, &pins);
    loop {
        adc.multi_read(SAMPLE_SIZE as u32, &mut output);
        for i in 0..SAMPLE_SIZE {
            logger!(" {} : {}\n", i, output[i]);
        }
        delay_ms(1000);
    }
}
// EOF
