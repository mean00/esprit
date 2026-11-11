//! Hardware TCP socket management and ring buffer arithmetic.

use crate::bus::W5500Bus;
use crate::registers::*;
pub use rs_socketrunner::{ReadReleaser, RxPacket, Socket, SocketError, SocketEvent, SocketListener};

pub const DEFAULT_RX_BUF_SIZE: usize = 2048;

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
    rx_buf: [u8; DEFAULT_RX_BUF_SIZE],
    rx_len: usize,
}

unsafe impl Send for W5500Socket {}
unsafe impl Sync for W5500Socket {}

impl W5500Socket {
    pub const fn new(sn: u8, port: u16, listener: Option<SocketListener>) -> Self {
        Self {
            sn,
            port,
            state: SocketState::Closed,
            is_sending: false,
            listener,
            rx_buf: [0u8; DEFAULT_RX_BUF_SIZE],
            rx_len: 0,
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

    pub fn set_state(&mut self, state: SocketState) {
        self.state = state;
    }

    pub fn clear_is_sending(&mut self) {
        self.is_sending = false;
    }

    pub fn invoke(&mut self, evt: SocketEvent) {
        if let Some(ref l) = self.listener {
            l.notify(evt);
        }
    }

    /// Issue a command to Sn_CR and busy-wait until accepted (register clears).
    pub fn send_command(&mut self, bus: &mut W5500Bus, cmd: u8) {
        bus.write_u8(Sn_CR, sreg_block(self.sn), cmd);
        while bus.read_u8(Sn_CR, sreg_block(self.sn)) != 0 {
            core::hint::spin_loop();
        }
    }

    /// Open socket in TCP server mode and start listening on `port`.
    pub fn init(&mut self, bus: &mut W5500Bus) -> bool {
        self.close(bus);

        bus.write_u8(Sn_MR, sreg_block(self.sn), Sn_MR_TCP);
        bus.write_u16(Sn_PORT, sreg_block(self.sn), self.port);
        self.send_command(bus, Sn_CR_OPEN);

        if self.status(bus) != Ok(SocketStatus::Init) {
            self.state = SocketState::Error;
            return false;
        }

        self.send_command(bus, Sn_CR_LISTEN);

        if self.status(bus) != Ok(SocketStatus::Listen) {
            self.state = SocketState::Error;
            return false;
        }

        // Enable interrupts for this socket
        let imr = Sn_IR_CON | Sn_IR_DISCON | Sn_IR_RECV | Sn_IR_TIMEOUT | Sn_IR_SENDOK;
        bus.write_u8(Sn_IMR, sreg_block(self.sn), imr);

        self.state = SocketState::Listening;
        self.is_sending = false;
        true
    }

    /// Close the socket.
    pub fn close(&mut self, bus: &mut W5500Bus) {
        self.send_command(bus, Sn_CR_CLOSE);
        bus.write_u8(Sn_IR, sreg_block(self.sn), 0xFF);
        self.state = SocketState::Closed;
        self.is_sending = false;
        self.rx_len = 0;
    }

    /// Disconnect connected peer (sends TCP FIN), closes socket and restarts listening.
    pub fn disconnect(&mut self, bus: &mut W5500Bus) {
        self.handle_disconnect(bus);
    }

    /// Handles disconnect event (local or remote), cleans up socket and re-listens.
    pub fn handle_disconnect(&mut self, bus: &mut W5500Bus) {
        if self.state == SocketState::Connected || self.state == SocketState::Closing {
            self.state = SocketState::Closing;
            if let Ok(s) = self.status(bus) {
                if s == SocketStatus::Established || s == SocketStatus::CloseWait {
                    self.send_command(bus, Sn_CR_DISCON);
                }
            }
            self.close(bus);
            self.init(bus);
            self.invoke(SocketEvent::Disconnect);
        }
    }

    /// Read incoming bytes into the internal RX buffer.
    /// Returns slice pointing to received data.
    pub fn read<'a>(&'a mut self, bus: &mut W5500Bus) -> Option<&'a [u8]> {
        if self.state != SocketState::Connected {
            return None;
        }

        let rsr = bus.read_stable_u16(Sn_RX_RSR, sreg_block(self.sn));
        if rsr == 0 {
            return None;
        }

        let to_read = (rsr as usize).min(self.rx_buf.len());
        let ptr = bus.read_stable_u16(Sn_RX_RD, sreg_block(self.sn));

        bus.read_reg(ptr, rxbuf_block(self.sn), &mut self.rx_buf[..to_read]);
        bus.write_u16(Sn_RX_RD, sreg_block(self.sn), ptr.wrapping_add(to_read as u16));
        self.send_command(bus, Sn_CR_RECV);

        self.rx_len = to_read;
        Some(&self.rx_buf[..self.rx_len])
    }

    /// Clear internal RX buffer tracking.
    pub fn free_read_data(&mut self) {
        self.rx_len = 0;
    }

