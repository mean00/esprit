//! Hardware TCP socket management and ring buffer arithmetic.

use crate::bus::W5500Bus;
use crate::coordinator::MAX_SOCKETS;
use crate::registers::*;
pub use rs_socketrunner::{Socket, SocketError, SocketEvent, SocketListener};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SocketState {
    Closed,
    Listening,
    Connected,
    Closing,
    Error,
}

/// Manages a single W5500 hardware TCP socket instance.
pub struct W5500Socket {
    sn: u8,
    port: u16,
    state: SocketState,
    is_sending: bool,
    listener: Option<SocketListener>,
}

const CMD_POLL_TRIES: usize = 10_000;

impl W5500Socket {
    pub const fn new(sn: u8, port: u16, listener: Option<SocketListener>) -> Self {
        Self {
            sn,
            port,
            state: SocketState::Closed,
            is_sending: false,
            listener,
        }
    }

    #[inline(always)]
    pub fn socket_num(&self) -> u8 {
        self.sn
    }

    #[inline(always)]
    pub fn state(&self) -> SocketState {
        self.state
    }

    /// Read hardware socket status register (Sn_SR).
    #[inline(always)]
    pub fn status(&self, bus: &mut W5500Bus) -> Result<SocketStatus, u8> {
        SocketStatus::try_from(bus.read_u8(Sn_SR, sreg_block(self.sn)))
    }

    pub fn clear_is_sending(&mut self) {
        self.is_sending = false;
    }

    pub fn invoke(&mut self, evt: SocketEvent) {
        if let Some(ref l) = self.listener {
            l.notify(evt);
        }
    }

    /// Issue a command to Sn_CR and wait until accepted (register clears) or timeout.
    pub fn send_command(&mut self, bus: &mut W5500Bus, cmd: u8) {
        bus.write_u8(Sn_CR, sreg_block(self.sn), cmd);
        for _ in 0..CMD_POLL_TRIES {
            if bus.read_u8(Sn_CR, sreg_block(self.sn)) == 0 {
                return;
            }
            core::hint::spin_loop();
        }
    }

    /// Open socket in TCP server mode and start listening on `port`.
    pub fn init(&mut self, bus: &mut W5500Bus) -> Result<(), SocketError> {
        self.close(bus);

        bus.write_u8(Sn_MR, sreg_block(self.sn), Sn_MR_TCP);
        bus.write_u16(Sn_PORT, sreg_block(self.sn), self.port);
        self.send_command(bus, Sn_CR_OPEN);

        if self.status(bus) != Ok(SocketStatus::Init) {
            self.state = SocketState::Error;
            return Err(SocketError::CreationFailed);
        }

        self.send_command(bus, Sn_CR_LISTEN);

        if self.status(bus) != Ok(SocketStatus::Listen) {
            self.state = SocketState::Error;
            return Err(SocketError::CreationFailed);
        }

        // Enable interrupts for this socket
        let imr = Sn_IR_CON | Sn_IR_DISCON | Sn_IR_RECV | Sn_IR_TIMEOUT | Sn_IR_SENDOK;
        bus.write_u8(Sn_IMR, sreg_block(self.sn), imr);

        self.state = SocketState::Listening;
        self.is_sending = false;
        Ok(())
    }

    /// Close the socket.
    pub fn close(&mut self, bus: &mut W5500Bus) {
        self.send_command(bus, Sn_CR_CLOSE);
        bus.write_u8(Sn_IR, sreg_block(self.sn), 0xFF);
        self.state = SocketState::Closed;
        self.is_sending = false;
    }

    /// Handles disconnect event (local or remote), cleans up socket and re-listens.
    pub fn handle_disconnect(&mut self, bus: &mut W5500Bus) {
        if self.state == SocketState::Connected || self.state == SocketState::Closing {
            self.state = SocketState::Closing;
            self.close(bus);
            let _ = self.init(bus);
            self.invoke(SocketEvent::Disconnect);
        }
    }

    /// Read pending bytes straight into `buf`, returning how many were read.
    ///
    /// The SPI transfer writes the caller's buffer: there is no intermediate
    /// socket buffer, hence nothing to copy out afterwards.
    pub fn read(&mut self, bus: &mut W5500Bus, buf: &mut [u8]) -> Option<usize> {
        if self.state != SocketState::Connected {
            return None;
        }

        let rsr = bus.read_stable_u16(Sn_RX_RSR, sreg_block(self.sn));
        if rsr == 0 {
            return None;
        }

        let to_read = (rsr as usize).min(buf.len());
        if to_read == 0 {
            return None;
        }

        let ptr = bus.read_stable_u16(Sn_RX_RD, sreg_block(self.sn));

        bus.read_reg(ptr, rxbuf_block(self.sn), &mut buf[..to_read]);
        bus.write_u16(Sn_RX_RD, sreg_block(self.sn), ptr.wrapping_add(to_read as u16));
        self.send_command(bus, Sn_CR_RECV);

        Some(to_read)
    }

