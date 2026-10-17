#![allow(unused_variables, unused_imports, unused_parens)]
#![allow(non_camel_case_types, non_snake_case, dead_code)]
#![no_std]

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

pub mod registers;
pub mod shim;
pub use shim::*;

use core::ffi::c_void;
use core::ptr::{read_volatile, write_volatile};
use registers::*;
use rs_dma_bluepill::{DmaChannel, DmaEngine};
use rs_esprit::BinarySemaphore;
use rs_rcu_bluepill::{enable, Peripheral};

unsafe extern "C" {
    fn xTimerCreate(
        pcTimerName: *const cty::c_char,
        xTimerPeriodInTicks: u32,
        uxAutoReload: u32,
        pvTimerID: *mut c_void,
        pxCallbackFunction: unsafe extern "C" fn(xTimer: *mut c_void),
    ) -> *mut c_void;
    fn xTimerGenericCommandFromTask(
        xTimer: *mut c_void,
        xCommandID: i32,
        xOptionalValue: u32,
        pxHigherPriorityTaskWoken: *mut i32,
        xTicksToWait: u32,
    ) -> i32;
    fn pvTimerGetTimerID(xTimer: *mut c_void) -> *mut c_void;
    fn xTaskGetTickCount() -> u32;
    fn vPortEnterCritical();
    fn vPortExitCritical();
}

#[inline]
pub(crate) unsafe fn xTimerStart(timer: *mut c_void, ticks_to_wait: u32) -> i32 {
    unsafe {
        let now = xTaskGetTickCount();
        xTimerGenericCommandFromTask(timer, 1, now, core::ptr::null_mut(), ticks_to_wait)
    }
}

#[inline]
pub(crate) unsafe fn xTimerStop(timer: *mut c_void, ticks_to_wait: u32) -> i32 {
    unsafe {
        xTimerGenericCommandFromTask(timer, 3, 0, core::ptr::null_mut(), ticks_to_wait)
    }
}

#[inline]
pub(crate) unsafe fn xTimerDelete(timer: *mut c_void, ticks_to_wait: u32) -> i32 {
    unsafe {
        xTimerGenericCommandFromTask(timer, 5, 0, core::ptr::null_mut(), ticks_to_wait)
    }
}

pub(crate) fn uart_set_baudrate(instance: u32, speed: u32) -> bool {
    let regs = UartRegisters::ptr(instance);
    let periph = match instance {
        UART_INSTANCE_0 => Peripheral::Uart0,
        UART_INSTANCE_1 => Peripheral::Uart1,
        UART_INSTANCE_2 => Peripheral::Uart2,
        _ => Peripheral::Uart0,
    };
    let pclk = rs_rcu_bluepill::get_clock(periph);
    let usartdiv = (pclk + (speed / 2)) / speed;
    unsafe {
        write_volatile(&mut (*regs).baud, usartdiv);
    }
    true
}

/// Idiomatic struct representing a UART transmitter.
pub struct UartTx {
    pub(crate) instance: u32,
    pub(crate) use_dma: bool,
    pub(crate) dma: Option<DmaChannel>,
    sem: BinarySemaphore,
}

impl UartTx {
    pub fn new(instance: u32, dma: bool) -> Self {
        if instance == UART_INSTANCE_0 {
            enable(Peripheral::Uart0);
        } else if instance == UART_INSTANCE_1 {
            enable(Peripheral::Uart1);
        } else if instance == UART_INSTANCE_2 {
            enable(Peripheral::Uart2);
        }
        let dma_chan = if dma {
            let ch_idx = match instance {
                UART_INSTANCE_0 => UART0_DMA_TX_CHANNEL,
                UART_INSTANCE_1 => UART1_DMA_TX_CHANNEL,
                UART_INSTANCE_2 => UART2_DMA_TX_CHANNEL,
                _ => UART0_DMA_TX_CHANNEL,
            };
            Some(DmaChannel::new(DmaEngine::Dma0, ch_idx))
        } else {
            None
        };
        Self {
            instance,
            use_dma: dma,
            dma: dma_chan,
            sem: BinarySemaphore::new(),
        }
    }

    #[inline(always)]
    pub fn instance(&self) -> u32 {
        self.instance
    }

