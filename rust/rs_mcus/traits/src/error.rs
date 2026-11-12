//! The one error type of the peripheral contracts.
//!
//! Every trait method that can fail returns [`Result`]. A family that cannot do
//! something at all (an RP2040 has no DAC, a CH32V3 has no PIO) answers
//! [`Error::Unsupported`] instead of panicking or returning a fake success.

/// Why a peripheral operation failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// This family/instance does not implement the capability. Permanent: retrying
    /// cannot help. A caller may fall back to a slower path.
    Unsupported,
    /// The request cannot be represented by the hardware: a rate outside the
    /// divider range, a pin with no analog input, a buffer longer than the
    /// transfer counter.
    InvalidConfig,
    /// The peripheral did not finish in the allotted time.
    Timeout,
    /// The peripheral is busy with a previous operation (a transfer still in
    /// flight, a scan not yet done). Retry once it completes.
    Busy,
    /// Data was lost: a FIFO overflowed, a frame was dropped, a queue was full.
    Overrun,
    /// The bus reported an error (I²C NACK/arbitration, SPI underrun) or the
    /// peripheral flagged one.
    Io,
    /// The buffer is too small for the requested number of samples/bytes.
    BufferTooSmall,
}

/// `core::result::Result` specialised to [`Error`].
pub type Result<T> = core::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    /// The error type stays small enough to be returned by value from an ISR-called
    /// handler: it is a `repr`-free fieldless enum, one byte today.
    #[test]
    fn error_is_one_byte() {
        assert_eq!(core::mem::size_of::<Error>(), 1);
    }

    /// `Result` keeps the error inspectable by value (so `?` works in a `no_std`
    /// driver without boxing).
    #[test]
    fn result_carries_the_reason() {
        let failed: Result<u8> = Err(Error::Busy);
        assert_eq!(failed, Err(Error::Busy));
    }
}
