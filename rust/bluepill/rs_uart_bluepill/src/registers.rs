#![allow(dead_code)]
#![allow(dead_code)]

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
            0 => 0x40013800 as *mut Self, // USART1 (instance 0)
            1 => 0x40004400 as *mut Self, // USART2 (instance 1)
            2 => 0x40004800 as *mut Self, // USART3 (instance 2)
            _ => panic!("Invalid UART instance"),
        }
    }
}
