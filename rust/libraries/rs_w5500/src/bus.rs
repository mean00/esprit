//! SPI communication bus and register framing for the W5500.

use rust_esprit::hal::spi::{BitOrder, Spi as SpiTrait, SpiConfig, SpiMode};
use rust_esprit::{Gpio, Pin, PinMode, SpiDriver};

const SPI_CONTROL_READ: u8 = 0x00;
const SPI_CONTROL_WRITE: u8 = 0x04;
const SPI_CONTROL_VDM: u8 = 0x00;

/// The W5500's documented maximum SPI clock.
const SPI_CLOCK_HZ: u32 = 30_000_000;
const SPI_BITS_PER_WORD: u8 = 8;

/// "No chip select pin": the bus drives CS itself, so the SPI driver must not.
const NO_CS: i32 = -1;

/// Read chunk, small enough to use one static zero pattern as MOSI filler.
const READ_CHUNK: usize = 64;

/// How often a changing 16-bit counter is re-read before the last sample is
/// accepted (see [`W5500Bus::read_stable_u16`]).
const STABLE_READ_TRIES: usize = 3;

/// Bus interface managing SPI transactions and chip-select for the W5500.
pub struct W5500Bus {
    spi: SpiDriver,
    cs: Pin,
}

// SAFETY: the bus lives in a `Mutex`, so only one task borrows it at a time;
// the SPI driver is a handle to a peripheral and does not care which task drives
// it.
unsafe impl Send for W5500Bus {}

impl W5500Bus {
    /// Initialize the SPI peripheral and the chip-select pin.
    ///
    /// The pads of the bus are routed by [`W5500Device::new`] (they are mode
    /// selections of their own); this only programs the port itself.
    pub fn new(spi_instance: u32, mut cs: Pin) -> Self {
        // Idle high before the pad becomes an output, so the slave is never
        // selected while the mode changes.
        Gpio::write(&mut cs, true);
        let _ = Gpio::set_mode(&mut cs, PinMode::Output);

        let mut spi = SpiDriver::new(spi_instance, NO_CS);
        let _ = SpiTrait::configure(
            &mut spi,
            &SpiConfig {
                speed_hz: SPI_CLOCK_HZ,
                mode: SpiMode::Mode0,
                bit_order: BitOrder::MsbFirst,
                data_size: SPI_BITS_PER_WORD,
            },
        );

        Self { spi, cs }
    }

    /// Write a slice of bytes to the given register address and block.
    pub fn write_reg(&mut self, addr: u16, block: u8, data: &[u8]) {
        let header = [
            (addr >> 8) as u8,
            (addr & 0xFF) as u8,
            (block << 3) | SPI_CONTROL_WRITE | SPI_CONTROL_VDM,
        ];
        Gpio::write(&mut self.cs, false);
        let _ = SpiTrait::write_slice(&mut self.spi, &header);
        if !data.is_empty() {
            let _ = SpiTrait::write_slice(&mut self.spi, data);
        }
        Gpio::write(&mut self.cs, true);
    }

    /// Read bytes from the given register address and block into `data`.
    pub fn read_reg(&mut self, addr: u16, block: u8, data: &mut [u8]) {
        let header = [
            (addr >> 8) as u8,
            (addr & 0xFF) as u8,
            (block << 3) | SPI_CONTROL_READ | SPI_CONTROL_VDM,
        ];
        Gpio::write(&mut self.cs, false);
        let _ = SpiTrait::write_slice(&mut self.spi, &header);

        const ZEROES: [u8; READ_CHUNK] = [0u8; READ_CHUNK];
        for chunk in data.chunks_mut(READ_CHUNK) {
            let len = chunk.len();
            let _ = SpiTrait::transfer_in_place(&mut self.spi, &ZEROES[..len], chunk);
        }
        Gpio::write(&mut self.cs, true);
    }

    /// Write an 8-bit register.
    pub fn write_u8(&mut self, addr: u16, block: u8, val: u8) {
        self.write_reg(addr, block, &[val]);
    }

    /// Read an 8-bit register.
    pub fn read_u8(&mut self, addr: u16, block: u8) -> u8 {
        let mut val = [0u8; 1];
        self.read_reg(addr, block, &mut val);
        val[0]
    }

    /// Write a big-endian 16-bit register.
    pub fn write_u16(&mut self, addr: u16, block: u8, val: u16) {
        self.write_reg(addr, block, &val.to_be_bytes());
    }

    /// Read a big-endian 16-bit register.
    pub fn read_u16(&mut self, addr: u16, block: u8) -> u16 {
        let mut bytes = [0u8; 2];
        self.read_reg(addr, block, &mut bytes);
        u16::from_be_bytes(bytes)
    }

    /// Read a 16-bit register that the chip may update while we read it.
    ///
    /// The W5500 counters (FSR, RSR, TX_WR, RX_RD) are not latched, so a value
    /// can change between the two bytes of a transfer. Re-read and accept only a
    /// repeat, but give up after [`STABLE_READ_TRIES`]: a socket that keeps
    /// receiving never stands still, and the callers only use the result to size
    /// one transfer.
    pub fn read_stable_u16(&mut self, addr: u16, block: u8) -> u16 {
        let mut value = self.read_u16(addr, block);
        for _ in 1..STABLE_READ_TRIES {
            if value == 0 {
                return 0;
            }
            let next = self.read_u16(addr, block);
            if next == value {
                return value;
            }
            value = next;
        }
        value
    }
}
