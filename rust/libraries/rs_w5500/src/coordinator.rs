//! FreeRTOS coordinator task for PHY state, DHCP client, and socket interrupt dispatch.

use crate::device::W5500Device;
use crate::dhcp::{DhcpClient, DhcpState};
use crate::registers::*;
use crate::socket::{SocketEvent, W5500Socket};
use core::ffi::c_void;
use rs_rtos::event::EventGroup;
use rs_rtos::sync::{Mutex, OnceLock};
use rs_rtos::task::delay_ms;
use rust_esprit::{
    attach_interrupt, disable_interrupt, enable_interrupt, Edge, Pin, PinCallback,
};

pub const EVT_DISCON_SOCK0: u32 = discon_bit(0);
pub const EVT_DISCON_SOCK1: u32 = discon_bit(1);
pub const EVT_DISCON_SOCK2: u32 = discon_bit(2);
pub const EVT_DISCON_SOCK3: u32 = discon_bit(3);
pub const EVT_DISCON_MASK: u32 =
    EVT_DISCON_SOCK0 | EVT_DISCON_SOCK1 | EVT_DISCON_SOCK2 | EVT_DISCON_SOCK3;
pub const EVT_INTR: u32 = 1 << 28;

pub const MAX_SOCKETS: usize = 4;

/// Event bit asking the coordinator to disconnect socket `sn` (bits 1..=4).
const fn discon_bit(sn: u8) -> u32 {
    1 << (1 + sn as u32)
}

/// Event-group wait timeout, in ms.
const WAIT_TIMEOUT_MS: i32 = 50;
/// One DHCP tick every N event-group wake-ups.
const DHCP_TICK_DIVIDER: u32 = 20;
/// Link-up wait at boot: `LINK_WAIT_TRIES` polls, `LINK_POLL_MS` apart.
const LINK_WAIT_TRIES: usize = 60;
const LINK_POLL_MS: u32 = 50;
/// Delay before re-checking a still-asserted interrupt line.
const INTR_RETRIGGER_MS: u32 = 1;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CoordinatorState {
    Idle,
    Dhcping,
    Running,
}

pub struct Coordinator {
    state: CoordinatorState,
    event_group: EventGroup,
    mac: [u8; 6],
    net_cb: Option<extern "C" fn(u32, *mut c_void)>,
    net_arg: *mut c_void,
    dhcp: DhcpClient,
}

// SAFETY: `net_arg` is an opaque pointer the board setup stores and the
// callback receives; it is only ever passed through, never dereferenced here.
unsafe impl Send for Coordinator {}

impl Coordinator {
    pub const fn new() -> Self {
        Self {
            state: CoordinatorState::Idle,
            event_group: EventGroup::uninit(),
            mac: [0; 6],
            net_cb: None,
            net_arg: core::ptr::null_mut(),
            dhcp: DhcpClient::new(),
        }
    }

    /// MAC address and network-ready callback handed over by the board setup.
    pub fn configure(
        &mut self,
        mac: [u8; 6],
        cb: Option<extern "C" fn(u32, *mut c_void)>,
        arg: *mut c_void,
    ) {
        self.mac = mac;
        self.net_cb = cb;
        self.net_arg = arg;
    }
}

pub static COORDINATOR: Mutex<Coordinator> = Mutex::new(Coordinator::new());

/// The controller and its sockets, behind one lock.
///
/// One lock for both: every user needs the device (`Sn_*` registers live there)
/// and the socket together, and with two locks each caller has to take them in
/// the same order — a rule the coordinator and the socket API already broke.
pub struct NetState {
    pub device: Option<W5500Device>,
    pub sockets: [Option<W5500Socket>; MAX_SOCKETS],
}

// `[None; MAX_SOCKETS]` would need `W5500Socket: Copy`, so the array is spelled out.
const _: () = assert!(MAX_SOCKETS == 4);

impl NetState {
    pub const fn new() -> Self {
        Self {
            device: None,
            sockets: [None, None, None, None],
        }
    }
}

pub static NET: Mutex<NetState> = Mutex::new(NetState::new());

/// Lock-free handle to the coordinator event group.
///
/// Published by [`w5500_coordinator_task`] before the W5500 interrupt line is
/// armed, so the EXTI handler can signal an event without locking
/// [`COORDINATOR`]: that lock is a kernel mutex and must never be taken from an
/// interrupt. `EventGroup::set_events` is safe in either context — the C++ side
/// picks the `...FromISR` kernel call when it runs inside an interrupt.
static EVENT_GROUP: OnceLock<EventGroup> = OnceLock::new();

/// Signal `bits` on the coordinator event group, from task or ISR context.
fn signal_event_group(bits: u32) {
    if let Some(event_group) = EVENT_GROUP.get() {
        event_group.set_events(bits);
    }
}

pub fn trigger_coordinator_interrupt() {
    signal_event_group(EVT_INTR);
}

pub fn trigger_coordinator_disconnect(sn: u8) {
    if (sn as usize) < MAX_SOCKETS {
        signal_event_group(discon_bit(sn));
    }
}

/// Exti pin callback wrapper
pub struct IntrPinCallback;
impl PinCallback for IntrPinCallback {
    fn on_interrupt(&mut self, pin: Pin) {
        disable_interrupt(pin);
        trigger_coordinator_interrupt();
    }
}

static INTR_CALLBACK: IntrPinCallback = IntrPinCallback;

