//! SocketRunner engine implementation.

use rs_rtos::event::EventGroup;

use crate::constants::*;
use crate::socket::Socket;
use crate::types::*;

#[unsafe(no_mangle)]
pub extern "C" fn ln_set_connected_state(_state: bool) {}

/// Helper function to perform forced write on a socket, waiting for `CanWrite` if needed.
fn forced_write_all<S: Socket>(
    socket: &mut S,
    event_group: &EventGroup,
    shift: u32,
    mut data: &[u8],
) -> Result<(), SocketError> {
    let can_write_bit = RunnerEvent::CAN_WRITE.shifted(shift);
    while !data.is_empty() {
        let done = socket.write(data)?;
        if done == 0 {
            event_group.wait_events(can_write_bit, WAIT_WRITE_TIMEOUT_MS);
        } else {
            data = &data[done..];
        }
    }
    Ok(())
}

/// Generic, memory-safe SocketRunner server engine.
///
/// `H` is the user-provided handler implementing `SocketRunnerHandler`.
/// `S` is the socket driver implementing `Socket`.
/// `WRITE_BUF_SIZE` is the staging buffer capacity in bytes (defaults to `DEFAULT_WRITE_BUFFER_SIZE`).
pub struct SocketRunner<
    H: SocketRunnerHandler,
    S: Socket,
    const WRITE_BUF_SIZE: usize = DEFAULT_WRITE_BUFFER_SIZE,
> {
    port: u16,
    shift: u32,
    event_group: EventGroup,
    socket: Option<S>,
    connected: bool,
    write_buffer: [u8; WRITE_BUF_SIZE],
    write_index: usize,
    pub handler: H,
}

impl<H: SocketRunnerHandler, S: Socket> SocketRunner<H, S, DEFAULT_WRITE_BUFFER_SIZE> {
    /// Creates a new SocketRunner instance with the default write buffer size (64 bytes).
    pub fn new(port: u16, event_group: EventGroup, shift: u32, handler: H) -> Self {
        Self::with_buffer_size(port, event_group, shift, handler)
    }
}

impl<H: SocketRunnerHandler, S: Socket, const WRITE_BUF_SIZE: usize> SocketRunner<H, S, WRITE_BUF_SIZE> {
    /// Creates a new SocketRunner instance with a custom write buffer capacity.
    pub fn with_buffer_size(port: u16, event_group: EventGroup, shift: u32, handler: H) -> Self {
        Self {
            port,
            shift,
            event_group,
            socket: None,
            connected: false,
            write_buffer: [0u8; WRITE_BUF_SIZE],
            write_index: 0,
            handler,
        }
    }

    /// Returns the slot shift assigned to this runner.
    #[inline(always)]
    pub fn shift(&self) -> u32 {
        self.shift
    }

    /// Returns true if a remote client is currently connected.
    #[inline(always)]
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Disconnects the active client.
    pub fn disconnect_client(&mut self) {
        self.write_index = 0;
        if let Some(ref mut socket) = self.socket {
            let _ = socket.disconnect_client();
        }
    }

    /// Cleans up any active socket instance and resets state.
    pub fn cleanup(&mut self) {
        self.connected = false;
        self.write_index = 0;
        self.socket = None;
    }

    /// Starts the server socket, enables async mode, and starts accepting.
    pub fn start_server(&mut self) -> Result<(), SocketError> {
        let listener = SocketListener::from_event_group(self.event_group.clone(), self.shift);
        let mut socket = S::create(self.port, listener)?;
        socket.accept()?;
        self.socket = Some(socket);
        Ok(())
    }

