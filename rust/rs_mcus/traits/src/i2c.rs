//! I²C: a master port, described by a list of operations.
//!
//! One transaction is a start, some writes, zero or more reads, a stop — which is
//! exactly [`Operation`], so a caller describes the bus traffic and the driver
//! decides how to put it on the wire (repeated start, DMA, polled).

use crate::error::Result;

/// Chunks a single [`I2c::transfer`] can carry, i.e. the depth of the operation
/// list a family builds for a multi-chunk write. 16 covers a register address plus
/// a payload split in a handful of pieces, which is what a sensor transaction is.
pub const MAX_CHUNKS: usize = 16;

/// Rate and own address of a port.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct I2cConfig {
    pub speed_hz: u32,
    pub own_address: u16,
}

impl Default for I2cConfig {
    fn default() -> Self {
        Self {
            speed_hz: 100_000,
            own_address: 0,
        }
    }
}

/// One step of a transaction, executed in order.
pub enum Operation<'a> {
    /// Clock these bytes out.
    Write(&'a [u8]),
    /// Clock these bytes in.
    Read(&'a mut [u8]),
}

/// What an I²C interrupt calls.
pub trait I2cHandler: Sync {
    /// A byte arrived (slave mode, or an asynchronous transfer).
    fn on_rx(&self, _byte: u8) {}
    /// A block arrived (DMA path).
    fn on_rx_buffer(&self, _buffer: &[u8]) {}
    /// The last byte left the shifter.
    fn on_tx_complete(&self) {}
    /// NACK, arbitration lost, bus error.
    fn on_error(&self, _error: crate::error::Error) {}
}

/// An I²C master port.
///
/// * **blocking** — [`transfer`](Self::transfer) plus the
///   [`write_to`](Self::write_to) / [`read_from`](Self::read_from) /
///   [`write_read`](Self::write_read) / [`write_chunks`](Self::write_chunks)
///   defaults built on it;
/// * **asynchronous** — [`start_write_chunks`](Self::start_write_chunks) queues the
///   transaction and returns; the handler installed with
///   [`set_handler`](Self::set_handler) gets `on_tx_complete` /
///   `on_rx_buffer` **from the bus interrupt** when the last byte left the
///   shifter. [`is_busy`](Self::is_busy) polls that state and
///   [`wait_idle`](Self::wait_idle) blocks on it.
///
/// The buffers of an asynchronous transaction belong to the bus until the
/// completion runs: they must not be reused before `on_tx_complete` or a
/// successful `wait_idle`.
pub trait I2c {
    /// Apply the configuration.
    fn configure(&mut self, config: &I2cConfig) -> Result<()>;

    /// Change the clock only.
    fn set_speed(&mut self, speed_hz: u32) -> Result<()>;

    /// Run one transaction against the 7-bit address `target`: start, the
    /// operations in order, stop. A read after a write uses a repeated start.
    fn transfer(&mut self, target: u8, operations: &mut [Operation<'_>]) -> Result<()>;

    /// [`transfer`](Self::transfer) with a single write.
    fn write_to(&mut self, target: u8, data: &[u8]) -> Result<()> {
        self.transfer(target, &mut [Operation::Write(data)])
    }

    /// [`transfer`](Self::transfer) with a single read.
    fn read_from(&mut self, target: u8, buffer: &mut [u8]) -> Result<()> {
        self.transfer(target, &mut [Operation::Read(buffer)])
    }

    /// Write a register address then read `buffer` back, with a repeated start.
    fn write_read(&mut self, target: u8, write: &[u8], read: &mut [u8]) -> Result<()> {
        self.transfer(
            target,
            &mut [Operation::Write(write), Operation::Read(read)],
        )
    }

    /// Write several buffers to `target` in one transaction (repeated starts
    /// between them), which is what a register-then-payload transfer wants.
    fn write_chunks(&mut self, target: u8, chunks: &[&[u8]]) -> Result<()> {
        // The operation list is built in place: at most `MAX_CHUNKS` chunks, so no
        // allocation and no heap on the target.
        let mut operations = [const { Operation::Write(&[]) }; MAX_CHUNKS];
        if chunks.len() > MAX_CHUNKS {
            return Err(crate::error::Error::InvalidConfig);
        }
        for (slot, chunk) in operations.iter_mut().zip(chunks) {
            *slot = Operation::Write(chunk);
        }
        self.transfer(target, &mut operations[..chunks.len()])
    }

    // ---- asynchronous ------------------------------------------------------

    /// Start a multi-chunk write and return; [`I2cHandler::on_tx_complete`] runs
    /// in the bus interrupt once the last byte is out.
    ///
    /// `Err(Unsupported)` when the family has no asynchronous path.
    fn start_write_chunks(&mut self, _target: u8, _chunks: &[&[u8]]) -> Result<()> {
        Err(crate::error::Error::Unsupported)
    }

    /// `true` while an asynchronous transaction is in flight.
    fn is_busy(&self) -> bool {
        false
    }

    /// Block until the transaction armed with
    /// [`start_write_chunks`](Self::start_write_chunks) completes.
    fn wait_idle(&mut self) -> Result<()> {
        Ok(())
    }

    /// Install the handler of **this** instance.
    fn set_handler(&mut self, handler: Option<&'static dyn I2cHandler>);
}
