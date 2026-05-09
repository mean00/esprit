#![allow(dead_code)]
use crate::rn_gpio::rnPin; //,rnGpioMode};
use crate::rn_timing_adc_c;
pub use crate::rn_exti::rnEdge;
//use alloc::boxed::Box;
use crate::rn_timing_adc_c as adc;

pub struct rnTimingAdc {
    ln      : *mut adc::ln_timing_adc_c,
    nb_pins : usize,
             //
}
//
//
impl rnTimingAdc {
    //
    //
    pub fn new(instance: usize) -> Self {
        unsafe {
            let t: rnTimingAdc = rnTimingAdc {
                ln:adc:: ln_timing_adc_create(instance as i32),
                nb_pins : 0,
            };
            t
        }
    }
    //
    //
    pub fn set_source(
        &mut self,
        timer: usize,
        channel: usize,
        frequency: usize,
        pins: &[rnPin],
    ) -> bool {
        self.nb_pins = pins.len();
        unsafe {
            adc::ln_timing_adc_set_source(
               self.ln,
               timer      as cty::c_int,
               channel    as cty::c_int ,
               frequency  as cty::c_int ,
               pins.len() as cty::c_int ,
               pins.as_ptr() as *const rn_timing_adc_c::lnPin)
        }
    }
    //
    //
    pub fn multi_read(&mut self, nbSamplePerChannel: cty::c_int, output: &mut [u16]) -> bool {
        unsafe {
          if output.len() < (nbSamplePerChannel as usize)*self.nb_pins {
              panic!("oops");
          }
          adc::ln_timing_adc_multi_read(
               self.ln,
               nbSamplePerChannel,
               output.as_mut_ptr())
        }
    }
}
// EOF
