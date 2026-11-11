//! Device initialization, reset, and PHY control for the W5500.

use crate::bus::W5500Bus;
use crate::registers::*;
use rs_rtos::task::delay_ms;
use rust_esprit::{GpioMode, GpioPin};

/// Pin configuration for the W5500 SPI interface.
#[derive(Copy, Clone)]
pub struct W5500Pins {
    pub spi_instance: u32,
    pub miso: GpioPin,
    pub mosi: GpioPin,
    pub clk: GpioPin,
    pub cs: GpioPin,
    pub reset: GpioPin,
    pub intr: GpioPin,
}

/// Buffer size distribution in KB across the 8 hardware sockets.
pub const SOCKET_TX_SIZES: [u8; 8] = [4, 4, 4, 0, 0, 0, 0, 2];
pub const SOCKET_RX_SIZES: [u8; 8] = [4, 4, 4, 0, 0, 0, 0, 2];

/// Core W5500 controller abstraction.
pub struct W5500Device {
    pub bus: W5500Bus,
    pub reset: GpioPin,
    pub intr: GpioPin,
}

unsafe impl Send for W5500Device {}
unsafe impl Sync for W5500Device {}

impl W5500Device {
    /// Initialize GPIOs, SPI bus, and execute a hardware reset.
    pub fn new(mut pins: W5500Pins) -> Self {
        // Setup SPI function on pins
        pins.mosi.set_mode(GpioMode::Spi);
        pins.miso.set_mode(GpioMode::Spi);
        pins.clk.set_mode(GpioMode::Spi);

        // Chip-select pin
        pins.cs.set_high();
        pins.cs.set_mode(GpioMode::Output);

        // Reset pin
        pins.reset.set_high();
        pins.reset.set_mode(GpioMode::Output);

        // Interrupt pin
        pins.intr.set_mode(GpioMode::InputPullUp);

        delay_ms(5);

        let bus = W5500Bus::new(pins.spi_instance, pins.cs);

        let mut device = Self {
            bus,
            reset: pins.reset,
            intr: pins.intr,
        };

        device.hardware_reset();
        device
    }

    /// Pulse the hardware reset line.
    pub fn hardware_reset(&mut self) {
        self.reset.set_low();
        delay_ms(10);
        self.reset.set_high();
        delay_ms(50);
    }

    /// Trigger a software reset via the Mode Register (MR).
    pub fn software_reset(&mut self) {
        self.bus.write_u8(MR, CREG_BLOCK, MR_RST);
        delay_ms(10);
    }

    /// Configure the TX and RX buffer allocations for all 8 sockets.
    pub fn configure_buffers(&mut self) {
        for sn in 0..8 {
            self.bus.write_u8(Sn_TXBUF_SIZE, sreg_block(sn), SOCKET_TX_SIZES[sn as usize]);
            self.bus.write_u8(Sn_RXBUF_SIZE, sreg_block(sn), SOCKET_RX_SIZES[sn as usize]);
        }
    }

    /// Set the device MAC (Hardware) address and verify it by reading back.
    pub fn set_mac(&mut self, mac: &[u8; 6]) -> bool {
        self.bus.write_reg(SHAR, CREG_BLOCK, mac);
        let mut read_mac = [0u8; 6];
        self.bus.read_reg(SHAR, CREG_BLOCK, &mut read_mac);
        &read_mac == mac
    }

    /// Get the current configured MAC address.
    pub fn get_mac(&mut self) -> [u8; 6] {
        let mut mac = [0u8; 6];
        self.bus.read_reg(SHAR, CREG_BLOCK, &mut mac);
        mac
    }

    /// Set the IPv4 network parameters (IP, subnet mask, default gateway).
    pub fn set_net_info(&mut self, ip: &[u8; 4], subnet: &[u8; 4], gateway: &[u8; 4]) {
        self.bus.write_reg(SIPR, CREG_BLOCK, ip);
        self.bus.write_reg(SUBR, CREG_BLOCK, subnet);
        self.bus.write_reg(GAR, CREG_BLOCK, gateway);
    }

    /// Configure the PHY for software control with Auto-Negotiation (all capable).
    pub fn configure_phy(&mut self) {
        let phy_conf = PHYCFGR_OPMD | PHYCFGR_OPMDC_ALLA;
        self.bus.write_u8(PHYCFGR, CREG_BLOCK, phy_conf);
        delay_ms(10);
        self.bus.write_u8(PHYCFGR, CREG_BLOCK, PHYCFGR_NORMAL | phy_conf);
        delay_ms(200);
    }

    /// Read the PHY status register and check if link is up.
    pub fn is_link_up(&mut self) -> bool {
        let stat = self.bus.read_u8(PHYCFGR, CREG_BLOCK);
        (stat & PHYCFGR_LNK_ON) != 0
    }

    /// Read the chip version (should return 0x04 for W5500).
    pub fn chip_version(&mut self) -> u8 {
        self.bus.read_u8(VERSIONR, CREG_BLOCK)
    }
}
