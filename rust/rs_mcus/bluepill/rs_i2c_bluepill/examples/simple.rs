//! I²C through the MCU-agnostic super trait.
//!
//! A transaction is described as a list of [`Operation`]s, which is what the
//! contract asks for: start, the operations in order, stop.

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use rs_i2c_bluepill::I2c;
use rs_mcu_traits::i2c::{I2c as I2cTrait, I2cConfig, Operation};

/// The `no_std` example binary needs an allocator even though nothing allocates.
struct DummyAlloc;

unsafe impl core::alloc::GlobalAlloc for DummyAlloc {
    unsafe fn alloc(&self, _layout: core::alloc::Layout) -> *mut u8 {
        core::ptr::null_mut()
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {}
}

#[global_allocator]
static ALLOCATOR: DummyAlloc = DummyAlloc;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

const EEPROM: u8 = 0x50;
const SPEED_HZ: u32 = 100_000;
/// The EEPROM's register pointer, then the two bytes to read back.
const REGISTER: [u8; 1] = [0x12];

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    let mut bus = I2c::new(0, SPEED_HZ);
    let _ = I2cTrait::configure(
        &mut bus,
        &I2cConfig {
            speed_hz: SPEED_HZ,
            own_address: 0,
        },
    );

    // Write two bytes to a register: the `write_to` default builds the operation
    // list for us.
    let payload = [0x12u8, 0x34];
    let _ = I2cTrait::write_to(&mut bus, EEPROM, &payload);

    // A register read is a write followed by a read with a repeated start.
    let mut value = [0u8; 2];
    let _ = I2cTrait::write_read(&mut bus, EEPROM, &REGISTER, &mut value);

    // The same transaction spelled out, which is what the defaults expand to.
    let _ = I2cTrait::transfer(
        &mut bus,
        EEPROM,
        &mut [Operation::Write(&REGISTER), Operation::Read(&mut value)],
    );

    // ---- asynchronous: the chunks are chained with repeated starts by the bus
    // interrupt, and `on_tx_complete` runs when the STOP is out.
    let header = [0x12u8];
    let payload_async = [0x34u8, 0x56];
    if I2cTrait::start_write_chunks(&mut bus, EEPROM, &[&header, &payload_async]).is_ok() {
        let _ = I2cTrait::wait_idle(&mut bus);
    }

    loop {
        core::hint::spin_loop();
    }
}


