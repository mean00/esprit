#![allow(unused_variables, unused_imports, unused_parens)]
#![allow(non_camel_case_types, non_snake_case, dead_code)]

#![no_std]

pub mod registers;

use registers::UartRegisters;
use rs_rcu_bluepill::{Peripheral, enable};
use rs_dma_bluepill::{DmaChannel, DmaEngine};
use core::ptr::{read_volatile, write_volatile};
use core::ffi::c_void;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct ln_serial_tx_c { pub dummy: *mut c_void }
#[derive(Clone, Copy)]
#[repr(C)]
pub struct ln_serial_rx_c { pub dummy: *mut c_void }

pub type ln_serial_event_cb = ::core::option::Option<unsafe extern "C" fn(cookie: *mut c_void, event: i32)>;

struct UartConfig {
    use_dma: bool,
    speed: u32,
    rx_enabled: bool,
}

static mut UART_CONFIGS: [UartConfig; 3] = [
    UartConfig { use_dma: false, speed: 115200, rx_enabled: false },
    UartConfig { use_dma: false, speed: 115200, rx_enabled: false },
    UartConfig { use_dma: false, speed: 115200, rx_enabled: false },
];

fn pack_tx_handle(instance: u32) -> *mut ln_serial_tx_c { instance as usize as *mut ln_serial_tx_c }
fn unpack_tx_handle(handle: *mut ln_serial_tx_c) -> u32 { handle as usize as u32 }
fn pack_rx_handle(instance: u32) -> *mut ln_serial_rx_c { instance as usize as *mut ln_serial_rx_c }
fn unpack_rx_handle(handle: *mut ln_serial_rx_c) -> u32 { handle as usize as u32 }

// --- TX API ---

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_tx_create(instance: u32, dma: bool, _buffered: bool) -> *mut ln_serial_tx_c {
    unsafe {
        UART_CONFIGS[instance as usize].use_dma = dma;
        if instance == 0 { enable(Peripheral::Uart0); }
        else if instance == 1 { enable(Peripheral::Uart1); }
        else if instance == 2 { enable(Peripheral::Uart2); }
    }
    pack_tx_handle(instance)
}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_tx_delete(_s: *mut ln_serial_tx_c) {}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_tx_init(s: *mut ln_serial_tx_c) -> bool {
    let instance = unpack_tx_handle(s);
    let regs = UartRegisters::ptr(instance);
    unsafe {
        let mut ctl0 = read_volatile(&mut (*regs).ctl0);
        ctl0 |= (1 << 13) | (1 << 3); // UEN (USART Enable) | TEN (Transmit Enable)
        write_volatile(&mut (*regs).ctl0, ctl0);
    }
    true
}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_tx_set_speed(s: *mut ln_serial_tx_c, speed: u32) -> bool {
    let instance = unpack_tx_handle(s);
    let regs = UartRegisters::ptr(instance);
    let periph = match instance {
        0 => Peripheral::Uart0,
        1 => Peripheral::Uart1,
        2 => Peripheral::Uart2,
        _ => Peripheral::Uart0,
    };
    let pclk = rs_rcu_bluepill::get_clock(periph);
    let usartdiv = (pclk + (speed / 2)) / speed;
    unsafe {
        write_volatile(&mut (*regs).baud, usartdiv);
    }
    true
}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_tx_transmit(s: *mut ln_serial_tx_c, size: u32, buffer: *const u8) -> bool {
    let instance = unpack_tx_handle(s);
    let regs = UartRegisters::ptr(instance);
    
    unsafe {
        if UART_CONFIGS[instance as usize].use_dma {
            // NATIVE DMA BRANCH
            let mut ctl2 = read_volatile(&mut (*regs).ctl2);
            ctl2 |= 1 << 7; // DENT (DMA Enable Transmitter)
            write_volatile(&mut (*regs).ctl2, ctl2);
            
            // Channel mapping: USART0 TX is DMA0 CH3, USART1 TX is DMA0 CH6
            let ch_idx = if instance == 0 { 3 } else { 6 };
            let mut dma = DmaChannel::new(DmaEngine::Dma0, ch_idx);
            
            dma.begin_tx_transfer(
                &(*regs).data as *const _ as u32,
                buffer as u32,
                size,
                false, // 8-bit PSIZE
                false  // 8-bit MSIZE
            );
            
            // Blocking wait for simulation (in a real system, you'd yield or use IRQ TC)
            while !dma.is_transfer_complete() {}
            dma.end_transfer();
            
        } else {
            // NATIVE IRQ/POLLING BRANCH
            for i in 0..size {
                let byte = *buffer.add(i as usize);
                while (read_volatile(&mut (*regs).stat) & (1 << 7)) == 0 {} // Wait TBE (Transmit Buffer Empty)
                write_volatile(&mut (*regs).data, byte as u32);
            }
            while (read_volatile(&mut (*regs).stat) & (1 << 6)) == 0 {} // Wait TC (Transmission Complete)
        }
    }
    true
}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_tx_raw_write(s: *mut ln_serial_tx_c, size: u32, buffer: *const u8) -> bool {
    lnserial_tx_transmit(s, size, buffer)
}

