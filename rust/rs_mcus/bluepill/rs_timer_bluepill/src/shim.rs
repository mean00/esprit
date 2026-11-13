use crate::*;
use crate::registers::*;
use rs_gpio_bluepill::lnPin;
use rs_rcu_bluepill::{Peripheral, enable};
use core::ffi::c_void;
use core::ptr::{read_volatile, write_volatile};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ln_timer_c {
    pub dummy: *mut c_void,
}

#[inline(always)]
pub fn pack_handle(timer: u32, channel: u32) -> *mut ln_timer_c {
    let packed = (timer & TIMER_HANDLE_MASK) | ((channel & TIMER_HANDLE_MASK) << TIMER_HANDLE_CHANNEL_SHIFT);
    packed as *mut ln_timer_c
}

#[inline(always)]
pub fn unpack_handle(handle: *mut ln_timer_c) -> (u32, u32) {
    let packed = handle as usize as u32;
    (packed & TIMER_HANDLE_MASK, (packed >> TIMER_HANDLE_CHANNEL_SHIFT) & TIMER_HANDLE_MASK)
}

pub fn ln_timer_create_from_pin(_pin: lnPin) -> *mut ln_timer_c {
    pack_handle(TIMER_INSTANCE_0, TIMER_CHANNEL_0)
}

pub fn ln_timer_create(timer: u32, channel: u32) -> *mut ln_timer_c {
    let periph = match timer {
        TIMER_INSTANCE_0 => Peripheral::Timer0,
        TIMER_INSTANCE_1 => Peripheral::Timer1,
        TIMER_INSTANCE_2 => Peripheral::Timer2,
        TIMER_INSTANCE_3 => Peripheral::Timer3,
        TIMER_INSTANCE_4 => Peripheral::Timer4,
        _ => Peripheral::Timer0,
    };
    enable(periph);

    pack_handle(timer, channel)
}

pub fn ln_timer_single_shot(handle: *mut ln_timer_c, duration_ms: u32, _up: bool) {
    let (timer_idx, _channel) = unpack_handle(handle);
    let regs = TimerRegisters::ptr(timer_idx);

    unsafe {
        let mut ctl0 = rs_mcu_traits::reg_read!(regs, ctl0);
        ctl0 &= !TIMER_CTL0_CEN;
        rs_mcu_traits::reg_write!(regs, ctl0, ctl0);

        rs_mcu_traits::reg_write!(regs, psc, TIMER_PRESCALER_1MS_72MHZ);
        rs_mcu_traits::reg_write!(regs, car, duration_ms);

        ctl0 |= TIMER_CTL0_OPM;
        ctl0 |= TIMER_CTL0_CEN;
        rs_mcu_traits::reg_write!(regs, ctl0, ctl0);
    }
}

pub fn ln_timer_delete(handle: *mut ln_timer_c) {
    let (timer_idx, _channel) = unpack_handle(handle);
    let regs = TimerRegisters::ptr(timer_idx);
    unsafe {
        let mut ctl0 = rs_mcu_traits::reg_read!(regs, ctl0);
        ctl0 &= !TIMER_CTL0_CEN;
        rs_mcu_traits::reg_write!(regs, ctl0, ctl0);
    }
}

static START_TICKS: [core::sync::atomic::AtomicU16; STOPWATCH_MAX_INSTANCES] =
    [const { core::sync::atomic::AtomicU16::new(0) }; STOPWATCH_MAX_INSTANCES];

#[unsafe(no_mangle)]
pub extern "C" fn ln_hw_stopwatch_create(timer_index: u32) -> *mut c_void {
    let packed = timer_index;
    packed as usize as *mut c_void
}

#[unsafe(no_mangle)]
pub extern "C" fn ln_hw_stopwatch_destroy(_sw: *mut c_void) {}

