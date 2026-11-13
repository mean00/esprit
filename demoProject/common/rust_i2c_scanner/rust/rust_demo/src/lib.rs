#![no_std]
use rust_esprit::delay_ms;
use rust_esprit::{HalI2c, I2cConfig, I2cDriver, logger, logger_init};

logger_init!();

#[cfg(feature = "rp2040")]
compile_error!(
    "this demo drives the GD32/CH32 I2C backend (bluepill48); the RP2xxx I2C driver has no \
     constructor in esprit yet"
);

const I2C_INSTANCE: u32 = 0;
const I2C_SPEED_HZ: u32 = 200_000;
const FIRST_ADDRESS: u8 = 0;
const LAST_ADDRESS: u8 = 0x7F;

/**
 * Probe every 7-bit address: an ACKed empty write means something answered.
 *
 * The port is the platform driver (`rs_i2c_bluepill`) and the traffic goes
 * through the `rs_mcu_traits::i2c::I2c` super trait, which is what takes the
 * target address per transaction: there is no `set_address` state to keep.
 */
#[unsafe(no_mangle)]
extern "C" fn user_init() {
    logger!("I2C Scanner !\n");
    let mut i2c = I2cDriver::new(I2C_INSTANCE, I2C_SPEED_HZ);
    let _ = i2c.configure(&I2cConfig {
        speed_hz: I2C_SPEED_HZ,
        own_address: 0,
    });
    loop {
        for dex in FIRST_ADDRESS..=LAST_ADDRESS {
            if HalI2c::write_to(&mut i2c, dex, &[]).is_ok() {
                logger!("Found a i2c device at address {} 0x{:x}\n", dex, dex);
            } else {
                delay_ms(100);
            }
        }
    }
}

// EOF

