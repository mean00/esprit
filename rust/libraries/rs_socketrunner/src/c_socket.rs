//! Raw C FFI bindings for lnSocket.

use core::ffi::c_void;

#[repr(C)]
pub struct ln_socket_c {
    _private: [u8; 0],
}

pub type LnSocketCallback = extern "C" fn(evt: u32, arg: *mut c_void);

unsafe extern "C" {
    pub fn ln_socket_create(port: u16, cb: LnSocketCallback, arg: *mut c_void) -> *mut ln_socket_c;
    pub fn ln_socket_delete(sock: *mut ln_socket_c);
    pub fn ln_socket_async_mode(sock: *mut ln_socket_c) -> u32;
    pub fn ln_socket_accept(sock: *mut ln_socket_c) -> u32;
    pub fn ln_socket_disconnect_client(sock: *mut ln_socket_c) -> u32;
    pub fn ln_socket_read(sock: *mut ln_socket_c, n: *mut u32, data: *mut *mut u8) -> u32;
    pub fn ln_socket_free_read_data(sock: *mut ln_socket_c) -> u32;
    pub fn ln_socket_write(sock: *mut ln_socket_c, n: u32, data: *const u8, done: *mut u32) -> u32;
    pub fn ln_socket_write_buffer_available(sock: *mut ln_socket_c, avail: *mut u32) -> u32;
    pub fn ln_socket_flush(sock: *mut ln_socket_c) -> u32;

    // Optional weak hook notified when link state changes
    pub fn ln_set_connected_state(state: bool);
}