#[unsafe(no_mangle)]
pub extern "C" fn ln_hw_stopwatch_setup(sw: *mut c_void) {
    let timer_index = sw as usize as u32;
    let periph = match timer_index {
        TIMER_INSTANCE_0 => Peripheral::Timer0,
        TIMER_INSTANCE_1 => Peripheral::Timer1,
        TIMER_INSTANCE_2 => Peripheral::Timer2,
        TIMER_INSTANCE_3 => Peripheral::Timer3,
        TIMER_INSTANCE_4 => Peripheral::Timer4,
        TIMER_INSTANCE_5 => Peripheral::Timer5,
        TIMER_INSTANCE_6 => Peripheral::Timer6,
        _ => Peripheral::Timer0,
    };
    enable(periph);

    let regs = TimerRegisters::ptr(timer_index);
    unsafe {
        rs_mcu_traits::reg_write!(regs, ctl0, 0);
        rs_mcu_traits::reg_write!(regs, psc, 0);
        rs_mcu_traits::reg_write!(regs, car, STOPWATCH_COUNTER_MAX);
        rs_mcu_traits::reg_write!(regs, cnt, 0);
        rs_mcu_traits::reg_write!(regs, ctl0, TIMER_CTL0_CEN);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ln_hw_stopwatch_start(sw: *mut c_void) {
    let timer_index = sw as usize as u32;
    let regs = TimerRegisters::ptr(timer_index);
    let cnt = unsafe { rs_mcu_traits::reg_read!(regs, cnt) as u16 };
    START_TICKS[timer_index as usize % STOPWATCH_MAX_INSTANCES]
        .store(cnt, core::sync::atomic::Ordering::Relaxed);
}

#[unsafe(no_mangle)]
pub extern "C" fn ln_hw_stopwatch_wait(sw: *mut c_void, ticks: u16) {
    let timer_index = sw as usize as u32;
    let start_tick = START_TICKS[timer_index as usize % STOPWATCH_MAX_INSTANCES]
        .load(core::sync::atomic::Ordering::Relaxed);

    let regs = TimerRegisters::ptr(timer_index);
    unsafe {
        while ((rs_mcu_traits::reg_read!(regs, cnt) as u16).wrapping_sub(start_tick)) < ticks {
            core::arch::asm!("nop");
        }
    }
}

// --- C ABI Compatibility Shims for lnDelayTimer ---

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ln_delay_timer_c {
    pub dummy: *mut c_void,
}

static SHIM_DELAY_TIMERS: rs_mcu_traits::SyncCell<[Option<DelayTimer>; TIMER_MAX_INSTANCES]> =
    rs_mcu_traits::SyncCell::new([None, None, None, None, None]);

#[unsafe(no_mangle)]
pub extern "C" fn ln_delay_timer_create(timer: i32, _channel: i32) -> *mut ln_delay_timer_c {
    let idx = timer as usize;
    if idx == 0 || idx >= TIMER_MAX_INSTANCES {
        return core::ptr::null_mut();
    }
    match DelayTimer::new(timer as u32) {
        Ok(dt) => unsafe {
            let slot = &mut (*SHIM_DELAY_TIMERS.as_ptr())[idx];
            *slot = Some(dt);
            slot.as_mut().unwrap() as *mut DelayTimer as *mut ln_delay_timer_c
        },
        Err(_) => core::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ln_delay_timer_delete(timer: *mut ln_delay_timer_c) {
    if !timer.is_null() {
        let dt = unsafe { &mut *(timer as *mut DelayTimer) };
        let idx = dt.timer_idx() as usize;
        if idx < TIMER_MAX_INSTANCES {
            unsafe {
                (*SHIM_DELAY_TIMERS.as_ptr())[idx] = None;
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ln_delay_timer_arm(timer: *mut ln_delay_timer_c, delay_us: i32) {
    if !timer.is_null() {
        let dt = unsafe { &mut *(timer as *mut DelayTimer) };
        dt.arm(delay_us as u32);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ln_delay_timer_set_interrupt(
    timer: *mut ln_delay_timer_c,
    handler: DelayTimerCallback,
    cookie: *mut c_void,
) {
    if !timer.is_null() {
        let dt = unsafe { &mut *(timer as *mut DelayTimer) };
        dt.set_interrupt(handler, cookie);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ln_delay_timer_enable_interrupt(timer: *mut ln_delay_timer_c) {
    if !timer.is_null() {
        let dt = unsafe { &mut *(timer as *mut DelayTimer) };
        dt.enable_interrupt();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ln_delay_timer_disable_interrupt(timer: *mut ln_delay_timer_c) {
    if !timer.is_null() {
        let dt = unsafe { &mut *(timer as *mut DelayTimer) };
        dt.disable_interrupt();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ln_delay_timer_irq(timer: *mut ln_delay_timer_c) {
    if !timer.is_null() {
        let dt = unsafe { &mut *(timer as *mut DelayTimer) };
        dt.on_irq();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ln_delay_timer_interrupt_handler(timer: i32) {
    delay_timer_interrupt_handler(timer as usize);
}
