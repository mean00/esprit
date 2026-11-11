//! SPI through the MCU-agnostic super trait.
//!
//! `rs_mcu_traits::spi::Spi` is the contract; the chip-select pin and the DMA
//! path stay family knobs on `rs_spi_bluepill::Spi` itself.

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use rs_mcu_traits::spi::{BitOrder, Spi as SpiTrait, SpiConfig, SpiMode};
use rs_spi_bluepill::Spi;

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

const SPEED_HZ: u32 = 1_000_000;
/// No chip select: the board wires the slave permanently.
const NO_CS: i32 = -1;

/// Told when the asynchronous transfer ends (it runs in the DMA interrupt).
struct Done;

impl rs_mcu_traits::spi::SpiHandler for Done {
    fn on_tx_complete(&self) {}
}

static DONE: Done = Done;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    let mut spi = Spi::new(0, NO_CS);

    // One call programs the rate, the clock phase and the frame size.
    let _ = SpiTrait::configure(
        &mut spi,
        &SpiConfig {
            speed_hz: SPEED_HZ,
            mode: SpiMode::Mode0,
            bit_order: BitOrder::MsbFirst,
            data_size: 8,
        },
    );

    // Byte exchange, then the block helpers the trait provides for free.
    let _echo = SpiTrait::transfer(&mut spi, 0xAA);

    let tx = [0x01u8, 0x02, 0x03];
    let mut rx = [0u8; 3];
    let _ = SpiTrait::transfer_in_place(&mut spi, &tx, &mut rx);

    let _ = SpiTrait::write_slice(&mut spi, &tx);
    let _ = SpiTrait::fill(&mut spi, 4, 0xFF);

    // ---- asynchronous: the transfer is DMA driven and the handler's
    // `on_tx_complete` runs from the DMA interrupt once the last bit is out.
    SpiTrait::set_handler(&mut spi, Some(&DONE));
    if SpiTrait::start_write(&mut spi, &tx).is_ok() {
        // Either wait for it, or use `is_busy()` and come back later.
        let _ = SpiTrait::wait_idle(&mut spi);
    }

    loop {
        core::hint::spin_loop();
    }
}