    /// Write bytes to the socket TX buffer and issue Sn_CR_SEND.
    /// Returns number of bytes sent, 0 if it would block.
    pub fn write(&mut self, bus: &mut W5500Bus, data: &[u8]) -> Result<usize, SocketError> {
        if self.state != SocketState::Connected {
            return Err(SocketError::WriteFailed);
        }

        if self.is_sending {
            let ir = bus.read_u8(Sn_IR, sreg_block(self.sn));
            if (ir & Sn_IR_SENDOK) != 0 {
                bus.write_u8(Sn_IR, sreg_block(self.sn), Sn_IR_SENDOK);
                self.is_sending = false;
            } else if (ir & Sn_IR_TIMEOUT) != 0 {
                self.handle_disconnect(bus);
                return Err(SocketError::WriteFailed);
            } else {
                return Ok(0); // Still sending previous packet
            }
        }

        let freesize = bus.read_stable_u16(Sn_TX_FSR, sreg_block(self.sn)) as usize;
        let to_send = data.len().min(freesize);
        if to_send == 0 {
            return Ok(0);
        }

        let ptr = bus.read_stable_u16(Sn_TX_WR, sreg_block(self.sn));
        bus.write_reg(ptr, txbuf_block(self.sn), &data[..to_send]);
        bus.write_u16(Sn_TX_WR, sreg_block(self.sn), ptr.wrapping_add(to_send as u16));

        self.is_sending = true;
        self.send_command(bus, Sn_CR_SEND);

        Ok(to_send)
    }

    /// Available space in TX buffer.
    pub fn write_buffer_available(&mut self, bus: &mut W5500Bus) -> usize {
        if self.state != SocketState::Connected {
            return 0;
        }
        bus.read_stable_u16(Sn_TX_FSR, sreg_block(self.sn)) as usize
    }

    /// Check status register and drive connection / disconnect state machine.
    pub fn poll(&mut self, bus: &mut W5500Bus) {
        let status = match self.status(bus) {
            Ok(s) => s,
            Err(_) => return,
        };

        match self.state {
            SocketState::Listening => {
                match status {
                    SocketStatus::Established => {
                        self.state = SocketState::Connected;
                        self.invoke(SocketEvent::Connected);
                    }
                    SocketStatus::Closed => {
                        let _ = self.init(bus);
                    }
                    _ => {}
                }
            }
            SocketState::Connected => {
                if status == SocketStatus::Closed || status == SocketStatus::CloseWait {
                    self.handle_disconnect(bus);
                }
            }
            SocketState::Closing | SocketState::Closed | SocketState::Error => {
                self.close(bus);
                let _ = self.init(bus);
            }
        }
    }
}

// ---------------------------------------------------------------------------
//  W5500SocketHandle — safe RAII socket implementing rs_socketrunner::Socket
// ---------------------------------------------------------------------------

/// RAII socket handle implementing [`rs_socketrunner::Socket`].
pub struct W5500SocketHandle {
    sn: u8,
}

impl W5500SocketHandle {
    #[inline(always)]
    pub fn socket_num(&self) -> u8 {
        self.sn
    }
}

impl Socket for W5500SocketHandle {
    fn create(port: u16, listener: SocketListener) -> Result<Self, SocketError> {
        let net = &mut *crate::coordinator::NET.lock();

        let sn = (0..MAX_SOCKETS)
            .find(|&slot| net.sockets[slot].is_none())
            .ok_or(SocketError::CreationFailed)? as u8;

        let mut sock = W5500Socket::new(sn, port, Some(listener));

        let dev = net
            .device
            .as_mut()
            .ok_or(SocketError::CreationFailed)?;
        sock.init(&mut dev.bus)?;

        net.sockets[sn as usize] = Some(sock);

        Ok(Self { sn })
    }

    fn accept(&mut self) -> Result<(), SocketError> {
        let net = &mut *crate::coordinator::NET.lock();

        let dev = net.device.as_mut().ok_or(SocketError::AcceptFailed)?;
        let sock = net.sockets[self.sn as usize]
            .as_mut()
            .ok_or(SocketError::AcceptFailed)?;

        sock.init(&mut dev.bus)
            .map_err(|_| SocketError::AcceptFailed)
    }

    fn disconnect_client(&mut self) -> Result<(), SocketError> {
        crate::coordinator::trigger_coordinator_disconnect(self.sn);
        Ok(())
    }

    fn read_packet(&mut self, buf: &mut [u8]) -> Result<Option<usize>, SocketError> {
        let net = &mut *crate::coordinator::NET.lock();

        let dev = net.device.as_mut().ok_or(SocketError::ReadFailed)?;
        let sock = net.sockets[self.sn as usize]
            .as_mut()
            .ok_or(SocketError::ReadFailed)?;

        Ok(sock.read(&mut dev.bus, buf))
    }

    fn write(&mut self, data: &[u8]) -> Result<usize, SocketError> {
        let net = &mut *crate::coordinator::NET.lock();

        let dev = net.device.as_mut().ok_or(SocketError::WriteFailed)?;
        let sock = net.sockets[self.sn as usize]
            .as_mut()
            .ok_or(SocketError::WriteFailed)?;

        sock.write(&mut dev.bus, data)
    }

    fn write_buffer_available(&mut self) -> usize {
        let net = &mut *crate::coordinator::NET.lock();

        let Some(dev) = net.device.as_mut() else {
            return 0;
        };
        match net.sockets[self.sn as usize].as_mut() {
            Some(sock) => sock.write_buffer_available(&mut dev.bus),
            None => 0,
        }
    }

    fn flush(&mut self) -> Result<(), SocketError> {
        Ok(())
    }
}

impl Drop for W5500SocketHandle {
    fn drop(&mut self) {
        let net = &mut *crate::coordinator::NET.lock();
        let Some(mut sock) = net.sockets[self.sn as usize].take() else {
            return;
        };
        if let Some(dev) = net.device.as_mut() {
            sock.close(&mut dev.bus);
        }
    }
}