    pub fn init(&mut self) -> bool {
        let (tx_pin, _) = match self.instance {
            UART_INSTANCE_0 => (rs_gpio_bluepill::Pin::PA9, rs_gpio_bluepill::Pin::PA10),
            UART_INSTANCE_1 => (rs_gpio_bluepill::Pin::PA2, rs_gpio_bluepill::Pin::PA3),
            UART_INSTANCE_2 => (rs_gpio_bluepill::Pin::PB10, rs_gpio_bluepill::Pin::PB11),
            _ => (rs_gpio_bluepill::Pin::PA9, rs_gpio_bluepill::Pin::PA10),
        };
        rs_gpio_bluepill::set_mode(tx_pin, rs_gpio_bluepill::Mode::AlternatePushPull, 50);

        let regs = UartRegisters::ptr(self.instance);
        unsafe {
            let mut ctl0 = read_volatile(&mut (*regs).ctl0);
            ctl0 |= USART_CTL0_UEN | USART_CTL0_TEN;
            write_volatile(&mut (*regs).ctl0, ctl0);
        }
        true
    }

    pub fn set_speed(&mut self, speed: u32) -> bool {
        uart_set_baudrate(self.instance, speed)
    }

    pub fn transmit(&mut self, buffer: &[u8]) -> bool {
        if buffer.is_empty() {
            return true;
        }
        let regs = UartRegisters::ptr(self.instance);
        let size = buffer.len() as u32;

        unsafe {
            if self.use_dma {
                if let Some(ref mut dma) = self.dma {
                    self.sem.try_take();

                    let mut ctl2 = read_volatile(&mut (*regs).ctl2);
                    ctl2 |= USART_CTL2_DMAT;
                    write_volatile(&mut (*regs).ctl2, ctl2);

                    dma.attach_completion_semaphore(&self.sem);

                    dma.begin_tx_transfer(
                        &(*regs).data as *const _ as u32,
                        buffer.as_ptr() as u32,
                        size,
                        false,
                        false,
                    );

                    self.sem.take();

                    dma.end_transfer();
                    dma.detach_callback();

                    while (read_volatile(&mut (*regs).stat) & USART_STAT_TC) == 0 {
                        core::hint::spin_loop();
                    }
                }
            } else {
                for &byte in buffer {
                    while (read_volatile(&mut (*regs).stat) & USART_STAT_TBE) == 0 {}
                    write_volatile(&mut (*regs).data, byte as u32);
                }
                while (read_volatile(&mut (*regs).stat) & USART_STAT_TC) == 0 {}
            }
        }
        true
    }
}

unsafe extern "C" fn uart_rx_dma_callback(_is_half: bool, cookie: *mut c_void) {
    let instance = cookie as usize;
    if instance < UART_MAX_INSTANCES {
        unsafe {
            if let Some(ref mut rx) = RX_INSTANCES[instance] {
                rx.check_for_new_data();
            }
        }
    }
}

unsafe extern "C" fn uart_rx_timer_cb(xtimer: *mut c_void) {
    let instance = unsafe { pvTimerGetTimerID(xtimer) } as usize;
    if instance < UART_MAX_INSTANCES {
        unsafe {
            if let Some(ref mut rx) = RX_INSTANCES[instance] {
                rx.check_for_new_data();
            }
        }
    }
}

/// Idiomatic struct representing a UART receiver with ring buffer and circular DMA.
pub struct UartRx {
    pub(crate) instance: u32,
    pub(crate) use_dma: bool,
    pub(crate) rx_buffer: Vec<u8>,
    pub(crate) rx_buffer_size: usize,
    pub(crate) rx_mask: usize,
    pub(crate) rx_head: usize,
    pub(crate) rx_tail: usize,
    pub(crate) dma: Option<DmaChannel>,
    pub(crate) timer: *mut c_void,
    pub(crate) callback: ln_serial_event_cb,
    pub(crate) callback_cookie: *mut c_void,
    pub(crate) rx_enabled: bool,
}

impl UartRx {
    pub fn new(instance: u32, buffer_size: u32, dma: bool) -> Self {
        if instance == UART_INSTANCE_0 {
            enable(Peripheral::Uart0);
        } else if instance == UART_INSTANCE_1 {
            enable(Peripheral::Uart1);
        } else if instance == UART_INSTANCE_2 {
            enable(Peripheral::Uart2);
        }
        let size = if buffer_size == 0 {
            256
        } else {
            buffer_size.next_power_of_two() as usize
        };
        let rx_buffer = alloc::vec![0u8; size];
        let rx_mask = size - 1;

        let dma_chan = if dma {
            let ch_idx = match instance {
                UART_INSTANCE_0 => UART0_DMA_RX_CHANNEL,
                UART_INSTANCE_1 => UART1_DMA_RX_CHANNEL,
                UART_INSTANCE_2 => UART2_DMA_RX_CHANNEL,
                _ => UART0_DMA_RX_CHANNEL,
            };
            Some(DmaChannel::new(DmaEngine::Dma0, ch_idx))
        } else {
            None
        };

        let timer = unsafe {
            xTimerCreate(
                b"uart_rx\0".as_ptr() as *const cty::c_char,
                20,
                1,
                instance as usize as *mut c_void,
                uart_rx_timer_cb,
            )
        };

        Self {
            instance,
            use_dma: dma,
            rx_buffer,
            rx_buffer_size: size,
            rx_mask,
            rx_head: 0,
            rx_tail: 0,
            dma: dma_chan,
            timer,
            callback: None,
            callback_cookie: core::ptr::null_mut(),
            rx_enabled: false,
        }
    }