// --- RX API ---

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_rx_create(instance: u32, _rx_buffer_size: u32, dma: bool) -> *mut ln_serial_rx_c {
    unsafe {
        UART_CONFIGS[instance as usize].use_dma = dma;
        if instance == 0 { enable(Peripheral::Uart0); }
        else if instance == 1 { enable(Peripheral::Uart1); }
        else if instance == 2 { enable(Peripheral::Uart2); }
    }
    pack_rx_handle(instance)
}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_rx_delete(_s: *mut ln_serial_rx_c) {}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_rx_init(s: *mut ln_serial_rx_c) -> bool {
    let instance = unpack_rx_handle(s);
    let regs = UartRegisters::ptr(instance);
    unsafe {
        let mut ctl0 = read_volatile(&mut (*regs).ctl0);
        ctl0 |= (1 << 13) | (1 << 2); // UEN | REN (Receiver Enable)
        write_volatile(&mut (*regs).ctl0, ctl0);
    }
    true
}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_rx_set_speed(s: *mut ln_serial_rx_c, speed: u32) -> bool {
    lnserial_tx_set_speed(pack_tx_handle(unpack_rx_handle(s)), speed)
}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_rx_transmit(s: *mut ln_serial_rx_c, size: u32, buffer: *const u8) -> bool {
    lnserial_tx_transmit(pack_tx_handle(unpack_rx_handle(s)), size, buffer)
}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_rx_transmit_no_block(s: *mut ln_serial_rx_c, size: u32, buffer: *const u8) -> i32 {
    lnserial_tx_transmit(pack_tx_handle(unpack_rx_handle(s)), size, buffer);
    size as i32
}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_rx_enable_rx(s: *mut ln_serial_rx_c, enabled: bool) -> bool {
    let instance = unpack_rx_handle(s);
    unsafe { UART_CONFIGS[instance as usize].rx_enabled = enabled; }
    true
}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_rx_purge_rx(_s: *mut ln_serial_rx_c) {}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_rx_read(s: *mut ln_serial_rx_c, max: u32, to: *mut u8) -> i32 {
    let instance = unpack_rx_handle(s);
    let regs = UartRegisters::ptr(instance);
    unsafe {
        if (read_volatile(&mut (*regs).stat) & (1 << 5)) != 0 { // RBNE (Read Buffer Not Empty)
            let byte = read_volatile(&mut (*regs).data) as u8;
            if max > 0 { *to = byte; return 1; }
        }
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_rx_set_callback(_s: *mut ln_serial_rx_c, _cb: ln_serial_event_cb, _cookie: *mut c_void) {}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_rx_get_read_pointer(_s: *mut ln_serial_rx_c, _to: *mut *mut u8) -> i32 { 0 }

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_rx_consume(_s: *mut ln_serial_rx_c, _n: u32) {}

// --- IRQ Handlers for native interception ---

fn USART0_IRQHandler() {
    // Intercept native hardware interrupts here for IRQ mode
}

fn USART1_IRQHandler() {}

fn USART2_IRQHandler() {}
