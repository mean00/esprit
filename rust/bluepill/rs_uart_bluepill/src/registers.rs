#![allow(dead_code)]
use rs_bluepill::{USART1_BASE, USART2_BASE, USART3_BASE};

pub const UART_INSTANCE_0: u32 = 0;
pub const UART_INSTANCE_1: u32 = 1;
pub const UART_INSTANCE_2: u32 = 2;
pub const UART_MAX_INSTANCES: usize = 3;

// --- STAT Register Bits (GD32 / STM32 SR) ---
pub const USART_STAT_PERR: u32 = 1 << 0;     // Parity error
pub const USART_STAT_FERR: u32 = 1 << 1;     // Framing error
pub const USART_STAT_NERR: u32 = 1 << 2;     // Noise error flag
pub const USART_STAT_ORERR: u32 = 1 << 3;    // Overrun error
pub const USART_STAT_ERROR_MASK: u32 = 0x0F;
pub const USART_STAT_IDLEF: u32 = 1 << 4;    // IDLE frame detected
pub const USART_STAT_RBNE: u32 = 1 << 5;     // Read data buffer not empty (RXNE)
pub const USART_STAT_TC: u32 = 1 << 6;       // Transmission complete
pub const USART_STAT_TBE: u32 = 1 << 7;      // Transmit data buffer empty (TXE)
pub const USART_STAT_LBDF: u32 = 1 << 8;     // LIN break detection flag
pub const USART_STAT_CTSF: u32 = 1 << 9;     // CTS flag

// --- CTL0 Register Bits (GD32 / STM32 CR1) ---
pub const USART_CTL0_SBKCMD: u32 = 1 << 0;   // Send break command
pub const USART_CTL0_RWU: u32 = 1 << 1;      // Receiver wakeup from mute mode
pub const USART_CTL0_REN: u32 = 1 << 2;      // Receiver enable (RE)
pub const USART_CTL0_TEN: u32 = 1 << 3;      // Transmitter enable (TE)
pub const USART_CTL0_IDLEIE: u32 = 1 << 4;   // IDLE interrupt enable
pub const USART_CTL0_RBNEIE: u32 = 1 << 5;   // Read data buffer not empty interrupt enable (RXNEIE)
pub const USART_CTL0_TCIE: u32 = 1 << 6;     // Transmission complete interrupt enable
pub const USART_CTL0_TBIE: u32 = 1 << 7;     // Transmitter buffer empty interrupt enable (TXEIE)
pub const USART_CTL0_PERRIE: u32 = 1 << 8;   // Parity error interrupt enable
pub const USART_CTL0_PM: u32 = 1 << 9;       // Parity selection
pub const USART_CTL0_PCEN: u32 = 1 << 10;    // Parity control enable
pub const USART_CTL0_WM: u32 = 1 << 11;      // Wakeup method in mute mode
pub const USART_CTL0_WL: u32 = 1 << 12;      // Word length
pub const USART_CTL0_UEN: u32 = 1 << 13;     // USART enable (UE)

// --- CTL2 Register Bits (GD32 / STM32 CR3) ---
pub const USART_CTL2_DMAR: u32 = 1 << 6;     // DMA enable receiver
pub const USART_CTL2_DMAT: u32 = 1 << 7;     // DMA enable transmitter

// --- Configuration Constants ---
pub const UART_DEFAULT_BAUDRATE: u32 = 115200;
pub const UART0_DMA_TX_CHANNEL: usize = 3;     // DMA0 Channel 3 for USART0 TX
pub const UART1_DMA_TX_CHANNEL: usize = 6;     // DMA0 Channel 6 for USART1 TX

#[repr(C)]
pub struct UartRegisters {
    pub stat: u32,
    pub data: u32,
    pub baud: u32,
    pub ctl0: u32,
    pub ctl1: u32,
    pub ctl2: u32,
    pub gp: u32,
}

impl UartRegisters {
    #[inline(always)]
    pub fn ptr(instance: u32) -> *mut Self {
        match instance {
            UART_INSTANCE_0 => USART1_BASE as *mut Self,
            UART_INSTANCE_1 => USART2_BASE as *mut Self,
            UART_INSTANCE_2 => USART3_BASE as *mut Self,
            _ => USART1_BASE as *mut Self,
        }
    }
}
