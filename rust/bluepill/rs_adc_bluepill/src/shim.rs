use crate::*;
use core::ffi::c_void;
use rs_gpio_bluepill::lnPin;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct ln_timing_adc_c {
    pub dummy: *mut c_void,
}

pub type ln_timing_adc_async_callback_t =
    ::core::option::Option<unsafe extern "C" fn(arg1: *mut c_void)>;

fn pack_timing_handle(instance: u32) -> *mut ln_timing_adc_c {
    instance as usize as *mut ln_timing_adc_c
}

fn unpack_timing_handle(handle: *mut ln_timing_adc_c) -> u32 {
    handle as usize as u32
}

static mut TIMING_ADC_INSTANCES: [Option<TimingAdc>; 2] = [None, None];

#[unsafe(no_mangle)]
pub extern "C" fn ln_timing_adc_create(instance: i32) -> *mut ln_timing_adc_c {
    let inst_idx = instance as usize;
    if inst_idx < 2 {
        unsafe {
            TIMING_ADC_INSTANCES[inst_idx] = Some(TimingAdc::new(instance as u32));
        }
    }
    pack_timing_handle(instance as u32)
}

#[unsafe(no_mangle)]
pub extern "C" fn ln_timing_adc_delete(handle: *mut ln_timing_adc_c) -> bool {
    let inst_idx = unpack_timing_handle(handle) as usize;
    if inst_idx < 2 {
        unsafe {
            TIMING_ADC_INSTANCES[inst_idx] = None;
        }
    }
    true
}

#[unsafe(no_mangle)]
pub extern "C" fn ln_timing_adc_set_source(
    handle: *mut ln_timing_adc_c,
    timer: u32,
    channel: u32,
    fq: u32,
    nb_pins: u32,
    pin: *const lnPin,
) -> bool {
    let instance = unpack_timing_handle(handle) as usize;
    if instance < 2 {
        unsafe {
            if let Some(ref mut adc) = TIMING_ADC_INSTANCES[instance] {
                let pins = core::slice::from_raw_parts(pin, nb_pins as usize);
                return adc.set_source(timer, channel, fq, pins);
            }
        }
    }
    false
}

#[unsafe(no_mangle)]
pub extern "C" fn ln_timing_adc_multi_read(
    handle: *mut ln_timing_adc_c,
    nb_sample_per_channel: u32,
    output: *mut u16,
) -> bool {
    let instance = unpack_timing_handle(handle) as usize;
    if instance < 2 {
        unsafe {
            if let Some(ref mut adc) = TIMING_ADC_INSTANCES[instance] {
                let nb_pins = adc.configured_pins_count();
                let total_samples = (nb_sample_per_channel * nb_pins) as usize;
                let out_slice = core::slice::from_raw_parts_mut(output, total_samples);
                return adc.multi_read(nb_sample_per_channel, out_slice);
            }
        }
    }
    false
}

#[unsafe(no_mangle)]
pub extern "C" fn ln_timing_adc_async_read(
    handle: *mut ln_timing_adc_c,
    nb_sample_per_channel: u32,
    output: *mut u16,
    cb: ln_timing_adc_async_callback_t,
    ctx: *mut c_void,
) -> bool {
    let instance = unpack_timing_handle(handle) as usize;
    if instance < 2 {
        unsafe {
            if let Some(ref mut adc) = TIMING_ADC_INSTANCES[instance] {
                let nb_pins = adc.configured_pins_count();
                let total_samples = (nb_sample_per_channel * nb_pins) as usize;
                let out_slice = core::slice::from_raw_parts_mut(output, total_samples);
                return adc.async_read(nb_sample_per_channel, out_slice, cb, ctx);
            }
        }
    }
    false
}

#[unsafe(no_mangle)]
pub extern "C" fn ln_simple_adc_create(instance: u32, pin: u32) -> *mut c_void {
    let adc = SimpleAdc::new(instance, pin);
    let packed = (adc.instance() & 0xFFFF) | ((adc.pin() & 0xFFFF) << 16);
    packed as usize as *mut c_void
}

#[unsafe(no_mangle)]
pub extern "C" fn ln_simple_adc_destroy(_adc: *mut c_void) {}

#[unsafe(no_mangle)]
pub extern "C" fn ln_simple_adc_set_smpt(adc: *mut c_void, smpt: u32) {
    let packed = adc as usize as u32;
    let instance = packed & 0xFFFF;
    let pin = (packed >> 16) & 0xFFFF;
    let mut a = SimpleAdc::new(instance, pin);
    a.set_smpt(smpt);
}

#[unsafe(no_mangle)]
pub extern "C" fn ln_simple_adc_read(adc: *mut c_void) -> i32 {
    let packed = adc as usize as u32;
    let instance = packed & 0xFFFF;
    let pin = (packed >> 16) & 0xFFFF;
    let a = SimpleAdc::new(instance, pin);
    a.read()
}
