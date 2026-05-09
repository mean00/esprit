#![allow(unused_imports)]
#![allow(dead_code)]
extern crate alloc;
use alloc::boxed::Box;
use embedded_hal as hal;
use embedded_hal::i2c::{ErrorKind, ErrorType, SevenBitAddress, TenBitAddress, I2c, Operation};
use embedded_hal::i2c::ErrorKind::*;


use core::fmt::Error;

use core::convert::Infallible;
use hal::delay::DelayUs;
use hal::digital::*;
//---------------------------------
use crate::rn_os_helper::{delay_ms,delay_us};
use crate::rn_gpio as gpio;
use crate::rust_esprit::lnPin;
//---------------------------------
use super::*;

struct rn_hal
{

}
///-------------------------------------------------
///
/// ** PINS ***
/// 
///-------------------------------------------------
struct Pin
{
    pin: lnPin,
}
///
/// 
/// 

impl embedded_hal::digital::ErrorType for Pin {
       type Error = Error;
}
impl Pin
{
    pub fn new( p : lnPin) -> Self
    {
        Pin {
            pin : p , 
        }
    }
}
///
/// 
/// 
impl InputPin for Pin
{       
    fn is_high(&self) ->   Result<bool, Self::Error>
    {
        Ok(gpio::digitalRead(self.pin))
    }
    fn is_low(&self) ->   Result<bool, Self::Error>
    {
        Ok(!gpio::digitalRead(self.pin))
    }
}
impl OutputPin for Pin
{       
    fn set_high(&mut self) ->   Result<(), Self::Error>
    {
        gpio::digital_write(self.pin,true);
        Ok(())
    }
    fn set_low(&mut self) ->   Result<(), Self::Error>
    {
        gpio::digital_write(self.pin,false);
        Ok(())
    }
}
///
/// 
/// 
impl ToggleableOutputPin for Pin
{       
    fn toggle(&mut self) -> Result<(), Self::Error>
    {
        gpio::digital_toggle(self.pin);
        Ok(())
    }
}
///
/// 
/// 
/// 
impl StatefulOutputPin for Pin
{
    /// Is the pin in drive high mode?
    ///
    /// *NOTE* this does *not* read the electrical state of the pin
    fn is_set_high(&self) -> Result<bool, Self::Error>
    {
        Ok(gpio::digital_read(self.pin))
    }

    /// Is the pin in drive low mode?
    ///
    /// *NOTE* this does *not* read the electrical state of the pin
    fn is_set_low(&self) -> Result<bool, Self::Error>
    {
        Ok(!gpio::digital_read(self.pin))
    }
}

///-------------------------------------------------
///
/// ** Delay ***
/// 
///-------------------------------------------------
impl DelayUs for rn_hal
{
    type Error = Infallible;

    fn delay_ms(&mut self, ms: u32) -> Result<(), Self::Error>
    {
        delay_ms(ms);
        Ok(())
    }
    fn delay_us(&mut self, us: u32) -> Result<(), Self::Error>
    {
        delay_us(us );
        Ok(())
    }
}

/*
*/
