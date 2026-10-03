#![allow(unused_variables, unused_imports, unused_parens)]
#![allow(non_camel_case_types, non_snake_case, dead_code)]

#![no_std]

pub mod registers;

use registers::AdcRegisters;
use rs_gpio_bluepill::lnPin;
use rs_dma_bluepill::{DmaChannel, DmaEngine};
use rs_rcu_bluepill::{Peripheral, enable};
use core::ptr::{read_volatile, write_volatile};
use core::ffi::c_void;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct ln_timing_adc_c {
    pub dummy: *mut c_void,
}

pub type ln_timing_adc_async_callback_t =
    ::core::option::Option<unsafe extern "C" fn(arg1: *mut c_void)>;

// For zero-allocation #![no_std], we pack data in the pointer
fn pack_handle(instance: u32) -> *mut ln_timing_adc_c {
    instance as usize as *mut ln_timing_adc_c
}

fn pin_to_adc_channel(pin: lnPin) -> u32 {
    let p = pin as u32;
    if p < 8 {
        p // PA0-PA7 -> CH0-CH7
    } else if p >= 16 && p <= 17 {
        p - 16 + 8 // PB0-PB1 -> CH8-CH9
    } else if p >= 32 && p <= 37 {
        p - 32 + 10 // PC0-PC5 -> CH10-CH15
    } else {
        0 // Default fallback
    }
}

fn unpack_handle(handle: *mut ln_timing_adc_c) -> u32 {
    handle as usize as u32
}

static mut ASYNC_CB: Option<unsafe extern "C" fn(*mut c_void)> = None;
static mut ASYNC_CTX: *mut c_void = core::ptr::null_mut();

#[unsafe(no_mangle)]
pub extern "C" fn ln_timing_adc_create(instance: i32) -> *mut ln_timing_adc_c {
    if instance == 0 {
        enable(Peripheral::Adc0);
    } else {
        enable(Peripheral::Adc1);
    }
    pack_handle(instance as u32)
}

#[unsafe(no_mangle)]
pub extern "C" fn ln_timing_adc_delete(_in: *mut ln_timing_adc_c) -> bool {
    true
}

#[unsafe(no_mangle)]
pub extern "C" fn ln_timing_adc_set_source(
    handle: *mut ln_timing_adc_c,
    _timer: u32,
    _channel: u32,
    _fq: u32,
    nb_pins: u32,
    pin: *const lnPin,
) -> bool {
    let instance = unpack_handle(handle);
    let regs = AdcRegisters::ptr(instance);
    
    // We strictly use DMA for Timing ADC, which means ADC0 only (instance 0)
    if instance != 0 { return false; }

    unsafe {
        // Setup Scan Mode
        let mut ctl0 = read_volatile(&mut (*regs).ctl0);
        ctl0 |= 1 << 8; // SCAN mode
        write_volatile(&mut (*regs).ctl0, ctl0);

        // Configure sequencer length
        let rsq0 = (nb_pins.saturating_sub(1) & 0x0F) << 20;
        write_volatile(&mut (*regs).rsq0, rsq0);
        
        // Map pins to RSQ2 (Assuming first 5 pins fit in RSQ2)
        let mut rsq2 = 0;
        for i in 0..nb_pins {
            let p = *pin.add(i as usize);
            let ch = pin_to_adc_channel(p);
            rsq2 |= ch << (5 * i);
        }
        write_volatile(&mut (*regs).rsq2, rsq2);
        
        // Setup External Trigger
        // For Timer 2 TRGO on STM32/GD32 ADC1, ETSRC is usually 0b011
        // We will default to a software trigger or specific timer trigger 
        // to simplify the example without fully mapping the entire timer matrix.
        let mut ctl1 = read_volatile(&mut (*regs).ctl1);
        ctl1 |= 0b111 << 17; // EXTSEL = SWSTART (for simple simulation/fallback)
        ctl1 |= 1 << 20; // EXTTRIG = Enabled
        write_volatile(&mut (*regs).ctl1, ctl1);
        
        // Enable ADC
        ctl1 |= 1; // ADON
        write_volatile(&mut (*regs).ctl1, ctl1);
    }

    true
}

#[unsafe(no_mangle)]
pub extern "C" fn ln_timing_adc_multi_read(
    handle: *mut ln_timing_adc_c,
    nb_sample_per_channel: u32,
    output: *mut u16,
) -> bool {
    let instance = unpack_handle(handle);
    let regs = AdcRegisters::ptr(instance);
    
    // Read total pins from RSQ0
    let nb_pins = unsafe { ((read_volatile(&mut (*regs).rsq0) >> 20) & 0x0F) + 1 };
    let total_samples = nb_sample_per_channel * nb_pins;
    
    // NATIVE DMA CONFIGURATION
    let mut dma = DmaChannel::new(DmaEngine::Dma0, 0); // DMA0 Channel 0 is ADC0
    
    unsafe {
        // Prepare ADC for DMA
        let mut ctl1 = read_volatile(&mut (*regs).ctl1);
        ctl1 |= 1 << 8; // DMA mode enabled
        write_volatile(&mut (*regs).ctl1, ctl1);
        
        // Configure DMA
        dma.begin_tx_transfer(
            &(*regs).rdata as *const _ as u32,
            output as u32,
            total_samples,
            true, // 16-bit PSIZE
            true  // 16-bit MSIZE
        );
        
        // Trigger conversion (SWSTART for fallback)
        ctl1 |= 1 << 22; // SWSTART
        write_volatile(&mut (*regs).ctl1, ctl1);
        
        // Blocking wait
        while !dma.is_transfer_complete() {}
        
        dma.end_transfer();
        
        // Cleanup ADC DMA flag
        ctl1 &= !(1 << 8);
        write_volatile(&mut (*regs).ctl1, ctl1);
    }
    
    true
}

#[unsafe(no_mangle)]
pub extern "C" fn ln_timing_adc_async_read(
    handle: *mut ln_timing_adc_c,
    nb_sample_per_channel: u32,
    output: *mut u16,
    cb: ln_timing_adc_async_callback_t,
    ctx: *mut c_void,
) -> bool {
    // For now, run synchronously and fire callback
    // (A real async would require setting up the DMA TC interrupt vector)
    ln_timing_adc_multi_read(handle, nb_sample_per_channel, output);
    if let Some(callback) = cb {
        unsafe { callback(ctx); }
    }
    true
}
