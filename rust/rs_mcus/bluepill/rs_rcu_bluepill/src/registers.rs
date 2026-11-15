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
    _reserved_34_bc: [u32; 35],
    pub addctl: u32,   // c0 Additional clock control (GD32)
}

const _: () = assert!(core::mem::offset_of!(RcuRegisters, cfg0) == 0x04);
const _: () = assert!(core::mem::offset_of!(RcuRegisters, dsv) == 0x30);
const _: () = assert!(core::mem::offset_of!(RcuRegisters, addctl) == 0xC0);

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
//
// Only the blocks both families have. The CH32V3-only ones (USBHS / USB-FS-OTG /
// RNG / SDIO) are in `registers_ch32v3x.rs`; the ARM side answers `None` for them
// in `registers_stm32.rs`.

pub const RCU_AHB_DMA0: u32 = 1 << 0;
pub const RCU_AHB_DMA1: u32 = 1 << 1;
/// USB full-speed, the bit both families decode (CH32V3 calls it USBFSOG).
pub const RCU_AHB_USBFS: u32 = 1 << 12;
pub const RCU_AHB_ETHMAC: u32 = 7 << 14;
pub const RCU_AHBRST_ETHMAC: u32 = 1 << 14;

// --- CFG0 ADC Prescaler (bits [15:14]) ---
pub const LN_RCU_CFG0_ADCPSC_MASK: u32 = 0x3 << 14;
pub const LN_RCU_CFG0_ADCPSC_DIV8: u32 = 0x3 << 14;

// --- CFG0 USB Prescaler ---
pub const LN_RCU_CFG0_USBPSC_SHIFT: u32 = 22;
pub const LN_RCU_CFG0_USBPSC_BITS: u32 = 3;
pub const LN_RCU_CFG0_USBPSC_MASK: u32 = !(LN_RCU_CFG0_USBPSC_BITS << LN_RCU_CFG0_USBPSC_SHIFT);

// --- ADDCTL (GD32F303, offset 0xC0) ---
pub const LN_GD_RCU_ADDCTL_IRC48M_SEL: u32 = 1 << 0;
pub const LN_GD_RCU_ADDCTL_IRC48M_EN: u32 = 1 << 16;
pub const LN_GD_RCU_ADDCTL_IRC48M_STB: u32 = 1 << 17;

// --- CTL / CFG0 System Clock Bits (GD32F3x / STM32F1) ---
pub const LN_CLOCK_IRC8: u32 = 0;
pub const LN_CLOCK_XTAL: u32 = 16;
pub const LN_CLOCK_PLL: u32 = 24;

pub const LN_RCU_CTL_HXTALEN: u32 = 1 << 16;
pub const LN_RCU_CTL_HXTASTB: u32 = 1 << 17;
pub const LN_RCU_CTL_PLLEN: u32 = 1 << 24;
pub const LN_RCU_CTL_PLLSTB: u32 = 1 << 25;

pub const LN_RCU_CFG0_SYSCLOCK_MASK: u32 = 3;
pub const LN_RCU_CFG0_SYSCLOCK_PLL: u32 = 1 << 1;
pub const LN_RCU_CFG0_PLLSEL: u32 = 1 << 16;
pub const LN_RCU_CFG0_PLLMF_SHIFT: u32 = 18;
pub const LN_RCU_CFG0_PLLMF_MASK: u32 = 0xF << 18;
pub const LN_RCU_CFG0_PLLMF_MSB: u32 = 1 << 27;
pub const LN_RCU_CFG0_AHBPSC_MASK: u32 = 0xF << 4;
pub const LN_RCU_CFG0_APB1PSC_MASK: u32 = 7 << 8;
pub const LN_RCU_CFG0_APB1PSC_DIV2: u32 = 4 << 8;
pub const LN_RCU_CFG0_APB2PSC_MASK: u32 = 3 << 11;

pub const FMC_WS_PREFETCH_BASE: u32 = 0x30;
pub const DBG_CTL0_HALT_TIMERS_I2C: u32 = (0xF << 10) + (1 << 15) + (0x1F << 16) + (0x3F << 25);
pub const LN_RCU_RSTCLK_RSTFC: u32 = 1 << 24;
