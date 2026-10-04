use crate::*;
use core::ffi::c_void;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct ln_serial_tx_c { pub dummy: *mut c_void }
#[derive(Clone, Copy)]
#[repr(C)]
pub struct ln_serial_rx_c { pub dummy: *mut c_void }

pub type ln_serial_event_cb = ::core::option::Option<unsafe extern "C" fn(cookie: *mut c_void, event: i32)>;

pub fn pack_tx_handle(instance: u32) -> *mut ln_serial_tx_c { instance as usize as *mut ln_serial_tx_c }
pub fn unpack_tx_handle(handle: *mut ln_serial_tx_c) -> u32 { handle as usize as u32 }
pub fn pack_rx_handle(instance: u32) -> *mut ln_serial_rx_c { instance as usize as *mut ln_serial_rx_c }
pub fn unpack_rx_handle(handle: *mut ln_serial_rx_c) -> u32 { handle as usize as u32 }

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_tx_create(instance: u32, dma: bool, _buffered: bool) -> *mut ln_serial_tx_c {
    let _tx = UartTx::new(instance, dma);
    pack_tx_handle(instance)
}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_tx_delete(_s: *mut ln_serial_tx_c) {}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_tx_init(s: *mut ln_serial_tx_c) -> bool {
    let mut tx = UartTx { instance: unpack_tx_handle(s) };
    tx.init()
}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_tx_set_speed(s: *mut ln_serial_tx_c, speed: u32) -> bool {
    let mut tx = UartTx { instance: unpack_tx_handle(s) };
    tx.set_speed(speed)
}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_tx_transmit(s: *mut ln_serial_tx_c, size: u32, buffer: *const u8) -> bool {
    let mut tx = UartTx { instance: unpack_tx_handle(s) };
    let slice = unsafe { core::slice::from_raw_parts(buffer, size as usize) };
    tx.transmit(slice)
}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_tx_raw_write(s: *mut ln_serial_tx_c, size: u32, buffer: *const u8) -> bool {
    lnserial_tx_transmit(s, size, buffer)
}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_rx_create(instance: u32, _rx_buffer_size: u32, dma: bool) -> *mut ln_serial_rx_c {
    let _rx = UartRx::new(instance, dma);
    pack_rx_handle(instance)
}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_rx_delete(_s: *mut ln_serial_rx_c) {}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_rx_init(s: *mut ln_serial_rx_c) -> bool {
    let mut rx = UartRx { instance: unpack_rx_handle(s) };
    rx.init()
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
    let mut rx = UartRx { instance: unpack_rx_handle(s) };
    rx.enable_rx(enabled)
}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_rx_purge_rx(_s: *mut ln_serial_rx_c) {}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_rx_read(s: *mut ln_serial_rx_c, max: u32, to: *mut u8) -> i32 {
    let mut rx = UartRx { instance: unpack_rx_handle(s) };
    if max == 0 {
        return 0;
    }
    let slice = unsafe { core::slice::from_raw_parts_mut(to, max as usize) };
    rx.read(slice) as i32
}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_rx_set_callback(_s: *mut ln_serial_rx_c, _cb: ln_serial_event_cb, _cookie: *mut c_void) {}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_rx_get_read_pointer(_s: *mut ln_serial_rx_c, _to: *mut *mut u8) -> i32 { 0 }

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_rx_consume(_s: *mut ln_serial_rx_c, _n: u32) {}
