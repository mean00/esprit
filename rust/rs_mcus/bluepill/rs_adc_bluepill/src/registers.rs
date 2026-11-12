#![allow(dead_code)]

use rs_bluepill::adc_base;

#[repr(C)]
pub struct AdcRegisters {
    pub stat: u32,
    pub ctl0: u32,
    pub ctl1: u32,
    pub sampt0: u32,
    pub sampt1: u32,
    pub iofr0: u32,
    pub iofr1: u32,
    pub iofr2: u32,
    pub iofr3: u32,
    /// Analog watchdog high threshold (WDHT, 0x24).
    pub wdht: u32,
    /// Analog watchdog low threshold (WDLT, 0x28). Its own register, not part of
    /// WDHT: without it every field below (RSQ0/RSQ1/RSQ2, ISQ, IDATA, RDATA) is
    /// 4 bytes short and a conversion's result is read from IDATA3 instead of RDATA.
    pub wdlt: u32,
    pub rsq0: u32,
    pub rsq1: u32,
    pub rsq2: u32,
    pub isq: u32,
    pub idata0: u32,
    pub idata1: u32,
    pub idata2: u32,
    pub idata3: u32,
    pub rdata: u32,
}

impl AdcRegisters {
    /// Register block of ADC `instance` (`0` = ADC1, `1` = ADC2, `2` = ADC3).
    ///
    /// The third ADC exists on the ARM side only (STM32F1 high-density /
    /// GD32F30x); CH32V3 has two, so its map answers `None` and this panics with
    /// the same message as an out-of-range instance would.
    #[inline(always)]
    pub fn ptr(instance: u32) -> *mut Self {
        match adc_base(instance) {
            Some(base) => base as *mut Self,
            None => panic!("Invalid ADC instance"),
        }
    }
}

// --- STAT register bits ---
pub const ADC_STAT_WDE: u32 = 1 << 0;  // Watchdog flag
pub const ADC_STAT_EOC: u32 = 1 << 1;  // End of conversion
pub const ADC_STAT_EOIC: u32 = 1 << 2; // End of inserted conversion
pub const ADC_STAT_STIC: u32 = 1 << 3; // Start inserted conversion flag
pub const ADC_STAT_STRC: u32 = 1 << 4; // Start regular conversion flag

// --- CTL0 register bits ---
pub const ADC_CTL0_SM: u32 = 1 << 8;   // Scan mode enable

// --- CTL1 register bits ---
pub const ADC_CTL1_ADCON: u32 = 1 << 0;   // ADC converter enable / power-on
pub const ADC_CTL1_CTN: u32 = 1 << 1;     // Continuous conversion mode
pub const ADC_CTL1_CLB: u32 = 1 << 2;     // Calibration start
pub const ADC_CTL1_RSTCLB: u32 = 1 << 3;  // Reset calibration
pub const ADC_CTL1_DMA: u32 = 1 << 8;     // DMA request enable
pub const ADC_CTL1_ETERC: u32 = 1 << 20;  // External trigger enable for regular channels
pub const ADC_CTL1_SWRCST: u32 = 1 << 22; // Start conversion on regular channel (software trigger)
pub const ADC_CTL1_TSVREN: u32 = 1 << 23; // Temperature sensor and Vrefint enable

// External trigger source selection (ETSRC) bits [19:17]
pub const ADC_CTL1_ETSRC_POS: u32 = 17;
pub const ADC_CTL1_ETSRC_MASK: u32 = 0x7 << ADC_CTL1_ETSRC_POS;
pub const ADC_CTL1_ETSRC_T0CH0: u32 = 0 << ADC_CTL1_ETSRC_POS;
pub const ADC_CTL1_ETSRC_T0CH1: u32 = 1 << ADC_CTL1_ETSRC_POS;
pub const ADC_CTL1_ETSRC_T0CH2: u32 = 2 << ADC_CTL1_ETSRC_POS;
pub const ADC_CTL1_ETSRC_T1CH1: u32 = 3 << ADC_CTL1_ETSRC_POS;
pub const ADC_CTL1_ETSRC_T2TRGO: u32 = 4 << ADC_CTL1_ETSRC_POS;
pub const ADC_CTL1_ETSRC_T3CH3: u32 = 5 << ADC_CTL1_ETSRC_POS;
pub const ADC_CTL1_ETSRC_EXTI11: u32 = 6 << ADC_CTL1_ETSRC_POS;
pub const ADC_CTL1_ETSRC_SWSTART: u32 = 7 << ADC_CTL1_ETSRC_POS;

// --- RSQ0 register bitfields ---
pub const ADC_RSQ0_LEN_POS: u32 = 20;
pub const ADC_RSQ0_LEN_MASK: u32 = 0x0F << ADC_RSQ0_LEN_POS;

// --- Sampling time constants ---
pub const ADC_SAMPT_239_5: u32 = 7;
/// Channels 0..=9 take their sampling time from SAMPT0, 10..=17 from SAMPT1.
pub const ADC_SAMPT0_CHANNELS: u32 = 10;
/// Bits per channel in SAMPT0 / SAMPT1.
pub const ADC_SAMPT_BITS: u32 = 3;
/// A channel's field in SAMPT0 / SAMPT1.
pub const ADC_SAMPT_MASK: u32 = 0x7;

// --- RDATA bitmask ---
pub const ADC_RDATA_DATA_MASK: u32 = 0x0FFF;

// --- DMA Channel Mapping ---
pub const ADC0_DMA_ENGINE: rs_dma_bluepill::DmaEngine = rs_dma_bluepill::DmaEngine::Dma0;
pub const ADC0_DMA_CHANNEL_IDX: usize = 0; // DMA0 Channel 0 is ADC0
