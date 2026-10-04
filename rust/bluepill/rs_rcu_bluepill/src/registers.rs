#![allow(dead_code)]
#[repr(C)]
pub struct RcuRegisters {
    pub ctl: u32,      // 00 CR
    pub cfg0: u32,     // 04 CFGR
    pub int: u32,      // 08 CIR
    pub apb2rst: u32,  // 0c APB2RSTR
    pub apb1rst: u32,  // 10 APB1RSTR
    pub ahben: u32,    // 14 AHBENR
    pub apb2en: u32,   // 18 APB2ENR
    pub apb1en: u32,   // 1c APB1ENR
    pub bdctl: u32,    // 20 BDCR
    pub rstclk: u32,   // 24 CSR
    pub ahbrst: u32,   // 28 AHBRSTR
    pub cfg1: u32,     // 2c CFGR2
    pub dsv: u32,      // 30 N/A
}

impl RcuRegisters {
    #[inline(always)]
    pub fn ptr() -> *mut Self {
        rs_bluepill::RCU_BASE as *mut Self
    }
}

// --- Bus Identifiers ---
pub const BUS_APB1: u8 = 1;
pub const BUS_APB2: u8 = 2;
pub const BUS_AHB: u8 = 8;

// --- APB1 Peripheral Enable / Reset Masks ---
pub const RCU_APB1_TIMER1: u32 = 1 << 0;
pub const RCU_APB1_TIMER2: u32 = 1 << 1;
pub const RCU_APB1_TIMER3: u32 = 1 << 2;
pub const RCU_APB1_TIMER4: u32 = 1 << 3;
pub const RCU_APB1_TIMER5: u32 = 1 << 4;
pub const RCU_APB1_TIMER6: u32 = 1 << 5;
pub const RCU_APB1_WWDGT: u32 = 1 << 11;
pub const RCU_APB1_SPI1: u32 = 1 << 14;
pub const RCU_APB1_SPI2: u32 = 1 << 15;
pub const RCU_APB1_USART1: u32 = 1 << 17;
pub const RCU_APB1_USART2: u32 = 1 << 18;
pub const RCU_APB1_USART3: u32 = 1 << 19;
pub const RCU_APB1_USART4: u32 = 1 << 20;
pub const RCU_APB1_I2C0: u32 = 1 << 21;
pub const RCU_APB1_I2C1: u32 = 1 << 22;
pub const RCU_APB1_USBD: u32 = 1 << 23;
pub const RCU_APB1_CAN0: u32 = 1 << 25;
pub const RCU_APB1_CAN1: u32 = 1 << 26;
pub const RCU_APB1_BKPI: u32 = 1 << 27;
pub const RCU_APB1_PMU: u32 = 1 << 28;
pub const RCU_APB1_DAC: u32 = 1 << 29;

// --- APB2 Peripheral Enable / Reset Masks ---
pub const RCU_APB2_AF: u32 = 1 << 0;
pub const RCU_APB2_PA: u32 = 1 << 2;
pub const RCU_APB2_PB: u32 = 1 << 3;
pub const RCU_APB2_PC: u32 = 1 << 4;
pub const RCU_APB2_PD: u32 = 1 << 5;
pub const RCU_APB2_PE: u32 = 1 << 6;
pub const RCU_APB2_ADC0: u32 = 1 << 9;
pub const RCU_APB2_ADC1: u32 = 1 << 10;
pub const RCU_APB2_TIMER0: u32 = 1 << 11;
pub const RCU_APB2_SPI0: u32 = 1 << 12;
pub const RCU_APB2_USART0: u32 = 1 << 14;

// --- AHB Peripheral Enable / Reset Masks ---
pub const RCU_AHB_DMA0: u32 = 1 << 0;
pub const RCU_AHB_DMA1: u32 = 1 << 1;
pub const RCU_AHB_USBHS_CH32V3X: u32 = 1 << 11;
pub const RCU_AHB_USBFS_OTG_CH32V3X: u32 = 1 << 12;
pub const RCU_AHB_ETHMAC: u32 = 7 << 14;
pub const RCU_AHBRST_ETHMAC: u32 = 1 << 14;
