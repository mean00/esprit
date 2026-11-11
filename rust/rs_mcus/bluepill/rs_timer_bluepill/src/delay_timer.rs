//! Hardware Microsecond Delay Timer (`DelayTimer`) with interrupt callback.
//!
//! Follows Esprit Rust Rule 1 (No Magic Numbers), Rule 2 (Platform Specific bluepill),
//! and Rule 6 (Idiomatic Rust & C++ Parity).
//!
//! Replicates `lnDelayTimer` from `esprit/mcus/common_bluepill/lnDelayTimer.cpp`.

use crate::registers::*;
use crate::timer::Timer;
use core::ffi::c_void;
use core::ptr::{read_volatile, write_volatile};
use rs_mcu_traits::handler::HandlerTable;
use rs_mcu_traits::timer;
use rs_mcu_traits::Result as HalResult;
use rs_rcu_bluepill::{get_clock, Peripheral};

/// C-style timer interrupt handler callback function pointer.
pub type DelayTimerCallback = Option<unsafe extern "C" fn(cookie: *mut c_void)>;

/// One handler slot per timer channel (the same channels `rs_timer_bluepill` uses).
static DELAY_HANDLERS: HandlerTable<dyn timer::DelayHandler, TIMER_MAX_INSTANCES> =
    HandlerTable::new();

/// Adapter from the family's C-style callback to the typed handler of the trait.
///
/// The cookie is the timer index, which is how the trampoline finds the handler
/// the caller installed.
unsafe extern "C" fn delay_trampoline(cookie: *mut c_void) {
    // SAFETY: the handler was installed before the interrupt was enabled, and the
    // interrupt is the only reader.
    if let Some(handler) = unsafe { DELAY_HANDLERS.isr_get(cookie as usize) } {
        handler.on_elapsed();
    }
}

unsafe extern "C" {
    #[link_name = "\u{1}_Z17lnEnableInterruptRK5LnIRQ"]
    fn ln_enable_interrupt(irq: *const u32);

    #[link_name = "\u{1}_Z18lnDisableInterruptRK5LnIRQ"]
    fn ln_disable_interrupt(irq: *const u32);
}

// Global dispatch table matching C++ `timerInstances[5]`
static mut DELAY_TIMER_INSTANCES: [Option<*mut DelayTimer>; TIMER_MAX_INSTANCES] = [None; TIMER_MAX_INSTANCES];

/// Hardware microsecond delay alarm timer with update interrupt callback.
pub struct DelayTimer {
    timer: Timer,
    timer_idx: u32,
    irq_num: u32,
    handler: DelayTimerCallback,
    cookie: *mut c_void,
}

impl DelayTimer {
    /// Create a new DelayTimer instance on a hardware timer (1..=4).
    ///
    /// Timer indices 1 to 4 correspond to hardware TIM2..TIM5 (or TIMER1..TIMER4 in Esprit).
    pub fn new(timer_idx: u32) -> Result<Self, &'static str> {
        if timer_idx == 0 || timer_idx >= (TIMER_MAX_INSTANCES as u32) {
            return Err("Invalid delay timer index: must be 1..=4");
        }

        unsafe {
            if DELAY_TIMER_INSTANCES[timer_idx as usize].is_some() {
                return Err("Delay timer instance already in use");
            }
        }

        let timer = Timer::new(timer_idx)?;
        let irq_num = LN_IRQ_TIMER1 + timer_idx - 1;

