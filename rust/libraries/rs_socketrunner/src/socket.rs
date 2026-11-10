//! Safe Rust wrapper for lnSocket.

use core::ffi::c_void;
use core::ptr;
use crate::c_socket::*;
use crate::types::{ReadReleaser, RxPacket, SocketError, SocketStatus};

/// Safe RAII wrapper around a native C++ `lnSocket` instance.
pub struct LnSocket {
    raw: *mut ln_socket_c,
}

impl LnSocket {
    /// Creates and binds a new listening socket on `port`.
    /// `cb` and `arg` are the event notification callback parameters.
    ///
    /// # Safety
    /// The caller must ensure that `arg` points to valid memory for the socket's lifetime
    /// or is null, and that `cb` is safe to call from socket interrupt/event contexts.
    pub unsafe fn create(port: u16, cb: LnSocketCallback, arg: *mut c_void) -> Result<Self, SocketError> {
        let raw = unsafe { ln_socket_create(port, cb, arg) };
        if raw.is_null() {
            return Err(SocketError::CreationFailed);
        }
        Ok(Self { raw })
    }

    /// Puts the socket into asynchronous event-driven mode.
    pub fn async_mode(&mut self) -> Result<(), SocketError> {
        let status = unsafe { ln_socket_async_mode(self.raw) };
        if status == (SocketStatus::Ok as u32) {
            Ok(())
        } else {
            Err(SocketError::AsyncModeFailed)
        }
    }

    /// Begins accepting incoming TCP connections.
    pub fn accept(&mut self) -> Result<(), SocketError> {
        let status = unsafe { ln_socket_accept(self.raw) };
        if status == (SocketStatus::Ok as u32) {
            Ok(())
        } else {
            Err(SocketError::AcceptFailed)
        }
    }

    /// Disconnects the currently connected client.
    pub fn disconnect_client(&mut self) {
        unsafe {
            ln_socket_disconnect_client(self.raw);
        }
    }

    /// Reads available incoming data without copying.
    ///
    /// Returns `Ok(Some(RxPacket))` if bytes were received. The returned `RxPacket`
    /// automatically releases the buffer via `free_read_data()` upon being dropped.
    /// Returns `Ok(None)` if no data was available.
    pub fn read_packet(&mut self) -> Result<Option<RxPacket<'_, Self>>, SocketError> {
        let mut n: u32 = 0;
        let mut data_ptr: *mut u8 = ptr::null_mut();

        let status = unsafe { ln_socket_read(self.raw, &mut n, &mut data_ptr) };
        if status != (SocketStatus::Ok as u32) {
            return Err(SocketError::ReadFailed);
        }

        if n == 0 || data_ptr.is_null() {
            return Ok(None);
        }

        let slice = unsafe { core::slice::from_raw_parts(data_ptr, n as usize) };
        Ok(Some(RxPacket::new(self, slice)))
    }

    /// Writes raw slice data to the socket.
    /// Returns the number of bytes actually written (may be less than `data.len()`).
    pub fn write(&mut self, data: &[u8]) -> Result<usize, SocketError> {
        let mut done: u32 = 0;
        let status = unsafe {
            ln_socket_write(
                self.raw,
                data.len() as u32,
                data.as_ptr(),
                &mut done,
            )
        };
        if status != (SocketStatus::Ok as u32) {
            return Err(SocketError::WriteFailed);
        }
        Ok(done as usize)
    }

    /// Returns the number of bytes available in the socket TX buffer.
    pub fn write_buffer_available(&mut self) -> usize {
        let mut avail: u32 = 0;
        let status = unsafe { ln_socket_write_buffer_available(self.raw, &mut avail) };
        if status == (SocketStatus::Ok as u32) {
            avail as usize
        } else {
            0
        }
    }

    pub fn raw(&self) -> *mut ln_socket_c {
        self.raw
    }

    /// Flushes any pending output in the underlying socket.
    pub fn flush(&mut self) -> Result<(), SocketError> {
        let status = unsafe { ln_socket_flush(self.raw) };
        if status == (SocketStatus::Ok as u32) {
            Ok(())
        } else {
            Err(SocketError::WriteFailed)
        }
    }
}

impl ReadReleaser for LnSocket {
    fn release_read_data(&mut self) {
        unsafe {
            ln_socket_free_read_data(self.raw);
        }
    }
}

impl Drop for LnSocket {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe {
                ln_socket_delete(self.raw);
            }
            self.raw = ptr::null_mut();
        }
    }
}