    /// Write bytes to the socket TX buffer and issue Sn_CR_SEND.
    /// Returns number of bytes sent, 0 if would block, or Err(()) on failure.
    pub fn write(&mut self, bus: &mut W5500Bus, data: &[u8]) -> Result<usize, ()> {
        if self.state != SocketState::Connected {
            return Err(());
        }

        let mut had_sendok = false;
        if self.is_sending {
            let ir = bus.read_u8(Sn_IR, sreg_block(self.sn));
            if (ir & Sn_IR_SENDOK) != 0 {
                had_sendok = true;
                self.is_sending = false;
            } else if (ir & Sn_IR_TIMEOUT) != 0 {
                self.close(bus);
                return Err(());
            } else {
                return Ok(0); // Still sending previous packet
            }
        }

        let freesize = bus.read_stable_u16(Sn_TX_FSR, sreg_block(self.sn)) as usize;
        let to_send = data.len().min(freesize);
        if to_send == 0 {
            return Ok(0);
        }

        if had_sendok {
            bus.write_u8(Sn_IR, sreg_block(self.sn), Sn_IR_SENDOK);
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
                        self.init(bus);
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
                self.init(bus);
            }
        }
    }
}

// ---------------------------------------------------------------------------
//  W5500SocketHandle — safe RAII socket implementing rs_socketrunner::Socket
// ---------------------------------------------------------------------------

/// Safe, zero-copy RAII socket handle implementing [`rs_socketrunner::Socket`].
pub struct W5500SocketHandle {
    sn: u8,
}

unsafe impl Send for W5500SocketHandle {}
unsafe impl Sync for W5500SocketHandle {}

impl W5500SocketHandle {
    #[inline(always)]
    pub fn socket_num(&self) -> u8 {
        self.sn
    }
}

impl ReadReleaser for W5500SocketHandle {
    fn release_read_data(&mut self) {
        let mut sockets_guard = crate::coordinator::SOCKETS.lock();
        if let Some(sock) = sockets_guard[self.sn as usize].as_mut() {
            sock.free_read_data();
        }
    }
}

impl Socket for W5500SocketHandle {
    fn create(port: u16, listener: SocketListener) -> Result<Self, SocketError> {
        let mut sockets_guard = crate::coordinator::SOCKETS.lock();

        let mut slot_idx = None;
        for (i, slot) in sockets_guard.iter().enumerate() {
            if slot.is_none() {
                slot_idx = Some(i);
                break;
            }
        }

        let sn = match slot_idx {
            Some(idx) => idx as u8,
            None => return Err(SocketError::CreationFailed),
        };

        let mut sock = W5500Socket::new(sn, port, Some(listener));

        let mut dev_guard = crate::coordinator::DEVICE.lock();
        let dev = dev_guard.as_mut().ok_or(SocketError::CreationFailed)?;
        if !sock.init(&mut dev.bus) {
            return Err(SocketError::CreationFailed);
        }

        sockets_guard[sn as usize] = Some(sock);

        Ok(Self { sn })
    }

    fn accept(&mut self) -> Result<(), SocketError> {
        let mut sockets_guard = crate::coordinator::SOCKETS.lock();
        let sock = sockets_guard[self.sn as usize]
            .as_mut()
            .ok_or(SocketError::AcceptFailed)?;

        let mut dev_guard = crate::coordinator::DEVICE.lock();
        let dev = dev_guard.as_mut().ok_or(SocketError::AcceptFailed)?;
        if sock.init(&mut dev.bus) {
            Ok(())
        } else {
            Err(SocketError::AcceptFailed)
        }
    }

    fn disconnect_client(&mut self) -> Result<(), SocketError> {
        crate::coordinator::trigger_coordinator_disconnect(self.sn);
        Ok(())
    }

    fn read_packet(&mut self) -> Result<Option<RxPacket<'_, Self>>, SocketError> {
        let mut sockets_guard = crate::coordinator::SOCKETS.lock();
        let sock = sockets_guard[self.sn as usize]
            .as_mut()
            .ok_or(SocketError::ReadFailed)?;

        let mut dev_guard = crate::coordinator::DEVICE.lock();
        let dev = dev_guard.as_mut().ok_or(SocketError::ReadFailed)?;

        let slice = match sock.read(&mut dev.bus) {
            Some(s) => unsafe { core::slice::from_raw_parts(s.as_ptr(), s.len()) },
            None => return Ok(None),
        };

        Ok(Some(RxPacket::new(self, slice)))
    }

    fn write(&mut self, data: &[u8]) -> Result<usize, SocketError> {
        let mut sockets_guard = crate::coordinator::SOCKETS.lock();
        let sock = sockets_guard[self.sn as usize]
            .as_mut()
            .ok_or(SocketError::WriteFailed)?;

        let mut dev_guard = crate::coordinator::DEVICE.lock();
        let dev = dev_guard.as_mut().ok_or(SocketError::WriteFailed)?;

        match sock.write(&mut dev.bus, data) {
            Ok(n) => Ok(n),
            Err(_) => Err(SocketError::WriteFailed),
        }
    }

    fn write_buffer_available(&mut self) -> usize {
        let mut sockets_guard = crate::coordinator::SOCKETS.lock();
        if let Some(sock) = sockets_guard[self.sn as usize].as_mut() {
            let mut dev_guard = crate::coordinator::DEVICE.lock();
            if let Some(dev) = dev_guard.as_mut() {
                return sock.write_buffer_available(&mut dev.bus);
            }
        }
        0
    }

    fn flush(&mut self) -> Result<(), SocketError> {
        Ok(())
    }
}

impl Drop for W5500SocketHandle {
    fn drop(&mut self) {
        let mut sockets_guard = crate::coordinator::SOCKETS.lock();
        if let Some(mut sock) = sockets_guard[self.sn as usize].take() {
            let mut dev_guard = crate::coordinator::DEVICE.lock();
            if let Some(dev) = dev_guard.as_mut() {
                sock.close(&mut dev.bus);
            }
        }
    }
}