        Ok(Self {
            timer,
            timer_idx,
            irq_num,
            handler: None,
            cookie: core::ptr::null_mut(),
        })
    }

    /// Create a new DelayTimer instance with channel parameter (C++ API parity).
    pub fn new_with_channel(timer_idx: u32, _channel: u32) -> Result<Self, &'static str> {
        Self::new(timer_idx)
    }

    /// Return the timer index (1..=4).
    #[inline]
    pub fn timer_idx(&self) -> u32 {
        self.timer_idx
    }

    /// Register interrupt callback and cookie.
    pub fn set_interrupt(&mut self, handler: DelayTimerCallback, cookie: *mut c_void) {
        self.handler = handler;
        self.cookie = cookie;
    }

    /// Enable the hardware timer update interrupt in NVIC/PFIC and DIEN.
    pub fn enable_interrupt(&mut self) {
        unsafe {
            ln_enable_interrupt(&self.irq_num);
            let regs = TimerRegisters::ptr(self.timer_idx);
            let mut dien = read_volatile(&mut (*regs).dien);
            dien |= TIMER_DMAINTEN_UPIE;
            write_volatile(&mut (*regs).dien, dien);
        }
    }

    /// Disable the hardware timer update interrupt in DIEN and NVIC/PFIC.
    pub fn disable_interrupt(&mut self) {
        unsafe {
            let regs = TimerRegisters::ptr(self.timer_idx);
            let mut dien = read_volatile(&mut (*regs).dien);
            dien &= !TIMER_DMAINTEN_UPIE;
            write_volatile(&mut (*regs).dien, dien);
            ln_disable_interrupt(&self.irq_num);
        }
    }

    /// Arm the timer to expire after `duration_us` microseconds and trigger the interrupt.
    pub fn arm(&mut self, duration_us: u32) {
        self.disable_interrupt();
        self.timer.disable();

        // Register self in dispatch table
        unsafe {
            DELAY_TIMER_INSTANCES[self.timer_idx as usize] = Some(self as *mut Self);
        }

        let periph = match self.timer_idx {
            TIMER_INSTANCE_0 => Peripheral::Timer0,
            TIMER_INSTANCE_1 => Peripheral::Timer1,
            TIMER_INSTANCE_2 => Peripheral::Timer2,
            TIMER_INSTANCE_3 => Peripheral::Timer3,
            TIMER_INSTANCE_4 => Peripheral::Timer4,
            _ => Peripheral::Timer1,
        };

        let clock = get_clock(periph) / HZ_PER_MHZ; // in MHz => clock cycles per µs
        let cycles = duration_us * clock;
        let psc = cycles >> TIMER_PRESCALER_SHIFT;
        let mut car = cycles & TIMER_PRESCALER_MASK;
        if car > 0 {
            car -= 1;
        }

        let regs = TimerRegisters::ptr(self.timer_idx);
        unsafe {
            write_volatile(&mut (*regs).ctl0, TIMER_CTL0_SPM | TIMER_CTL0_UPS);
            write_volatile(&mut (*regs).ctl1, 0);
            write_volatile(&mut (*regs).dien, 0);
            write_volatile(&mut (*regs).intf, 0);
            write_volatile(&mut (*regs).psc, psc);
            write_volatile(&mut (*regs).car, car);
            write_volatile(&mut (*regs).cnt, 0);
        }

        self.enable_interrupt();
        self.timer.enable();
    }

    /// Internal IRQ handler called when hardware update interrupt fires.
    pub fn on_irq(&mut self) {
        self.disable_interrupt();
        self.timer.disable();

        let regs = TimerRegisters::ptr(self.timer_idx);
        unsafe {
            let mut intf = read_volatile(&mut (*regs).intf);
            intf &= !TIMER_INTF_UPIF;
            write_volatile(&mut (*regs).intf, intf);
        }

        if let Some(handler) = self.handler {
            unsafe {
                handler(self.cookie);
            }
        }
    }
}

impl timer::DelayTimer for DelayTimer {
    /// Arm the one-shot delay, in microseconds (the timer runs at 1 MHz).
    fn arm_us(&mut self, duration_us: u32) -> HalResult<()> {
        DelayTimer::arm(self, duration_us);
        Ok(())
    }

    /// Typed handler: the family keeps its C-style callback, and the trampoline
    /// below looks the handler up by timer index (the cookie).
    fn set_handler(&mut self, handler: Option<&'static dyn timer::DelayHandler>) {
        DELAY_HANDLERS.set(self.timer_idx as usize, handler);
        match handler {
            Some(_) => {
                let cookie = self.timer_idx as usize as *mut core::ffi::c_void;
                self.set_interrupt(Some(delay_trampoline), cookie);
            }
            None => self.set_interrupt(None, core::ptr::null_mut()),
        }
    }

    fn enable_interrupt(&mut self) {
        DelayTimer::enable_interrupt(self);
    }

    fn disable_interrupt(&mut self) {
        DelayTimer::disable_interrupt(self);
    }
}

impl Drop for DelayTimer {
    fn drop(&mut self) {
        self.timer.disable();
        self.disable_interrupt();
        unsafe {
            DELAY_TIMER_INSTANCES[self.timer_idx as usize] = None;
        }
    }
}

/// Central interrupt dispatcher invoked by vector ISR handlers.
pub fn delay_timer_interrupt_handler(timer_id: usize) {
    if timer_id < TIMER_MAX_INSTANCES {
        unsafe {
            if let Some(ptr) = DELAY_TIMER_INSTANCES[timer_id] {
                (*ptr).on_irq();
            }
        }
    }
}
