use crate::*;
use core::ffi::c_void;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct ln_serial_tx_c {
    pub dummy: *mut c_void,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct ln_serial_rx_c {
    pub dummy: *mut c_void,
}

pub type ln_serial_event_cb =
    ::core::option::Option<unsafe extern "C" fn(cookie: *mut c_void, event: i32)>;

pub fn pack_tx_handle(instance: u32) -> *mut ln_serial_tx_c {
    instance as usize as *mut ln_serial_tx_c
}

pub fn unpack_tx_handle(handle: *mut ln_serial_tx_c) -> u32 {
    handle as usize as u32
}

pub fn pack_rx_handle(instance: u32) -> *mut ln_serial_rx_c {
    instance as usize as *mut ln_serial_rx_c
}

pub fn unpack_rx_handle(handle: *mut ln_serial_rx_c) -> u32 {
    handle as usize as u32
}

pub(crate) static mut TX_INSTANCES: [Option<UartTx>; UART_MAX_INSTANCES] = [None, None, None];
pub(crate) static mut RX_INSTANCES: [Option<UartRx>; UART_MAX_INSTANCES] = [None, None, None];

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_tx_create(
    instance: u32,
    dma: bool,
    _buffered: bool,
) -> *mut ln_serial_tx_c {
    if (instance as usize) < UART_MAX_INSTANCES {
        unsafe {
            TX_INSTANCES[instance as usize] = Some(UartTx::new(instance, dma));
        }
    }
    pack_tx_handle(instance)
}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_tx_delete(s: *mut ln_serial_tx_c) {
    let instance = unpack_tx_handle(s) as usize;
    if instance < UART_MAX_INSTANCES {
        unsafe {
            TX_INSTANCES[instance] = None;
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_tx_init(s: *mut ln_serial_tx_c) -> bool {
    let instance = unpack_tx_handle(s) as usize;
    if instance < UART_MAX_INSTANCES {
        unsafe {
            if let Some(ref mut tx) = TX_INSTANCES[instance] {
                return tx.init();
            }
        }
    }
    false
}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_tx_set_speed(s: *mut ln_serial_tx_c, speed: u32) -> bool {
    let instance = unpack_tx_handle(s) as usize;
    if instance < UART_MAX_INSTANCES {
        unsafe {
            if let Some(ref mut tx) = TX_INSTANCES[instance] {
                return tx.set_speed(speed);
            }
        }
    }
    false
}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_tx_transmit(
    s: *mut ln_serial_tx_c,
    size: u32,
    buffer: *const u8,
) -> bool {
    let instance = unpack_tx_handle(s) as usize;
    if instance < UART_MAX_INSTANCES && !buffer.is_null() {
        unsafe {
            if let Some(ref mut tx) = TX_INSTANCES[instance] {
                let slice = core::slice::from_raw_parts(buffer, size as usize);
                return tx.transmit(slice);
            }
        }
    }
    false
}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_tx_raw_write(
    s: *mut ln_serial_tx_c,
    size: u32,
    buffer: *const u8,
) -> bool {
    lnserial_tx_transmit(s, size, buffer)
}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_rx_create(
    instance: u32,
    rx_buffer_size: u32,
    dma: bool,
) -> *mut ln_serial_rx_c {
    if (instance as usize) < UART_MAX_INSTANCES {
        unsafe {
            if TX_INSTANCES[instance as usize].is_none() {
                TX_INSTANCES[instance as usize] = Some(UartTx::new(instance, dma));
            }
            RX_INSTANCES[instance as usize] = Some(UartRx::new(instance, rx_buffer_size, dma));
        }
    }
    pack_rx_handle(instance)
}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_rx_delete(s: *mut ln_serial_rx_c) {
    let instance = unpack_rx_handle(s) as usize;
    if instance < UART_MAX_INSTANCES {
        unsafe {
            if let Some(ref mut rx) = RX_INSTANCES[instance] {
                rx.enable_rx(false);
            }
            RX_INSTANCES[instance] = None;
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_rx_init(s: *mut ln_serial_rx_c) -> bool {
    let instance = unpack_rx_handle(s) as usize;
    if instance < UART_MAX_INSTANCES {
        unsafe {
            let tx_ok = if let Some(ref mut tx) = TX_INSTANCES[instance] {
                tx.init()
            } else {
                false
            };
            let rx_ok = if let Some(ref mut rx) = RX_INSTANCES[instance] {
                rx.init()
            } else {
                false
            };
            return tx_ok && rx_ok;
        }
    }
    false
}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_rx_set_speed(s: *mut ln_serial_rx_c, speed: u32) -> bool {
    let instance = unpack_rx_handle(s) as usize;
    if instance < UART_MAX_INSTANCES {
        uart_set_baudrate(instance as u32, speed)
    } else {
        false
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_rx_transmit(
    s: *mut ln_serial_rx_c,
    size: u32,
    buffer: *const u8,
) -> bool {
    lnserial_tx_transmit(pack_tx_handle(unpack_rx_handle(s)), size, buffer)
}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_rx_transmit_no_block(
    s: *mut ln_serial_rx_c,
    size: u32,
    buffer: *const u8,
) -> i32 {
    let instance = unpack_rx_handle(s) as usize;
    if instance < UART_MAX_INSTANCES && !buffer.is_null() {
        unsafe {
            if let Some(ref mut tx) = TX_INSTANCES[instance] {
                let slice = core::slice::from_raw_parts(buffer, size as usize);
                let res = tx.transmit(slice);
                if res {
                    if let Some(ref mut rx) = RX_INSTANCES[instance] {
                        if let Some(cb) = rx.callback {
                            cb(rx.callback_cookie, 1); // 1 = SerialEvent::TxDone
                        }
                    }
                    return size as i32;
                }
            }
        }
    }
    -1
}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_rx_enable_rx(s: *mut ln_serial_rx_c, enabled: bool) -> bool {
    let instance = unpack_rx_handle(s) as usize;
    if instance < UART_MAX_INSTANCES {
        unsafe {
            if let Some(ref mut rx) = RX_INSTANCES[instance] {
                return rx.enable_rx(enabled);
            }
        }
    }
    false
}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_rx_purge_rx(s: *mut ln_serial_rx_c) {
    let instance = unpack_rx_handle(s) as usize;
    if instance < UART_MAX_INSTANCES {
        unsafe {
            if let Some(ref mut rx) = RX_INSTANCES[instance] {
                rx.purge_rx();
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_rx_read(s: *mut ln_serial_rx_c, max: u32, to: *mut u8) -> i32 {
    let instance = unpack_rx_handle(s) as usize;
    if instance < UART_MAX_INSTANCES && max > 0 && !to.is_null() {
        unsafe {
            if let Some(ref mut rx) = RX_INSTANCES[instance] {
                let (nb, ptr) = rx.get_read_pointer();
                if nb > 0 && !ptr.is_null() {
                    let to_copy = core::cmp::min(nb as u32, max) as usize;
                    core::ptr::copy_nonoverlapping(ptr, to, to_copy);
                    rx.consume(to_copy as u32);
                    return to_copy as i32;
                }
            }
        }
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_rx_set_callback(
    s: *mut ln_serial_rx_c,
    cb: ln_serial_event_cb,
    cookie: *mut c_void,
) {
    let instance = unpack_rx_handle(s) as usize;
    if instance < UART_MAX_INSTANCES {
        unsafe {
            if let Some(ref mut rx) = RX_INSTANCES[instance] {
                rx.callback = cb;
                rx.callback_cookie = cookie;
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_rx_get_read_pointer(
    s: *mut ln_serial_rx_c,
    to: *mut *mut u8,
) -> i32 {
    let instance = unpack_rx_handle(s) as usize;
    if instance < UART_MAX_INSTANCES {
        unsafe {
            if let Some(ref mut rx) = RX_INSTANCES[instance] {
                let (nb, ptr) = rx.get_read_pointer();
                if !to.is_null() {
                    *to = ptr;
                }
                return nb;
            }
        }
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn lnserial_rx_consume(s: *mut ln_serial_rx_c, n: u32) {
    let instance = unpack_rx_handle(s) as usize;
    if instance < UART_MAX_INSTANCES {
        unsafe {
            if let Some(ref mut rx) = RX_INSTANCES[instance] {
                rx.consume(n);
            }
        }
    }
}
