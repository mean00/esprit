//! SPI communication bus and register framing for the W5500.

use rs_rtos::sync::Mutex;
use rust_esprit::{BitOrder, GpioMode, GpioPin, Pin, Spi, SpiMode};

const SPI_CONTROL_READ: u8 = 0x00;
const SPI_CONTROL_WRITE: u8 = 0x04;
const SPI_CONTROL_VDM: u8 = 0x00;

/// Bus interface managing SPI transactions and chip-select for the W5500.
pub struct W5500Bus {
    spi: Spi,
    cs: GpioPin,
    mutex: Mutex<()>,
}

unsafe impl Send for W5500Bus {}
unsafe impl Sync for W5500Bus {}

impl W5500Bus {
    /// Initialize the SPI peripheral and chip-select GPIO.
    pub fn new(spi_instance: u32, mut cs: GpioPin) -> Self {
        cs.set_high();
        cs.set_mode(GpioMode::Output);

        // Spi::new takes instance and CS pin. We pass Pin::NoPin because W5500Bus manages CS explicitly.
        let mut spi = Spi::new(spi_instance, Pin::NoPin);
        spi.set_data_mode(SpiMode::Mode0);
        spi.set_speed(30_000_000); // 30 MHz max reliable SPI
        spi.set_bit_order(BitOrder::MsbFirst);
        spi.begin(8);

        Self {
            spi,
            cs,
            mutex: Mutex::new(()),
        }
    }

    /// Write a slice of bytes to the given register address and block.
    pub fn write_reg(&mut self, addr: u16, block: u8, data: &[u8]) {
        let _guard = self.mutex.lock();
        let header = [
            (addr >> 8) as u8,
            (addr & 0xFF) as u8,
            (block << 3) | SPI_CONTROL_WRITE | SPI_CONTROL_VDM,
        ];
        self.cs.set_low();
        self.spi.write_slice8(&header);
        if !data.is_empty() {
            self.spi.write_slice8(data);
        }
        self.cs.set_high();
    }

    /// Read bytes from the given register address and block into `data`.
    pub fn read_reg(&mut self, addr: u16, block: u8, data: &mut [u8]) {
        let _guard = self.mutex.lock();
        let header = [
            (addr >> 8) as u8,
            (addr & 0xFF) as u8,
            (block << 3) | SPI_CONTROL_READ | SPI_CONTROL_VDM,
        ];
        self.cs.set_low();
        self.spi.write_slice8(&header);

        const ZEROES: [u8; 64] = [0u8; 64];
        for chunk in data.chunks_mut(64) {
            let len = chunk.len();
            self.spi.transfer8(&ZEROES[..len], chunk);
        }
        self.cs.set_high();
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
        let bytes = [(val >> 8) as u8, (val & 0xFF) as u8];
        self.write_reg(addr, block, &bytes);
    }

    /// Read a big-endian 16-bit register.
    pub fn read_u16(&mut self, addr: u16, block: u8) -> u16 {
        let mut bytes = [0u8; 2];
        self.read_reg(addr, block, &mut bytes);
        ((bytes[0] as u16) << 8) | (bytes[1] as u16)
    }

    /// Read a 16-bit register with double-read consistency check.
    ///
    /// W5500 internal counters (FSR, RSR, TX_WR, RX_RD) can update asynchronously
    /// between byte transfers, causing torn reads. Double-reading ensures consistency.
    pub fn read_stable_u16(&mut self, addr: u16, block: u8) -> u16 {
        let mut buf1 = [0u8; 2];
        let mut buf2 = [0u8; 2];
        loop {
            self.read_reg(addr, block, &mut buf1);
            let val1 = ((buf1[0] as u16) << 8) | (buf1[1] as u16);
            if val1 == 0 {
                return 0;
            }
            self.read_reg(addr, block, &mut buf2);
            let val2 = ((buf2[0] as u16) << 8) | (buf2[1] as u16);
            if val1 == val2 {
                return val1;
            }
        }
    }
}
