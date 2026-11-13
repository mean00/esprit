//! Common Rust CDC demo using high-level `rust_esprit` API.
//!
//! Mirrors `usb_cdc` demo behavior in idiomatic Rust using `Cdc::new_static`.

#![no_std]

use rust_esprit::{
    Cdc, CdcEvent, CdcEventHandler, EventGroup, Gpio, OnceLock, Pin, PinMode, Usb, UsbEvent, logger,
    logger_init,
};

logger_init!();

#[cfg(feature = "rp2040")]
const LED: Pin = Pin::GPIO25;
#[cfg(feature = "rp2040")]
const LED2: Pin = Pin::GPIO10;
#[cfg(feature = "rp2040")]
const LED3: Pin = Pin::GPIO11;

#[cfg(not(feature = "rp2040"))]
const LED: Pin = Pin::PC13;
#[cfg(not(feature = "rp2040"))]
const LED2: Pin = Pin::PB8;
#[cfg(not(feature = "rp2040"))]
const LED3: Pin = Pin::PB9;

static EVENT_GROUP: OnceLock<EventGroup> = OnceLock::new();

struct DemoCdcHandler;

impl CdcEventHandler for DemoCdcHandler {
    fn handle(&self, _interface: i32, event: CdcEvent, _payload: u32) {
        if let Some(eg) = EVENT_GROUP.get() {
            eg.set_events(1 << (event as u32));
        }
    }
}

static CDC_HANDLER: DemoCdcHandler = DemoCdcHandler;

fn usb_event_handler(event: UsbEvent) {
    match event {
        UsbEvent::Connect => logger!("CONNECT\n"),
        UsbEvent::Disconnect => logger!("DISCONNECT\n"),
        UsbEvent::Suspend => logger!("SUSPEND\n"),
        UsbEvent::Resume => logger!("RESUME\n"),
    }
}

#[unsafe(no_mangle)]
extern "C" fn user_init() {
    logger!("Starting lnTUSB Test (Rust)\n");

    let mut led = LED;
    let mut led2 = LED2;
    let mut led3 = LED3;
    let _ = Gpio::set_mode(&mut led, PinMode::Output);
    let _ = Gpio::set_mode(&mut led2, PinMode::Output);
    let _ = Gpio::set_mode(&mut led3, PinMode::Output);

    let mut eg = EventGroup::new();
    eg.take_ownership();
    let _ = EVENT_GROUP.set(eg.clone());

    let mut usb = Usb::new_fn(0, usb_event_handler);
    usb.set_configuration();

    let mut cdc = Cdc::new_static(0, &CDC_HANDLER);

    usb.start();

    let mut buf = [0u8; 100];

    loop {
        let t = eg.wait_events(0xffffffff, 1000);
        if t == 0 {
            Gpio::toggle(&mut led);
            Gpio::toggle(&mut led2);
            Gpio::toggle(&mut led3);
            continue;
        }
        if (t & (1 << (CdcEvent::SessionStart as u32))) != 0 {
            logger!("CDC SESSION START\n");
        }
        if (t & (1 << (CdcEvent::SessionEnd as u32))) != 0 {
            logger!("CDC SESSION END\n");
        }
        if (t & (1 << (CdcEvent::SetSpeed as u32))) != 0 {
            logger!("CDC SET SPEED\n");
        }
        if (t & (1 << (CdcEvent::DataAvailable as u32))) != 0 {
            logger!("Dt\n");
            let n = cdc.read(&mut buf);
            if n > 0 {
                let _ = cdc.write(b">");
                let _ = cdc.write(&buf[..n as usize]);
                cdc.flush();
            }
        }
    }
}
