#![allow(unused_variables, unused_imports, unused_parens)]
#![allow(non_camel_case_types, non_snake_case, dead_code)]

#![no_std]

pub mod registers;

use registers::*;
use rs_rcu_bluepill::{Peripheral, enable};
use rs_dma_bluepill::{DmaChannel, DmaEngine};
use core::ptr::{read_volatile, write_volatile};

use rs_esprit::BinarySemaphore;

struct UartConfig {
    use_dma: bool,
    speed: u32,
    rx_enabled: bool,
}

static mut UART_CONFIGS: [UartConfig; UART_MAX_INSTANCES] = [
    UartConfig { use_dma: false, speed: UART_DEFAULT_BAUDRATE, rx_enabled: false },
    UartConfig { use_dma: false, speed: UART_DEFAULT_BAUDRATE, rx_enabled: false },
    UartConfig { use_dma: false, speed: UART_DEFAULT_BAUDRATE, rx_enabled: false },
];

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
    sem: BinarySemaphore,
}

impl UartTx {
    pub fn new(instance: u32, dma: bool) -> Self {
        unsafe {
            UART_CONFIGS[instance as usize].use_dma = dma;
            if instance == UART_INSTANCE_0 { enable(Peripheral::Uart0); }
            else if instance == UART_INSTANCE_1 { enable(Peripheral::Uart1); }
            else if instance == UART_INSTANCE_2 { enable(Peripheral::Uart2); }
        }
        Self {
            instance,
            sem: BinarySemaphore::new(),
        }
    }

    #[inline(always)]
    pub fn instance(&self) -> u32 {
        self.instance
    }

    pub fn init(&mut self) -> bool {
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
        let regs = UartRegisters::ptr(self.instance);
        let size = buffer.len() as u32;

        unsafe {
            if UART_CONFIGS[self.instance as usize].use_dma {
                // Clear any pending/stale semaphore token
                self.sem.try_take();

                let mut ctl2 = read_volatile(&mut (*regs).ctl2);
                ctl2 |= USART_CTL2_DMAT;
                write_volatile(&mut (*regs).ctl2, ctl2);

                let ch_idx = if self.instance == UART_INSTANCE_0 {
                    UART0_DMA_TX_CHANNEL
                } else {
                    UART1_DMA_TX_CHANNEL
                };
                let mut dma = DmaChannel::new(DmaEngine::Dma0, ch_idx);

                dma.attach_completion_semaphore(&self.sem);

                dma.begin_tx_transfer(
                    &(*regs).data as *const _ as u32,
                    buffer.as_ptr() as u32,
                    size,
                    false,
                    false,
                );

                // Block until DMA transfer complete interrupt triggers semaphore
                self.sem.take();

                dma.end_transfer();
                dma.detach_callback();

                // Wait for USART Transmission Complete (TC) flag before returning
                while (read_volatile(&mut (*regs).stat) & USART_STAT_TC) == 0 {
                    core::hint::spin_loop();
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

/// Idiomatic struct representing a UART receiver.
pub struct UartRx {
    pub(crate) instance: u32,
}

impl UartRx {
    pub fn new(instance: u32, dma: bool) -> Self {
        unsafe {
            UART_CONFIGS[instance as usize].use_dma = dma;
            if instance == UART_INSTANCE_0 { enable(Peripheral::Uart0); }
            else if instance == UART_INSTANCE_1 { enable(Peripheral::Uart1); }
            else if instance == UART_INSTANCE_2 { enable(Peripheral::Uart2); }
        }
        Self { instance }
    }

    #[inline(always)]
    pub fn instance(&self) -> u32 {
        self.instance
    }

    pub fn init(&mut self) -> bool {
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
        unsafe { UART_CONFIGS[self.instance as usize].rx_enabled = enabled; }
        true
    }

    pub fn purge_rx(&mut self) {}

    pub fn read(&mut self, to: &mut [u8]) -> usize {
        let regs = UartRegisters::ptr(self.instance);
        unsafe {
            if (read_volatile(&mut (*regs).stat) & USART_STAT_RBNE) != 0 {
                let byte = read_volatile(&mut (*regs).data) as u8;
                if !to.is_empty() {
                    to[0] = byte;
                    return 1;
                }
            }
        }
        0
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

pub mod shim;
pub use shim::*;
