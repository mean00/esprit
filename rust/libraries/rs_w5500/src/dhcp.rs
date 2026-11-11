//! Embedded DHCP client running on hardware UDP Socket 7.

use crate::bus::W5500Bus;
use crate::registers::*;

pub const DHCP_SOCKET: u8 = 7;
pub const DHCP_SERVER_PORT: u16 = 67;
pub const DHCP_CLIENT_PORT: u16 = 68;
pub const MAGIC_COOKIE: [u8; 4] = [0x63, 0x82, 0x53, 0x63];

const DHCP_DISCOVER: u8 = 1;
const DHCP_OFFER: u8 = 2;
const DHCP_REQUEST: u8 = 3;
const DHCP_ACK: u8 = 5;
const DHCP_NAK: u8 = 6;

const DHCP_OPT_SUBNET_MASK: u8 = 1;
const DHCP_OPT_ROUTER: u8 = 3;
const DHCP_OPT_DNS: u8 = 6;
const DHCP_OPT_HOST_NAME: u8 = 12;
const DHCP_OPT_REQUESTED_IP: u8 = 50;
const DHCP_OPT_LEASE_TIME: u8 = 51;
const DHCP_OPT_MSG_TYPE: u8 = 53;
const DHCP_OPT_SERVER_ID: u8 = 54;
const DHCP_OPT_PARAM_REQ: u8 = 55;
const DHCP_OPT_END: u8 = 255;

const BOOTP_MAGIC_OFFSET: usize = 236;
const BOOTP_OPTIONS_OFFSET: usize = 240;