/// Coordinator task loop
pub extern "C" fn w5500_coordinator_task(_param: *mut c_void) {
    let event_group = {
        let mut coord = COORDINATOR.lock();
        coord.event_group.init_static();
        coord.event_group.take_ownership();
        let event_group = coord.event_group.clone();
        // Publish before the interrupt line is armed, so the ISR can signal.
        let _ = EVENT_GROUP.set(event_group.clone());
        event_group
    };

    let (mac, intr_pin) = {
        let mut net = NET.lock();
        let Some(dev) = net.device.as_mut() else {
            return;
        };
        dev.software_reset();
        dev.configure_buffers();

        let mac = COORDINATOR.lock().mac;
        dev.set_mac(&mac);
        dev.configure_phy();

        // Enable socket interrupts in SIMR for managed sockets and clear any pending common interrupts
        dev.bus.write_u8(SIMR, CREG_BLOCK, ((1 << MAX_SOCKETS) - 1) as u8);
        dev.bus.write_u8(IR, CREG_BLOCK, 0xFF);

        (mac, dev.intr.pin())
    };

    attach_interrupt(intr_pin, Edge::Falling, &INTR_CALLBACK);
    enable_interrupt(intr_pin);

    // Wait for physical Ethernet link up (up to 3 seconds)
    for _ in 0..LINK_WAIT_TRIES {
        let up = {
            let mut net = NET.lock();
            net.device.as_mut().map(|dev| dev.is_link_up()).unwrap_or(false)
        };
        if up {
            break;
        }
        delay_ms(LINK_POLL_MS);
    }

    // Initialise DHCP
    {
        let mut net = NET.lock();
        let Some(dev) = net.device.as_mut() else {
            return;
        };
        let mut coord = COORDINATOR.lock();
        coord.dhcp.init(&mut dev.bus, &mac);
        coord.state = CoordinatorState::Dhcping;
    }

    let mut tick_counter: u32 = 0;

    loop {
        let events = event_group.wait_events(EVT_INTR | EVT_DISCON_MASK, WAIT_TIMEOUT_MS);

        tick_counter = tick_counter.wrapping_add(1);
        let is_second_tick = (tick_counter % DHCP_TICK_DIVIDER) == 0;

        let state = {
            let coord = COORDINATOR.lock();
            coord.state
        };

        match state {
            CoordinatorState::Idle => {}

            CoordinatorState::Dhcping => {
                let mut bound = false;
                let mut net_info = (None, core::ptr::null_mut(), [0u8; 4], [0u8; 4], [0u8; 4]);

                {
                    let mut net = NET.lock();
                    let Some(dev) = net.device.as_mut() else {
                        break;
                    };
                    let mut coord = COORDINATOR.lock();

                    if dev.is_link_up() {
                        if is_second_tick {
                            coord.dhcp.tick(&mut dev.bus, &mac);
                        }

                        if coord.dhcp.poll(&mut dev.bus, &mac) == DhcpState::Bound {
                            dev.set_net_info(
                                &coord.dhcp.info.ip,
                                &coord.dhcp.info.subnet,
                                &coord.dhcp.info.gateway,
                            );
                            coord.state = CoordinatorState::Running;
                            bound = true;
                            net_info = (coord.net_cb, coord.net_arg, coord.dhcp.info.ip, coord.dhcp.info.subnet, coord.dhcp.info.gateway);
                        }
                    }
                }

                if bound {
                    // Fire network ready event (LwipReady = 2)
                    if let Some(cb) = net_info.0 {
                        cb(2, net_info.1);
                    }
                }
            }

            CoordinatorState::Running => {
                let net = &mut *NET.lock();
                let Some(dev) = net.device.as_mut() else {
                    break;
                };
                let sockets = &mut net.sockets;

                // 1. Handle explicit software disconnect requests
                if (events & EVT_DISCON_MASK) != 0 {
                    for sn in 0..MAX_SOCKETS {
                        if (events & discon_bit(sn as u8)) != 0 {
                            if let Some(sock) = sockets[sn].as_mut() {
                                sock.handle_disconnect(&mut dev.bus);
                            }
                        }
                    }
                }

                // 2. Handle hardware interrupts from W5500
                let ir = dev.bus.read_u8(IR, CREG_BLOCK);
                if ir != 0 {
                    dev.bus.write_u8(IR, CREG_BLOCK, ir);
                }

                let sir = dev.bus.read_u8(SIR, CREG_BLOCK);

                if sir != 0 {
                    for sn in 0..MAX_SOCKETS {
                        if (sir & (1 << sn)) != 0 {
                            if let Some(sock) = sockets[sn].as_mut() {
                                let sn_ir = dev.bus.read_u8(Sn_IR, sreg_block(sn as u8));
                                if sn_ir != 0 {
                                    dev.bus.write_u8(Sn_IR, sreg_block(sn as u8), sn_ir);

                                    if (sn_ir & Sn_IR_SENDOK) != 0 {
                                        sock.clear_is_sending();
                                        sock.invoke(SocketEvent::WriteAvailable);
                                    }
                                    if (sn_ir & Sn_IR_RECV) != 0 {
                                        sock.invoke(SocketEvent::DataAvailable);
                                    }
                                    if (sn_ir & Sn_IR_DISCON) != 0 {
                                        sock.handle_disconnect(&mut dev.bus);
                                    }
                                    if (sn_ir & Sn_IR_TIMEOUT) != 0 {
                                        sock.handle_disconnect(&mut dev.bus);
                                    }
                                }
                            }
                        }
                    }
                }

                // Connected events and state transitions are owned by `poll()`.
                for sock in sockets.iter_mut().flatten() {
                    sock.poll(&mut dev.bus);
                }

                // Re-enable EXTI interrupt
                enable_interrupt(intr_pin);

                // If pin is still asserted (active low), re-trigger
                if dev.intr.is_low() {
                    disable_interrupt(intr_pin);
                    trigger_coordinator_interrupt();
                    delay_ms(INTR_RETRIGGER_MS);
                }
            }
        }
    }
}
