//! W5500 TCP echo server demo using `rs_w5500` + `rs_socketrunner`.

#![no_std]

extern crate rs_w5500;

use core::cell::UnsafeCell;
use core::ffi::c_void;
use core::ptr::null_mut;
use core::sync::atomic::{AtomicPtr, Ordering};

use rs_socketrunner::{
    DEFAULT_EVENT_WAIT_TIMEOUT_MS, EventGroup, RunnerEvent, SocketRunner, SocketRunnerHandler,
};
use rs_w5500::W5500SocketHandle;
use rust_esprit::{logger, logger_init};

logger_init!();

const TCP_PORT: u16 = 3000;
const RX_PACKET_SIZE: usize = 2048;

struct SyncBuf(UnsafeCell<[u8; RX_PACKET_SIZE]>);
unsafe impl Sync for SyncBuf {}

static RX_PACKET: SyncBuf = SyncBuf(UnsafeCell::new([0; RX_PACKET_SIZE]));
static EVENT_GROUP_PTR: AtomicPtr<EventGroup> = AtomicPtr::new(null_mut());

struct EchoHandler {
    connected: bool,
}

impl SocketRunnerHandler for EchoHandler {
    fn on_connected(&mut self) {
        self.connected = true;
        logger!("EchoHandler: client connected\n");
    }

    fn on_disconnected(&mut self) {
        self.connected = false;
        logger!("EchoHandler: client disconnected\n");
    }

    fn on_poll(&mut self) {}

    fn on_incoming_data(&mut self) {}
}

/// Callback matching `lnLwipEvent`: `1` = `LwipDown`, `2` = `LwipReady`.
#[unsafe(no_mangle)]
pub extern "C" fn rust_w5500_net_cb(evt: u32, _arg: *mut c_void) {
    let ptr = EVENT_GROUP_PTR.load(Ordering::Acquire);
    if ptr.is_null() {
        return;
    }
    let eg = unsafe { &*ptr };
    match evt {
        1 => eg.set_events(RunnerEvent::DOWN.0),
        2 => eg.set_events(RunnerEvent::UP.0),
        _ => {}
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_w5500_loop() {
    let mut eg = EventGroup::uninit();
    eg.init_static();
    eg.take_ownership();
    EVENT_GROUP_PTR.store(&mut eg as *mut EventGroup, Ordering::Release);

    let mut runner: SocketRunner<EchoHandler, W5500SocketHandle> =
        SocketRunner::new(TCP_PORT, eg.clone(), 0, EchoHandler { connected: false });

    // SAFETY: single-threaded loop owns `RX_PACKET`.
    let rx_packet = unsafe { &mut *RX_PACKET.0.get() };

    loop {
        let events = eg.wait_events(0xFFFF_FFFF, DEFAULT_EVENT_WAIT_TIMEOUT_MS as i32);
        if (events & RunnerEvent::DOWN.0) != 0 {
            logger!("Network down. Cleaning up...\n");
        }
        if (events & RunnerEvent::UP.0) != 0 {
            logger!("Network up. Setting up...\n");
        }
        runner.process_events(events);

        if runner.is_connected() {
            while let Ok(Some(n)) = runner.read_packet(&mut rx_packet[..]) {
                if n == 0 {
                    break;
                }
                if runner.write(&rx_packet[..n]).is_err() {
                    break;
                }
            }
            let _ = runner.flush();
        }
    }
}
