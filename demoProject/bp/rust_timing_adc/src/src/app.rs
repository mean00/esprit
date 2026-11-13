use rust_esprit::{
    Gpio, Pin, PinMode, ScanAdc, ScanAdcDriver, ScanConfig, delay_ms, logger, logger_init,
};
const PIN_IN: Pin = Pin::PA3;
const SAMPLE_SIZE: usize = 32;
const SAMPLE_RATE_HZ: u32 = 10_000;

logger_init!();
#[unsafe(no_mangle)]
pub extern "C" fn rnInit() {
    logger!("Setuping up Timing ADC demo...\n");
}

/**
 * \fn rnLoop
 *
 * The sense pin is an analog pad (`Gpio`) and the scan is the `ScanAdc` contract:
 * sources as pin numbers, one rate, samples filled by DMA.
 */
#[unsafe(no_mangle)]
pub extern "C" fn rnLoop() {
    logger!("Running Timing ADC demo...\n");

    let mut pin = PIN_IN;
    let _ = Gpio::set_mode(&mut pin, PinMode::Analog);

    let mut output: [u16; SAMPLE_SIZE] = [0; SAMPLE_SIZE];
    let sources = [PIN_IN as u32];
    let mut adc = ScanAdcDriver::new(0);
    let _ = ScanAdc::configure(
        &mut adc,
        &ScanConfig {
            sources: &sources,
            fq_hz: SAMPLE_RATE_HZ,
        },
    );
    loop {
        let filled = ScanAdc::read(&mut adc, &mut output).unwrap_or(0);
        for i in 0..filled.min(SAMPLE_SIZE) {
            logger!(" {} : {}\n", i, output[i]);
        }
        delay_ms(1000);
    }
}
// EOF

