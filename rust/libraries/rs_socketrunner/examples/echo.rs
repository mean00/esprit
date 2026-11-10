//! Example echo server demonstrating `SocketRunner` and `SocketRunnerHandler`.

use rs_rtos::event::EventGroup;
use rs_socketrunner::{
    DEFAULT_ECHO_PORT, GDB_SLOT_SHIFT, LONG_EVENT_WAIT_TIMEOUT_MS,
    RunnerEvent, SocketRunner, SocketRunnerHandler,
};

struct EchoHandler;

impl SocketRunnerHandler for EchoHandler {
    fn on_connected(&mut self) {
        // Handle client connection
    }

    fn on_disconnected(&mut self) {
        // Handle client disconnection
    }

    fn on_poll(&mut self) {
        // Periodic background poll
    }

    fn on_incoming_data(&mut self) {
        // Notified when data is ready
    }
}

pub fn run_echo_server() {
    let eg = EventGroup::new();
    let handler = EchoHandler;
    let mut runner = SocketRunner::new(DEFAULT_ECHO_PORT, eg.clone(), GDB_SLOT_SHIFT, handler);

    let wait_mask = RunnerEvent::UP.0 | RunnerEvent::DOWN.0 | (RunnerEvent::MASK << GDB_SLOT_SHIFT);

    loop {
        let events = eg.wait_events(wait_mask, LONG_EVENT_WAIT_TIMEOUT_MS);
        runner.process_events(events);

        if runner.is_connected() {
            let _ = runner.echo_incoming();
        }
    }
}

fn main() {
    // Embedded targets drive this from loop()
}
