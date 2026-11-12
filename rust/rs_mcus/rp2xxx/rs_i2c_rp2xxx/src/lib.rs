//! RP2040 / RP2350 I²C: the `rs_mcu_traits::i2c::I2c` contract, not written yet.
//!
//! esprit has no RP2xxx implementation of this peripheral, and the C++ bridge
//! (`lnI2C_c.cpp`) is deliberately *not* built for RP2040/RP2350 — see the
//! `if(NOT (USE_RP2040 OR USE_RP2350))` guard in `esprit/rust/CMakeLists.txt`.
//!
//! The crate used to also export a set of panicking `lni2c_*` C-ABI stubs, so
//! that the old `rust_esprit::I2c` C wrapper had something to link against.
//! That wrapper is gone (M7b): nothing in Rust or C++ references those symbols
//! on this platform, so the stubs were deleted rather than kept as traps.
//!
//! What remains is the contract implementation below: every call returns
//! [`Error::Unsupported`](rs_mcu_traits::Error::Unsupported) instead of panicking,
//! and the gap is visible in `rs_mcus/rp2xxx/README.md`.

#![no_std]

// ---------------------------------------------------------------------------
//  The MCU-agnostic I²C contract
// ---------------------------------------------------------------------------

/// The RP2xxx I²C driver (DW_apb on `I2C0` / `I2C1`) is not written yet.
///
/// This type exists so the contract is *implemented*: a caller gets
/// [`Error::Unsupported`](rs_mcu_traits::Error::Unsupported) instead of a panic or
/// a link error, and the gap is visible in `rs_mcus/rp2xxx/README.md` rather than
/// hidden behind an `unimplemented!`.
pub struct I2c;

impl rs_mcu_traits::i2c::I2c for I2c {
    fn configure(&mut self, _config: &rs_mcu_traits::i2c::I2cConfig) -> rs_mcu_traits::Result<()> {
        Err(rs_mcu_traits::Error::Unsupported)
    }

    fn set_speed(&mut self, _speed_hz: u32) -> rs_mcu_traits::Result<()> {
        Err(rs_mcu_traits::Error::Unsupported)
    }

    fn transfer(
        &mut self,
        _target: u8,
        _operations: &mut [rs_mcu_traits::i2c::Operation<'_>],
    ) -> rs_mcu_traits::Result<()> {
        Err(rs_mcu_traits::Error::Unsupported)
    }

    fn set_handler(&mut self, _handler: Option<&'static dyn rs_mcu_traits::i2c::I2cHandler>) {}
}