    /// Reads incoming packet data using zero-copy semantics.
    /// The returned `RxPacket` releases the receive buffer automatically on Drop.
    pub fn read_packet(&mut self) -> Result<Option<RxPacket<'_, S>>, SocketError> {
        match self.socket.as_mut() {
            Some(socket) => socket.read_packet(),
            None => Err(SocketError::NotConnected),
        }
    }

    /// Reads incoming packet data with zero-copy and invokes `f` on the slice.
    /// The receive buffer is automatically released when `f` returns.
    pub fn with_incoming_packet<R>(&mut self, f: impl FnOnce(&[u8]) -> R) -> Result<Option<R>, SocketError> {
        let socket = self.socket.as_mut().ok_or(SocketError::NotConnected)?;
        if let Some(packet) = socket.read_packet()? {
            let res = f(&packet);
            drop(packet);
            return Ok(Some(res));
        }
        Ok(None)
    }

    /// Reads incoming data directly into the provided slice buffer.
    pub fn read_into(&mut self, buf: &mut [u8]) -> Result<usize, SocketError> {
        let opt_n = self.with_incoming_packet(|data| {
            let to_copy = data.len().min(buf.len());
            buf[..to_copy].copy_from_slice(&data[..to_copy]);
            to_copy
        })?;
        Ok(opt_n.unwrap_or(0))
    }

    /// Directly echoes all pending incoming data back to the client.
    pub fn echo_incoming(&mut self) -> Result<usize, SocketError> {
        let mut total_echoed = 0;
        let mut buf = [0u8; 128];
        loop {
            let n = self.read_into(&mut buf)?;
            if n == 0 {
                break;
            }
            self.write(&buf[..n])?;
            self.flush()?;
            total_echoed += n;
        }
        Ok(total_echoed)
    }

    /// Writes data to the socket with staging buffer coalescing.
    pub fn write(&mut self, data: &[u8]) -> Result<(), SocketError> {
        if self.write_index + data.len() < WRITE_BUF_SIZE {
            self.write_buffer[self.write_index..self.write_index + data.len()].copy_from_slice(data);
            self.write_index += data.len();
            return Ok(());
        }

        if self.write_index != 0 {
            self.flush()?;
        }

        let socket = self.socket.as_mut().ok_or(SocketError::NotConnected)?;
        forced_write_all(socket, &self.event_group, self.shift, data)
    }

    /// Flushes any pending data from the internal staging buffer.
    pub fn flush(&mut self) -> Result<(), SocketError> {
        if self.write_index == 0 {
            return Ok(());
        }
        let len = self.write_index;
        self.write_index = 0;
        let socket = self.socket.as_mut().ok_or(SocketError::NotConnected)?;
        forced_write_all(socket, &self.event_group, self.shift, &self.write_buffer[..len])
    }

    /// Returns the number of bytes available in the socket TX buffer.
    pub fn write_buffer_available(&mut self) -> usize {
        match self.socket.as_mut() {
            Some(socket) => socket.write_buffer_available(),
            None => 0,
        }
    }

    /// Processes an event group bitmask, driving the socket state machine.
    pub fn process_events(&mut self, mut events: u32) {
        self.handler.on_poll();

        // Strip custom event bit if passed
        let custom_bit = RunnerEvent::CUSTOM_EVENT.shifted(self.shift);
        events &= !custom_bit;

        // 1. Link Up
        if (events & RunnerEvent::UP.0) != 0 {
            events &= !RunnerEvent::UP.0;
            ln_set_connected_state(true);
            self.cleanup();
            let _ = self.start_server();
        }

        // 2. Link Down
        if (events & RunnerEvent::DOWN.0) != 0 {
            ln_set_connected_state(false);
            self.cleanup();
            return;
        }

        // 3. Disconnected
        let disc_bit = RunnerEvent::DISCONNECTED.shifted(self.shift);
        if (events & disc_bit) != 0 {
            events &= !disc_bit;
            self.connected = false;
            self.write_index = 0;
            self.handler.on_disconnected();
            self.disconnect_client();
        }

        // 4. Connected
        let conn_bit = RunnerEvent::CONNECTED.shifted(self.shift);
        if (events & conn_bit) != 0 {
            events &= !conn_bit;
            self.connected = true;
            self.handler.on_connected();
        }

        // 5. Data Available
        let data_bit = RunnerEvent::DATA_AVAILABLE.shifted(self.shift);
        if (events & data_bit) != 0 && self.connected {
            self.handler.on_incoming_data();
        }
    }
}