const BROADCAST_IP: [u8; 4] = [255, 255, 255, 255];
const BROADCAST_MAC: [u8; 6] = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DhcpState {
    Idle,
    Discovering,
    Requesting,
    Bound,
    Failed,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct DhcpInfo {
    pub ip: [u8; 4],
    pub subnet: [u8; 4],
    pub gateway: [u8; 4],
    pub dns: [u8; 4],
    pub lease_time: u32,
}

pub struct DhcpClient {
    state: DhcpState,
    xid: u32,
    server_ip: [u8; 4],
    requested_ip: [u8; 4],
    pub info: DhcpInfo,
    buf: [u8; 600],
    retry_count: u8,
    tick_count: u32,
}

impl DhcpClient {
    pub const fn new() -> Self {
        Self {
            state: DhcpState::Idle,
            xid: 0x12345678,
            server_ip: [0; 4],
            requested_ip: [0; 4],
            info: DhcpInfo {
                ip: [0; 4],
                subnet: [0; 4],
                gateway: [0; 4],
                dns: [0; 4],
                lease_time: 0,
            },
            buf: [0; 600],
            retry_count: 0,
            tick_count: 0,
        }
    }

    pub fn state(&self) -> DhcpState {
        self.state
    }

    pub fn init(&mut self, bus: &mut W5500Bus, mac: &[u8; 6]) {
        self.state = DhcpState::Idle;
        self.retry_count = 0;
        self.tick_count = 0;
        // Derive initial XID from MAC address
        self.xid = 0x55000000
            | ((mac[2] as u32) << 16)
            | ((mac[4] as u32) << 8)
            | (mac[5] as u32);

        self.open_socket(bus);
    }

    fn send_command(&mut self, bus: &mut W5500Bus, cmd: u8) {
        bus.write_u8(Sn_CR, sreg_block(DHCP_SOCKET), cmd);
        while bus.read_u8(Sn_CR, sreg_block(DHCP_SOCKET)) != 0 {
            core::hint::spin_loop();
        }
    }

    fn open_socket(&mut self, bus: &mut W5500Bus) {
        self.send_command(bus, Sn_CR_CLOSE);
        bus.write_u8(Sn_IR, sreg_block(DHCP_SOCKET), 0xFF);

        bus.write_u8(Sn_MR, sreg_block(DHCP_SOCKET), Sn_MR_UDP);
        bus.write_u16(Sn_PORT, sreg_block(DHCP_SOCKET), DHCP_CLIENT_PORT);
        self.send_command(bus, Sn_CR_OPEN);
    }

    fn send_udp(&mut self, bus: &mut W5500Bus, len: usize, dst_ip: &[u8; 4], dst_port: u16) -> bool {
        bus.write_reg(Sn_DIPR, sreg_block(DHCP_SOCKET), dst_ip);
        bus.write_u16(Sn_DPORT, sreg_block(DHCP_SOCKET), dst_port);
        bus.write_reg(Sn_DHAR, sreg_block(DHCP_SOCKET), &BROADCAST_MAC);

        let free = bus.read_stable_u16(Sn_TX_FSR, sreg_block(DHCP_SOCKET)) as usize;
        if free < len {
            return false;
        }

        let ptr = bus.read_stable_u16(Sn_TX_WR, sreg_block(DHCP_SOCKET));
        bus.write_reg(ptr, txbuf_block(DHCP_SOCKET), &self.buf[..len]);
        bus.write_u16(Sn_TX_WR, sreg_block(DHCP_SOCKET), ptr.wrapping_add(len as u16));

        self.send_command(bus, Sn_CR_SEND);

        // Wait for SEND_OK
        for _ in 0..10_000 {
            let ir = bus.read_u8(Sn_IR, sreg_block(DHCP_SOCKET));
            if (ir & Sn_IR_SENDOK) != 0 {
                bus.write_u8(Sn_IR, sreg_block(DHCP_SOCKET), Sn_IR_SENDOK);
                return true;
            }
            if (ir & Sn_IR_TIMEOUT) != 0 {
                bus.write_u8(Sn_IR, sreg_block(DHCP_SOCKET), Sn_IR_TIMEOUT);
                return false;
            }
        }
        false
    }

    fn recv_udp(&mut self, bus: &mut W5500Bus) -> Option<usize> {
        let rsr = bus.read_stable_u16(Sn_RX_RSR, sreg_block(DHCP_SOCKET));
        if rsr < 8 {
            return None;
        }

        // Read 8-byte UDP header
        let mut hdr = [0u8; 8];
        let mut ptr = bus.read_stable_u16(Sn_RX_RD, sreg_block(DHCP_SOCKET));
        bus.read_reg(ptr, rxbuf_block(DHCP_SOCKET), &mut hdr);
        ptr = ptr.wrapping_add(8);
        bus.write_u16(Sn_RX_RD, sreg_block(DHCP_SOCKET), ptr);
        self.send_command(bus, Sn_CR_RECV);

        let pack_len = (((hdr[6] as u16) << 8) | (hdr[7] as u16)) as usize;
        let payload_len = if pack_len > 0 { pack_len } else { (rsr - 8) as usize };
        if payload_len == 0 {
            return None;
        }

        let to_read = payload_len.min(self.buf.len());
        ptr = bus.read_stable_u16(Sn_RX_RD, sreg_block(DHCP_SOCKET));
        bus.read_reg(ptr, rxbuf_block(DHCP_SOCKET), &mut self.buf[..to_read]);
        ptr = ptr.wrapping_add(payload_len as u16);
        bus.write_u16(Sn_RX_RD, sreg_block(DHCP_SOCKET), ptr);
        self.send_command(bus, Sn_CR_RECV);

        Some(to_read)
    }

    fn build_header(&mut self, mac: &[u8; 6], msg_type: u8) -> usize {
        self.buf.fill(0);
        self.buf[0] = 1; // BOOTREQUEST
        self.buf[1] = 1; // Ethernet
        self.buf[2] = 6; // MAC length
        self.buf[3] = 0;

        self.buf[4] = (self.xid >> 24) as u8;
        self.buf[5] = (self.xid >> 16) as u8;
        self.buf[6] = (self.xid >> 8) as u8;
        self.buf[7] = self.xid as u8;

        self.buf[28..34].copy_from_slice(mac);

        // Magic cookie
        self.buf[BOOTP_MAGIC_OFFSET..BOOTP_MAGIC_OFFSET + 4].copy_from_slice(&MAGIC_COOKIE);

        let mut off = BOOTP_OPTIONS_OFFSET;
        self.buf[off] = DHCP_OPT_MSG_TYPE;
        self.buf[off + 1] = 1;
        self.buf[off + 2] = msg_type;
        off += 3;

        off
    }

    fn send_discover(&mut self, bus: &mut W5500Bus, mac: &[u8; 6]) {
        self.xid = self.xid.wrapping_add(1);
        let mut off = self.build_header(mac, DHCP_DISCOVER);

        // Option: parameter request list (subnet, router, dns, lease)
        self.buf[off] = DHCP_OPT_PARAM_REQ;
        self.buf[off + 1] = 4;
        self.buf[off + 2] = DHCP_OPT_SUBNET_MASK;
        self.buf[off + 3] = DHCP_OPT_ROUTER;
        self.buf[off + 4] = DHCP_OPT_DNS;
        self.buf[off + 5] = DHCP_OPT_LEASE_TIME;
        off += 6;

        // Option: host name
        let hostname = b"swindle";
        self.buf[off] = DHCP_OPT_HOST_NAME;
        self.buf[off + 1] = hostname.len() as u8;
        self.buf[off + 2..off + 2 + hostname.len()].copy_from_slice(hostname);
        off += 2 + hostname.len();

        self.buf[off] = DHCP_OPT_END;
        off += 1;

        self.send_udp(bus, off, &BROADCAST_IP, DHCP_SERVER_PORT);
        self.state = DhcpState::Discovering;
    }

    fn send_request(&mut self, bus: &mut W5500Bus, mac: &[u8; 6]) {
        let mut off = self.build_header(mac, DHCP_REQUEST);

        // Requested IP
        self.buf[off] = DHCP_OPT_REQUESTED_IP;
        self.buf[off + 1] = 4;
        self.buf[off + 2..off + 6].copy_from_slice(&self.requested_ip);
        off += 6;

        // Server ID
        self.buf[off] = DHCP_OPT_SERVER_ID;
        self.buf[off + 1] = 4;
        self.buf[off + 2..off + 6].copy_from_slice(&self.server_ip);
        off += 6;

        // Parameter request list
        self.buf[off] = DHCP_OPT_PARAM_REQ;
        self.buf[off + 1] = 4;
        self.buf[off + 2] = DHCP_OPT_SUBNET_MASK;
        self.buf[off + 3] = DHCP_OPT_ROUTER;
        self.buf[off + 4] = DHCP_OPT_DNS;
        self.buf[off + 5] = DHCP_OPT_LEASE_TIME;
        off += 6;

        self.buf[off] = DHCP_OPT_END;
        off += 1;

        self.send_udp(bus, off, &BROADCAST_IP, DHCP_SERVER_PORT);
        self.state = DhcpState::Requesting;
    }

    /// Step the DHCP state machine. Should be called periodically until Bound or Failed.
    pub fn poll(&mut self, bus: &mut W5500Bus, mac: &[u8; 6]) -> DhcpState {
        match self.state {
            DhcpState::Idle => {
                self.send_discover(bus, mac);
            }
            DhcpState::Discovering => {
                if let Some(len) = self.recv_udp(bus) {
                    if self.parse_reply(len, DHCP_OFFER) {
                        self.send_request(bus, mac);
                    }
                }
            }
            DhcpState::Requesting => {
                if let Some(len) = self.recv_udp(bus) {
                    if self.parse_reply(len, DHCP_ACK) {
                        self.state = DhcpState::Bound;
                    }
                }
            }
            DhcpState::Bound | DhcpState::Failed => {}
        }
        self.state
    }

    /// Periodic 1-second tick for retries and lease timeouts.
    pub fn tick(&mut self, bus: &mut W5500Bus, mac: &[u8; 6]) {
        self.tick_count = self.tick_count.wrapping_add(1);
        if self.state == DhcpState::Discovering || self.state == DhcpState::Requesting {
            self.retry_count += 1;
            if self.retry_count >= 5 {
                self.retry_count = 0;
                // Retry discover
                self.open_socket(bus);
                self.send_discover(bus, mac);
            }
        }
    }

    fn parse_reply(&mut self, len: usize, expected_type: u8) -> bool {
        if len < 240 {
            return false;
        }

        // Validate magic cookie
        if self.buf[BOOTP_MAGIC_OFFSET..BOOTP_MAGIC_OFFSET + 4] != MAGIC_COOKIE {
            return false;
        }

        // Validate XID
        let rx_xid = ((self.buf[4] as u32) << 24)
            | ((self.buf[5] as u32) << 16)
            | ((self.buf[6] as u32) << 8)
            | (self.buf[7] as u32);
        if rx_xid != self.xid {
            return false;
        }

        let mut msg_type = 0u8;
        let mut offered_ip = [0u8; 4];
        offered_ip.copy_from_slice(&self.buf[16..20]);

        let mut off = BOOTP_OPTIONS_OFFSET;
        while off < len {
            let tag = self.buf[off];
            if tag == DHCP_OPT_END {
                break;
            }
            if tag == 0 {
                off += 1;
                continue;
            }
            if off + 1 >= len {
                break;
            }
            let opt_len = self.buf[off + 1] as usize;
            off += 2;
            if off + opt_len > len {
                break;
            }

            match tag {
                DHCP_OPT_MSG_TYPE if opt_len >= 1 => {
                    msg_type = self.buf[off];
                }
                DHCP_OPT_SERVER_ID if opt_len >= 4 => {
                    self.server_ip.copy_from_slice(&self.buf[off..off + 4]);
                }
                DHCP_OPT_SUBNET_MASK if opt_len >= 4 => {
                    self.info.subnet.copy_from_slice(&self.buf[off..off + 4]);
                }
                DHCP_OPT_ROUTER if opt_len >= 4 => {
                    self.info.gateway.copy_from_slice(&self.buf[off..off + 4]);
                }
                DHCP_OPT_DNS if opt_len >= 4 => {
                    self.info.dns.copy_from_slice(&self.buf[off..off + 4]);
                }
                DHCP_OPT_LEASE_TIME if opt_len >= 4 => {
                    self.info.lease_time = ((self.buf[off] as u32) << 24)
                        | ((self.buf[off + 1] as u32) << 16)
                        | ((self.buf[off + 2] as u32) << 8)
                        | (self.buf[off + 3] as u32);
                }
                _ => {}
            }
            off += opt_len;
        }

        if msg_type == expected_type {
            if expected_type == DHCP_OFFER {
                self.requested_ip = offered_ip;
            } else if expected_type == DHCP_ACK {
                self.info.ip = offered_ip;
            }
            true
        } else {
            false
        }
    }
}
