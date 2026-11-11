//! FreeRTOS coordinator task for PHY state, DHCP client, and socket interrupt dispatch.

use crate::device::W5500Device;
use crate::dhcp::{DhcpClient, DhcpState};
use crate::registers::*;
use crate::socket::{SocketEvent, W5500Socket};
use core::ffi::c_void;
use rs_rtos::event::EventGroup;
use rs_rtos::sync::Mutex;
use rs_rtos::task::delay_ms;
use rust_esprit::{
    attach_interrupt, disable_interrupt, enable_interrupt, Edge, Pin, PinCallback,
};

pub const EVT_DHCP_ASSIGN: u32 = 1 << 0;
pub const EVT_DISCON_SOCK0: u32 = 1 << 1;
pub const EVT_DISCON_SOCK1: u32 = 1 << 2;
pub const EVT_DISCON_SOCK2: u32 = 1 << 3;
pub const EVT_DISCON_SOCK3: u32 = 1 << 4;
pub const EVT_DISCON_MASK: u32 = 0x1E;
pub const EVT_INTR: u32 = 1 << 28;

pub const MAX_SOCKETS: usize = 4;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CoordinatorState {
    Idle,
    Dhcping,
    Running,
}

pub struct Coordinator {
    pub state: CoordinatorState,
    pub event_group: EventGroup,
    pub mac: [u8; 6],
    pub net_cb: Option<extern "C" fn(u32, *mut c_void)>,
    pub net_arg: *mut c_void,
    pub dhcp: DhcpClient,
}

unsafe impl Send for Coordinator {}
unsafe impl Sync for Coordinator {}

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
}

pub static COORDINATOR: Mutex<Coordinator> = Mutex::new(Coordinator::new());

/// Global storage for active sockets
pub static SOCKETS: Mutex<[Option<W5500Socket>; MAX_SOCKETS]> = Mutex::new([None, None, None, None]);

/// Global pointer to the W5500 device
pub static DEVICE: Mutex<Option<W5500Device>> = Mutex::new(None);

static EVENT_GROUP_RAW: core::sync::atomic::AtomicPtr<c_void> =
    core::sync::atomic::AtomicPtr::new(core::ptr::null_mut());

pub fn trigger_coordinator_interrupt() {
    let ptr = EVENT_GROUP_RAW.load(core::sync::atomic::Ordering::Acquire);
    if !ptr.is_null() {
        unsafe {
            rs_rtos::c_fast_event::lnfast_event_group_set_events(
                ptr as *const rs_rtos::c_fast_event::lnfast_event_group_c,
                EVT_INTR,
            );
        }
    }
}

pub fn trigger_coordinator_disconnect(sn: u8) {
    if (sn as usize) < MAX_SOCKETS {
        let bit = 1 << (1 + sn);
        let ptr = EVENT_GROUP_RAW.load(core::sync::atomic::Ordering::Acquire);
        if !ptr.is_null() {
            unsafe {
                rs_rtos::c_fast_event::lnfast_event_group_set_events(
                    ptr as *const rs_rtos::c_fast_event::lnfast_event_group_c,
                    bit,
                );
            }
        }
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
        EVENT_GROUP_RAW.store(coord.event_group.raw(), core::sync::atomic::Ordering::Release);
        coord.event_group.clone()
    };

    let (mac, intr_pin) = {
        let mut dev_guard = DEVICE.lock();
        let dev = dev_guard.as_mut().expect("W5500Device not initialized");
        dev.software_reset();
        dev.configure_buffers();

        let coord = COORDINATOR.lock();
        let mac = coord.mac;
        dev.set_mac(&mac);
        dev.configure_phy();
        (mac, dev.intr.pin())
    };

    attach_interrupt(intr_pin, Edge::Falling, &INTR_CALLBACK);
    enable_interrupt(intr_pin);

    // Wait for physical Ethernet link up (up to 3 seconds)
    {
        let mut dev_guard = DEVICE.lock();
        if let Some(dev) = dev_guard.as_mut() {
            for _ in 0..60 {
                if dev.is_link_up() {
                    break;
                }
                delay_ms(50);
            }
        }
    }

    // Initialise DHCP
    {
        let mut dev_guard = DEVICE.lock();
        let dev = dev_guard.as_mut().unwrap();
        let mut coord = COORDINATOR.lock();
        coord.dhcp.init(&mut dev.bus, &mac);
        coord.state = CoordinatorState::Dhcping;
    }

    let mut tick_counter: u32 = 0;

    loop {
        let events = event_group.wait_events(EVT_DHCP_ASSIGN | EVT_INTR | EVT_DISCON_MASK, 50);

        tick_counter = tick_counter.wrapping_add(1);
        let is_second_tick = (tick_counter % 20) == 0;

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
                    let mut dev_guard = DEVICE.lock();
                    let dev = dev_guard.as_mut().unwrap();
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
                let mut dev_guard = DEVICE.lock();
                if let Some(dev) = dev_guard.as_mut() {
                    // 1. Handle explicit software disconnect requests
                    if (events & EVT_DISCON_MASK) != 0 {
                        let mut sockets_guard = SOCKETS.lock();
                        for sn in 0..MAX_SOCKETS {
                            if (events & (1 << (1 + sn))) != 0 {
                                if let Some(sock) = sockets_guard[sn].as_mut() {
                                    sock.handle_disconnect(&mut dev.bus);
                                }
                            }
                        }
                    }

                    // 2. Handle hardware interrupts from W5500
                    let sir = dev.bus.read_u8(SIR, CREG_BLOCK);

                    if sir != 0 {
                        let mut sockets_guard = SOCKETS.lock();
                        for sn in 0..MAX_SOCKETS {
                            if (sir & (1 << sn)) != 0 {
                                if let Some(sock) = sockets_guard[sn].as_mut() {
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
                                        if (sn_ir & Sn_IR_CON) != 0 {
                                            sock.set_state(crate::socket::SocketState::Connected);
                                            sock.invoke(SocketEvent::Connected);
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

                    // Poll all sockets for state transitions (disconnect detection, etc.)
                    {
                        let mut sockets_guard = SOCKETS.lock();
                        for sock_opt in sockets_guard.iter_mut() {
                            if let Some(sock) = sock_opt.as_mut() {
                                sock.poll(&mut dev.bus);
                            }
                        }
                    }

                    // Re-enable EXTI interrupt
                    enable_interrupt(intr_pin);

                    // If pin is still asserted (active low), re-trigger
                    if dev.intr.is_low() {
                        disable_interrupt(intr_pin);
                        trigger_coordinator_interrupt();
                        delay_ms(1);
                    }
                }
            }
        }
    }
}