    #[inline(always)]
    pub fn instance(&self) -> u32 {
        self.instance
    }

    pub fn init(&mut self) -> bool {
        let (_, rx_pin) = match self.instance {
            UART_INSTANCE_0 => (rs_gpio_bluepill::Pin::PA9, rs_gpio_bluepill::Pin::PA10),
            UART_INSTANCE_1 => (rs_gpio_bluepill::Pin::PA2, rs_gpio_bluepill::Pin::PA3),
            UART_INSTANCE_2 => (rs_gpio_bluepill::Pin::PB10, rs_gpio_bluepill::Pin::PB11),
            _ => (rs_gpio_bluepill::Pin::PA9, rs_gpio_bluepill::Pin::PA10),
        };
        rs_gpio_bluepill::set_mode(rx_pin, rs_gpio_bluepill::Mode::InputFloating, 0);

        let regs = UartRegisters::ptr(self.instance);
        unsafe {
            let mut ctl0 = read_volatile(&mut (*regs).ctl0);
            ctl0 |= USART_CTL0_UEN | USART_CTL0_REN;
            write_volatile(&mut (*regs).ctl0, ctl0);
        }
        true
    }

    pub fn set_speed(&mut self, speed: u32) -> bool {
        uart_set_baudrate(self.instance, speed)
    }

    pub fn enable_rx(&mut self, enabled: bool) -> bool {
        let regs = UartRegisters::ptr(self.instance);
        unsafe {
            vPortEnterCritical();
        }

        self.rx_enabled = enabled;
        if enabled {
            self.rx_head = 0;
            self.rx_tail = 0;

            unsafe {
                while (read_volatile(&mut (*regs).stat) & USART_STAT_RBNE) != 0 {
                    let _ = read_volatile(&mut (*regs).data);
                }

                let mut ctl0 = read_volatile(&mut (*regs).ctl0);
                ctl0 |= USART_CTL0_UEN | USART_CTL0_REN;
                write_volatile(&mut (*regs).ctl0, ctl0);

                if self.use_dma {
                    let mut ctl2 = read_volatile(&mut (*regs).ctl2);
                    ctl2 |= USART_CTL2_DMAR;
                    write_volatile(&mut (*regs).ctl2, ctl2);

                    if let Some(ref mut dma) = self.dma {
                        dma.attach_callback(uart_rx_dma_callback, self.instance as usize as *mut c_void);
                        dma.begin_circular_rx_transfer(
                            &(*regs).data as *const _ as u32,
                            self.rx_buffer.as_ptr() as u32,
                            self.rx_buffer_size as u32,
                            false,
                            false,
                        );
                    }

                    if !self.timer.is_null() {
                        xTimerStart(self.timer, 1);
                    }
                }
                vPortExitCritical();
            }
        } else {
            unsafe {
                if !self.timer.is_null() {
                    xTimerStop(self.timer, 1);
                }
                if self.use_dma {
                    if let Some(ref mut dma) = self.dma {
                        dma.end_transfer();
                    }
                    let mut ctl2 = read_volatile(&mut (*regs).ctl2);
                    ctl2 &= !USART_CTL2_DMAR;
                    write_volatile(&mut (*regs).ctl2, ctl2);
                }
                let mut ctl0 = read_volatile(&mut (*regs).ctl0);
                ctl0 &= !USART_CTL0_REN;
                write_volatile(&mut (*regs).ctl0, ctl0);

                vPortExitCritical();
            }
        }
        true
    }

    #[inline]
    pub fn current_dma_pos(&self) -> usize {
        if self.use_dma {
            if let Some(ref dma) = self.dma {
                let rem = dma.remaining_transfers() as usize;
                if rem <= self.rx_buffer_size {
                    self.rx_buffer_size - rem
                } else {
                    0
                }
            } else {
                0
            }
        } else {
            self.rx_head & self.rx_mask
        }
    }

