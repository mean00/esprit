#![allow(dead_code)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
use crate::gpio::lnPin;
use crate::rn_spi_c;

// Re-export the raw C types so advanced users can bypass the wrapper
pub use rn_spi_c::{
    lnSpiCallback, lnSPISettings, ln_spi_c, spiBitOrder, spiDataMode,
    spiBitOrder_SPI_LSBFIRST, spiBitOrder_SPI_MSBFIRST,
    spiDataMode_SPI_MODE0, spiDataMode_SPI_MODE1,
    spiDataMode_SPI_MODE2, spiDataMode_SPI_MODE3,
};

/// SPI data mode (CPOL/CPHA combination).
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum SpiMode {
    Mode0,
    Mode1,
    Mode2,
    Mode3,
}

impl From<SpiMode> for rn_spi_c::spiDataMode {
    fn from(m: SpiMode) -> Self {
        match m {
            SpiMode::Mode0 => rn_spi_c::spiDataMode_SPI_MODE0,
            SpiMode::Mode1 => rn_spi_c::spiDataMode_SPI_MODE1,
            SpiMode::Mode2 => rn_spi_c::spiDataMode_SPI_MODE2,
            SpiMode::Mode3 => rn_spi_c::spiDataMode_SPI_MODE3,
        }
    }
}

/// Bit ordering on the SPI bus.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum BitOrder {
    LsbFirst,
    MsbFirst,
}

impl From<BitOrder> for rn_spi_c::spiBitOrder {
    fn from(o: BitOrder) -> Self {
        match o {
            BitOrder::LsbFirst => rn_spi_c::spiBitOrder_SPI_LSBFIRST,
            BitOrder::MsbFirst => rn_spi_c::spiBitOrder_SPI_MSBFIRST,
        }
    }
}

// ---------- the SpiBus wrapper ----------

/// Owned, non‑copy handle to an SPI peripheral.
/// Created via `SpiBus::new(instance, cs_pin)`.
/// The underlying C object is destroyed when `SpiBus` is dropped.
pub struct SpiBus {
    raw: *mut rn_spi_c::ln_spi_c,
}

impl SpiBus {
    /// Create a new SPI bus on hardware instance `instance` with chip-select `cs_pin`.
    /// `cs_pin` is the C `lnPin` enum val.  Use `lnPin::PA4` etc.
    pub fn new(instance: u32, cs_pin: lnPin) -> Self {
        let raw = unsafe { rn_spi_c::lnspi_create(instance, cs_pin as i32) };
        assert!(!raw.is_null(), "lnspi_create returned NULL");
        Self { raw }
    }

    /// Returns a pointer to the raw C object.  Advanced use only.
    #[inline]
    pub fn raw(&self) -> *mut rn_spi_c::ln_spi_c {
        self.raw
    }

    /// Initialise the bus with a given data size (typically 8).
    pub fn begin(&mut self, data_size_bits: u32) {
        unsafe { rn_spi_c::lnspi_begin(self.raw, data_size_bits); }
    }

    /// De‑initialise the bus (call `begin` again to re‑initialise).
    pub fn end(&mut self) {
        unsafe { rn_spi_c::lnspi_end(self.raw); }
    }

    pub fn set_bit_order(&mut self, order: BitOrder) {
        unsafe { rn_spi_c::lnspi_set_bit_order(self.raw, order.into()); }
    }

    pub fn set_data_mode(&mut self, mode: SpiMode) {
        unsafe { rn_spi_c::lnspi_set_data_mode(self.raw, mode.into()); }
    }

    pub fn set_speed(&mut self, speed_hz: u32) {
        unsafe { rn_spi_c::lnspi_set_speed(self.raw, speed_hz); }
    }

    pub fn set_ssel(&mut self, ssel: lnPin) {
        unsafe { rn_spi_c::lnspi_set_ssel(self.raw, ssel as i32); }
    }

    /// Apply a full `lnSPISettings` struct (expert).
    pub fn apply_settings(&mut self, settings: &rn_spi_c::lnSPISettings) {
        unsafe { rn_spi_c::lnspi_set(self.raw, settings); }
    }

    // ---------- synchronous writes ----------

    /// Write a single byte; returns `true` on success.
    pub fn write8(&mut self, data: u8) -> bool {
        unsafe { rn_spi_c::lnspi_write8(self.raw, data) }
    }

