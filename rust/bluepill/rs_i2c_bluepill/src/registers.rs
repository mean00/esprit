#![allow(dead_code)]
use rs_bluepill::{I2C1_BASE, I2C2_BASE};

#[repr(C)]
pub struct I2cRegisters {
    pub cr1: u32,
    pub cr2: u32,
    pub oar1: u32,
    pub oar2: u32,
    pub dr: u32,
    pub sr1: u32,
    pub sr2: u32,
    pub ccr: u32,
    pub trise: u32,
}

impl I2cRegisters {
    #[inline(always)]
    pub fn ptr(instance: u32) -> *mut Self {
        match instance {
            I2C_INSTANCE_0 => I2C1_BASE as *mut Self,
            I2C_INSTANCE_1 => I2C2_BASE as *mut Self,
            _ => I2C1_BASE as *mut Self,
        }
    }
}

pub const I2C_INSTANCE_0: u32 = 0;
pub const I2C_INSTANCE_1: u32 = 1;
pub const I2C_MAX_INSTANCES: usize = 2;

// --- CR1 / CTL0 Register Bits ---
pub const I2C_CR1_PE: u32 = 1 << 0;          // Peripheral enable
pub const I2C_CR1_START: u32 = 1 << 8;       // Generate START condition
pub const I2C_CR1_STOP: u32 = 1 << 9;        // Generate STOP condition
pub const I2C_CR1_ACK: u32 = 1 << 10;        // Acknowledge enable

// --- CR2 / CTL1 Register Bits ---
pub const I2C_CR2_ITERREN: u32 = 1 << 8;     // Error interrupt enable
pub const I2C_CR2_ITEVTEN: u32 = 1 << 9;     // Event interrupt enable
pub const I2C_CR2_ITBUFEN: u32 = 1 << 10;    // Buffer interrupt enable
pub const I2C_CR2_DMAEN: u32 = 1 << 11;      // DMA requests enable

// --- SR1 / STAT0 Register Bits ---
pub const I2C_SR1_SB: u32 = 1 << 0;          // Start bit generated
pub const I2C_SR1_ADDR: u32 = 1 << 1;        // Address sent/matched
pub const I2C_SR1_BTF: u32 = 1 << 2;         // Byte transfer finished
pub const I2C_SR1_ADD10: u32 = 1 << 3;       // 10-bit header sent
pub const I2C_SR1_STOPF: u32 = 1 << 4;       // Stop detection
pub const I2C_SR1_RXNE: u32 = 1 << 6;        // Rx buffer not empty
pub const I2C_SR1_TXE: u32 = 1 << 7;         // Tx buffer empty
pub const I2C_SR1_BERR: u32 = 1 << 8;        // Bus error
pub const I2C_SR1_ARLO: u32 = 1 << 9;        // Arbitration lost
pub const I2C_SR1_AF: u32 = 1 << 10;         // Acknowledge failure
pub const I2C_SR1_OVR: u32 = 1 << 11;        // Overrun/underrun
pub const I2C_SR1_PECERR: u32 = 1 << 12;     // PEC error
pub const I2C_SR1_TIMEOUT: u32 = 1 << 14;    // Timeout error
pub const I2C_SR1_SMBALERT: u32 = 1 << 15;   // SMBus alert

// Combined error flags in SR1 (BERR | ARLO | AF | OVR) = 0x0F00
pub const I2C_SR1_ERROR_MASK: u32 = I2C_SR1_BERR | I2C_SR1_ARLO | I2C_SR1_AF | I2C_SR1_OVR;

// --- SR2 / STAT1 Register Bits ---
pub const I2C_SR2_MSL: u32 = 1 << 0;         // Master/slave mode
pub const I2C_SR2_BUSY: u32 = 1 << 1;        // Bus busy
pub const I2C_SR2_TRA: u32 = 1 << 2;         // Transmitter/receiver

// --- Addressing Constants ---
pub const I2C_ADDRESS_7BIT_MASK: u32 = 0x7F;
pub const I2C_ADDRESS_7BIT_SHIFT: u32 = 1;

// --- GD32 Aliases ---
pub const I2C_CTL0_I2CEN: u32 = I2C_CR1_PE;
pub const I2C_CTL0_START: u32 = I2C_CR1_START;
pub const I2C_CTL0_STOP: u32 = I2C_CR1_STOP;
pub const I2C_CTL0_ACKEN: u32 = I2C_CR1_ACK;

pub const I2C_CTL1_ERRIE: u32 = I2C_CR2_ITERREN;
pub const I2C_CTL1_EVIE: u32 = I2C_CR2_ITEVTEN;
pub const I2C_CTL1_BUFIE: u32 = I2C_CR2_ITBUFEN;
pub const I2C_CTL1_DMAON: u32 = I2C_CR2_DMAEN;

pub const I2C_STAT0_SBSEND: u32 = I2C_SR1_SB;
pub const I2C_STAT0_ADDSEND: u32 = I2C_SR1_ADDR;
pub const I2C_STAT0_BTC: u32 = I2C_SR1_BTF;
pub const I2C_STAT0_RBNE: u32 = I2C_SR1_RXNE;
pub const I2C_STAT0_TBE: u32 = I2C_SR1_TXE;
pub const I2C_STAT0_BERR: u32 = I2C_SR1_BERR;
pub const I2C_STAT0_LOSTARB: u32 = I2C_SR1_ARLO;
pub const I2C_STAT0_AERR: u32 = I2C_SR1_AF;
pub const I2C_STAT0_OUERR: u32 = I2C_SR1_OVR;
pub const I2C_STAT0_ERROR_MASK: u32 = I2C_SR1_ERROR_MASK;