    pub fn check_for_new_data(&mut self) {
        if !self.rx_enabled {
            return;
        }
        let count = self.current_dma_pos();
        let masked_head = self.rx_head & self.rx_mask;
        if count == masked_head {
            return;
        }
        let t = self.rx_head & self.rx_mask;
        self.rx_head = (self.rx_head & !self.rx_mask) + count;
        if self.rx_head.saturating_sub(self.rx_tail) > self.rx_buffer_size {
            self.rx_tail = self.rx_head; // overflow recovery
        }
        if t >= count {
            self.rx_head += self.rx_buffer_size;
            if self.rx_head.saturating_sub(self.rx_tail) > self.rx_buffer_size {
                self.rx_tail = self.rx_head;
            }
        }
        if let Some(cb) = self.callback {
            unsafe {
                cb(self.callback_cookie, 0); // 0 = SerialEvent::DataAvailable
            }
        }
    }

    pub fn get_read_pointer(&mut self) -> (i32, *mut u8) {
        unsafe {
            vPortEnterCritical();
        }
        let h = self.rx_head & self.rx_mask;
        let t = self.rx_tail & self.rx_mask;

        let nb = if h >= t {
            if self.rx_head.saturating_sub(self.rx_tail) > self.rx_buffer_size {
                self.rx_tail = self.rx_head;
                0
            } else {
                self.rx_head - self.rx_tail
            }
        } else {
            self.rx_buffer_size - t
        };

        let ptr = if nb > 0 {
            unsafe { self.rx_buffer.as_mut_ptr().add(t) }
        } else {
            core::ptr::null_mut()
        };

        unsafe {
            vPortExitCritical();
        }
        (nb as i32, ptr)
    }

    pub fn consume(&mut self, n: u32) {
        unsafe {
            vPortEnterCritical();
        }
        self.rx_tail += n as usize;
        if self.rx_tail > self.rx_buffer_size && self.rx_head > self.rx_buffer_size {
            self.rx_tail -= self.rx_buffer_size;
            self.rx_head -= self.rx_buffer_size;
        }
        unsafe {
            vPortExitCritical();
        }
    }

    pub fn purge_rx(&mut self) {
        unsafe {
            vPortEnterCritical();
        }
        self.rx_head = 0;
        self.rx_tail = 0;
        unsafe {
            vPortExitCritical();
        }
    }

    pub fn read(&mut self, to: &mut [u8]) -> usize {
        let (nb, ptr) = self.get_read_pointer();
        if nb <= 0 || ptr.is_null() {
            return 0;
        }
        let count = core::cmp::min(nb as usize, to.len());
        unsafe {
            core::ptr::copy_nonoverlapping(ptr, to.as_mut_ptr(), count);
        }
        self.consume(count as u32);
        count
    }
}

impl Drop for UartRx {
    fn drop(&mut self) {
        self.enable_rx(false);
        if !self.timer.is_null() {
            unsafe {
                xTimerDelete(self.timer, 1);
            }
            self.timer = core::ptr::null_mut();
        }
    }
}

static mut UART_TX_HANDLERS: [Option<&'static dyn rs_esprit::UartTxHandler>; UART_MAX_INSTANCES] = [None, None, None];
static mut UART_RX_HANDLERS: [Option<&'static dyn rs_esprit::UartRxHandler>; UART_MAX_INSTANCES] = [None, None, None];

impl rs_esprit::UartTx for UartTx {
    #[inline]
    fn configure(&mut self, config: &rs_esprit::UartConfig) -> bool {
        self.set_speed(config.baudrate)
    }

    #[inline]
    fn set_speed(&mut self, baudrate: u32) -> bool {
        self.set_speed(baudrate)
    }

    #[inline]
    fn transmit(&mut self, buffer: &[u8]) -> bool {
        let res = self.transmit(buffer);
        if res {
            unsafe {
                if let Some(handler) = UART_TX_HANDLERS[self.instance as usize] {
                    handler.on_tx_complete();
                }
            }
        }
        res
    }

    #[inline]
    fn transmit_no_block(&mut self, buffer: &[u8]) -> i32 {
        if self.transmit(buffer) {
            buffer.len() as i32
        } else {
            -1
        }
    }

    #[inline]
    fn set_tx_handler(&mut self, handler: Option<&'static dyn rs_esprit::UartTxHandler>) {
        unsafe {
            UART_TX_HANDLERS[self.instance as usize] = handler;
        }
    }
}

impl rs_esprit::UartRx for UartRx {
    #[inline]
    fn configure(&mut self, config: &rs_esprit::UartConfig) -> bool {
        self.set_speed(config.baudrate)
    }

    #[inline]
    fn set_speed(&mut self, baudrate: u32) -> bool {
        self.set_speed(baudrate)
    }

    #[inline]
    fn enable_rx(&mut self, enabled: bool) -> bool {
        self.enable_rx(enabled)
    }

    #[inline]
    fn set_rx_handler(&mut self, handler: Option<&'static dyn rs_esprit::UartRxHandler>) {
        unsafe {
            UART_RX_HANDLERS[self.instance as usize] = handler;
        }
    }
}