    /// Write a single 16‑bit word; returns `true` on success.
    pub fn write16(&mut self, data: u16) -> bool {
        unsafe { rn_spi_c::lnspi_write16(self.raw, data) }
    }

    /// Block until the prev write completes.
    pub fn wait_done(&mut self) -> bool {
        unsafe { rn_spi_c::lnspi_wait_for_completion(self.raw) }
    }

    /// Write `data` bytes (blocking).
    pub fn write_slice8(&mut self, data: &[u8]) -> bool {
        unsafe {
            rn_spi_c::lnspi_block_write8(self.raw, data.len() as u32, data.as_ptr())
        }
    }

    /// Write `data` 16‑bit words (blocking).
    pub fn write_slice16(&mut self, data: &[u16]) -> bool {
        unsafe {
            rn_spi_c::lnspi_block_write16(self.raw, data.len() as u32, data.as_ptr())
        }
    }

    /// Repeatedly write the same 8‑bit val `count` times.
    pub fn fill8(&mut self, count: u32, val: u8) -> bool {
        unsafe { rn_spi_c::lnspi_block_write8_repeat(self.raw, count, val) }
    }

    /// Repeatedly write the same 16‑bit val `count` times.
    pub fn fill16(&mut self, count: u32, val: u16) -> bool {
        unsafe { rn_spi_c::lnspi_block_write16_repeat(self.raw, count, val) }
    }

    /// Bidirectional transfer: sends `tx` slice, receives into `rx` slice.
    /// Both slices must have the same length.
    pub fn transfer8(&mut self, tx: &[u8], rx: &mut [u8]) -> bool {
        assert_eq!(tx.len(), rx.len());
        unsafe {
            rn_spi_c::lnspi_transfer8(self.raw, tx.len() as u32, tx.as_ptr(), rx.as_mut_ptr())
        }
    }

    // ---------- DMA / async (raw API) ----------

    /// Start an async 8‑bit DMA write.
    pub fn async_write8(
        &mut self,
        data: &[u8],
        cb: rn_spi_c::lnSpiCallback,
        cookie: *mut cty::c_void,
        repeat: bool,
    ) -> bool {
        unsafe {
            rn_spi_c::lnspi_asyncWrite8(
                self.raw,
                data.len() as u32,
                data.as_ptr(),
                cb,
                cookie,
                repeat,
            )
        }
    }

    /// Queue another 8‑bit DMA write (use after a prev async write finishes).
    pub fn next_write8(
        &mut self,
        data: &[u8],
        cb: rn_spi_c::lnSpiCallback,
        cookie: *mut cty::c_void,
        repeat: bool,
    ) -> bool {
        unsafe {
            rn_spi_c::lnspi_nextWrite8(
                self.raw,
                data.len() as u32,
                data.as_ptr(),
                cb,
                cookie,
                repeat,
            )
        }
    }

    /// Start an async 16‑bit DMA write.
    pub fn async_write16(
        &mut self,
        data: &[u16],
        cb: rn_spi_c::lnSpiCallback,
        cookie: *mut cty::c_void,
        repeat: bool,
    ) -> bool {
        unsafe {
            rn_spi_c::lnspi_asyncWrite16(
                self.raw,
                data.len() as u32,
                data.as_ptr(),
                cb,
                cookie,
                repeat,
            )
        }
    }

    /// Queue another 16‑bit DMA write.
    pub fn next_write16(
        &mut self,
        data: &[u16],
        cb: rn_spi_c::lnSpiCallback,
        cookie: *mut cty::c_void,
        repeat: bool,
    ) -> bool {
        unsafe {
            rn_spi_c::lnspi_nextWrite16(
                self.raw,
                data.len() as u32,
                data.as_ptr(),
                cb,
                cookie,
                repeat,
            )
        }
    }

    /// Finish an async DMA transfer; returns `true` if idle.
    pub fn finish_async_dma(&mut self) -> bool {
        unsafe { rn_spi_c::lnspi_finishAsyncDma(self.raw) }
    }

    /// Block waiting for an async transfer to finish.
    pub fn wait_async(&mut self) -> bool {
        unsafe { rn_spi_c::lnspi_waitForAsync(self.raw) }
    }
}

impl Drop for SpiBus {
    fn drop(&mut self) {
        unsafe { rn_spi_c::lnspi_delete(self.raw); }
    }
}