use crate::*;

const C_NONE: i32 = -1;

#[unsafe(no_mangle)]
pub extern "C" fn rs_pinout_adc(pin: u32) -> i32 {
    adc_channel(Pin::from(pin)).map_or(C_NONE, i32::from)
}

#[unsafe(no_mangle)]
pub extern "C" fn rs_pinout_dac(pin: u32) -> i32 {
    dac_channel(Pin::from(pin)).map_or(C_NONE, i32::from)
}

#[unsafe(no_mangle)]
pub extern "C" fn rs_pinout_timer(pin: u32) -> i32 {
    timer_channel(Pin::from(pin)).map_or(C_NONE, |t| i32::from(t.timer))
}

#[unsafe(no_mangle)]
pub extern "C" fn rs_pinout_timer_channel(pin: u32) -> i32 {
    timer_channel(Pin::from(pin)).map_or(C_NONE, |t| i32::from(t.channel))
}

#[unsafe(no_mangle)]
pub extern "C" fn rs_pinout_timer_dma_engine(timer: u8, channel: u8) -> i32 {
    timer_dma(timer, channel).map_or(C_NONE, |d| i32::from(d.engine))
}

#[unsafe(no_mangle)]
pub extern "C" fn rs_pinout_timer_dma_channel(timer: u8, channel: u8) -> i32 {
    timer_dma(timer, channel).map_or(C_NONE, |d| i32::from(d.channel))
}
