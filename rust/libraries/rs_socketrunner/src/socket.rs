//! Abstract socket trait.

use crate::types::{ReadReleaser, RxPacket, SocketError, SocketListener};

/// Abstract socket interface used by `SocketRunner`.
///
/// Implementors represent a streaming TCP server socket that notifies events
/// (connection state, data readiness, TX buffer availability) via `SocketListener`.
pub trait Socket: ReadReleaser + Sized {
    /// Creates and binds a listening socket on `port` with the given event listener.
    fn create(port: u16, listener: SocketListener) -> Result<Self, SocketError>;

    /// Starts accepting incoming connections.
    fn accept(&mut self) -> Result<(), SocketError>;

    /// Disconnects the currently connected client.
    fn disconnect_client(&mut self) -> Result<(), SocketError>;

    /// Reads incoming packet using zero-copy semantics with RAII buffer release.
    fn read_packet(&mut self) -> Result<Option<RxPacket<'_, Self>>, SocketError>;

    /// Writes raw slice data to the socket.
    /// Returns the number of bytes actually written.
    fn write(&mut self, data: &[u8]) -> Result<usize, SocketError>;

    /// Returns the number of bytes available in the socket TX buffer.
    fn write_buffer_available(&mut self) -> usize;

    /// Flushes any pending output in the underlying socket.
    fn flush(&mut self) -> Result<(), SocketError> {
        Ok(())
    }
}
