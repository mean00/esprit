//! Constants used by the pin mapping tables (no magic numbers in the logic).

// --- Timer indices (same convention as lnPinMapping.cpp: TIM1 = 0 ... TIM5 = 4) ---
pub const TIM1: u8 = 0;
pub const TIM2: u8 = 1;
pub const TIM3: u8 = 2;
pub const TIM4: u8 = 3;
pub const TIM5: u8 = 4;

// --- Timer channels (0 based) ---
pub const CH1: u8 = 0;
pub const CH2: u8 = 1;
pub const CH3: u8 = 2;
pub const CH4: u8 = 3;

// --- ADC channels ---
pub const ADC_IN0: u8 = 0;
pub const ADC_IN1: u8 = 1;
pub const ADC_IN2: u8 = 2;
pub const ADC_IN3: u8 = 3;
pub const ADC_IN4: u8 = 4;
pub const ADC_IN5: u8 = 5;
pub const ADC_IN6: u8 = 6;
pub const ADC_IN7: u8 = 7;
pub const ADC_IN8: u8 = 8;
pub const ADC_IN9: u8 = 9;
pub const ADC_IN10: u8 = 10;
pub const ADC_IN11: u8 = 11;
pub const ADC_IN12: u8 = 12;
pub const ADC_IN13: u8 = 13;
pub const ADC_IN14: u8 = 14;
pub const ADC_IN15: u8 = 15;

// --- DAC channels ---
pub const DAC_OUT0: u8 = 0;
pub const DAC_OUT1: u8 = 1;

// --- DMA engines ---
pub const DMA_ENGINE0: u8 = 0;
pub const DMA_ENGINE1: u8 = 1;

// --- Peripheral instances (0 based) ---
pub const I2C0: u8 = 0;
pub const I2C1: u8 = 1;
pub const SPI0: u8 = 0;
pub const SPI1: u8 = 1;
pub const SPI2: u8 = 2;
pub const UART0: u8 = 0;
pub const UART1: u8 = 1;
pub const UART2: u8 = 2;

// --- Pin numbering (matches rs_gpio_bluepill::Pin: port * PINS_PER_PORT + bit) ---
pub const PINS_PER_PORT: u32 = 16;
pub const PORT_A: u32 = 0;
pub const PORT_B: u32 = 1;
pub const PORT_C: u32 = 2;
pub const PORT_D: u32 = 3;

/// Number of pins (PA0..PC15) described by the main mapping table.
pub const TABLE_PINS: usize = (3 * PINS_PER_PORT) as usize;

// --- Pin existence per package ---
/// First PC pin that exists in the 48 pins package (PC13..PC15 only).
pub const PC_FIRST_48: u32 = 13;
/// Last PD pin existing in the 48 pins package (PD0, PD1: oscillator pins).
pub const PD_LAST_48: u32 = 1;
/// Last PD pin existing in the 64 pins package (PD0..PD2).
pub const PD_LAST_64: u32 = 2;
