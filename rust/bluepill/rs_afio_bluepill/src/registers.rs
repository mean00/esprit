#![allow(dead_code)]
#[repr(C)]
pub struct AfioRegisters {
    pub ec: u32,
    pub pcf0: u32,
    pub extiss: [u32; 4],
    pub dummy: u32,
    pub pcf1: u32,
}

impl AfioRegisters {
    #[inline(always)]
    pub fn ptr() -> *mut Self {
        rs_bluepill::AFIO_BASE as *mut Self
    }
}

// --- AFIO_PCF0 register bitfield positions and values ---
// SWJ_CFG: bits [26:24]
pub const AFIO_PCF0_SWJ_CFG_POS: u32 = 24;
pub const AFIO_PCF0_SWJ_CFG_MASK: u32 = 0b111 << AFIO_PCF0_SWJ_CFG_POS;
pub const AFIO_PCF0_SWJ_CFG_FULL: u32 = 0b000 << AFIO_PCF0_SWJ_CFG_POS;
pub const AFIO_PCF0_SWJ_CFG_NO_NJTRST: u32 = 0b001 << AFIO_PCF0_SWJ_CFG_POS;
pub const AFIO_PCF0_SWJ_CFG_SWD_NO_JTAG: u32 = 0b010 << AFIO_PCF0_SWJ_CFG_POS;
pub const AFIO_PCF0_SWJ_CFG_NO_SW_NO_JTAG: u32 = 0b100 << AFIO_PCF0_SWJ_CFG_POS;

// TIMER0..TIMER2 remap positions
pub const AFIO_PCF0_TIMER0_REMAP_POS: u32 = 6;
pub const AFIO_PCF0_TIMER1_REMAP_POS: u32 = 8;
pub const AFIO_PCF0_TIMER2_REMAP_POS: u32 = 10;

pub const AFIO_PCF0_TIMER_REMAP_MASK_2BIT: u32 = 0b11;

pub const AFIO_PCF0_TIMER2_REMAP_MASK: u32 = AFIO_PCF0_TIMER_REMAP_MASK_2BIT << AFIO_PCF0_TIMER2_REMAP_POS;
pub const AFIO_PCF0_TIMER2_REMAP_NO: u32 = 0b00 << AFIO_PCF0_TIMER2_REMAP_POS;
pub const AFIO_PCF0_TIMER2_REMAP_PARTIAL: u32 = 0b10 << AFIO_PCF0_TIMER2_REMAP_POS;
pub const AFIO_PCF0_TIMER2_REMAP_FULL: u32 = 0b11 << AFIO_PCF0_TIMER2_REMAP_POS;
