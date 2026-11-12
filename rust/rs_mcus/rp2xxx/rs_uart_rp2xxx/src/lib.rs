//! UART (PL011) driver for the RP2040 and the RP2350.
//!
//! The RP2xxx has two PrimeCells (`UART0`, `UART1`) with the same layout, both
//! clocked from `clk_peri`. This crate is the family implementation of
//! [`rs_mcu_traits::uart::Uart`]: blocking writes, draining reads, the frame/rate
//! configuration and a per-instance handler slot.
//!
//! * `configure` / `set_baudrate` program `IBRD`/`FBRD`/`LCR_H`/`CR` from the
//!   peripheral clock (`rs_rcu_rp2xxx`), so 115200 8N1 at 125 MHz is exact to one
//!   fractional divisor step (0.006 %).
//! * `write` polls `FR.TXFF`, `flush` polls `FR.BUSY`, `try_write` fills the
//!   transmit FIFO up to its free depth.
//! * With [`Uart::with_dma`] the receive path is interrupt driven: `RXIM`/`RTIM`
//!   are enabled on the instance's NVIC line, the ISR drains the FIFO into a
//!   [`ring::Ring`] and reports it with [`uart::UartHandler::on_rx_available`],
//!   so a bridge task can sleep until data arrives instead of polling `read`.

#![no_std]

pub mod registers;
pub mod ring;

pub use registers::{UART_INSTANCES, base};

use portable_atomic::{AtomicBool, Ordering};
use ring::Ring;
use rs_gpio_rp2xxx::Pin;
use rs_mcu_traits::handler::HandlerTable;
use rs_mcu_traits::uart::{self, Parity, StopBits, UartConfig, UartHandler};
use rs_mcu_traits::{Error, Result};
use rs_rcu_rp2xxx::{Peripheral, Rcu};

/// Offset of a PL011 register inside the block (`registers.rs` owns the layout).
macro_rules! reg {
    ($member:ident) => {
        core::mem::offset_of!(registers::UartRegisters, $member) as u32
    };
}

/// One handler slot per UART instance.
static HANDLERS: HandlerTable<dyn UartHandler, UART_INSTANCES> = HandlerTable::new();

/// Receive rings, one per instance (`.bss`): the ISR is the only producer, the
/// task's `read` the only consumer.
static RX_RINGS: [Ring<RX_RING_SIZE>; UART_INSTANCES] = [const { Ring::new() }; UART_INSTANCES];

/// One ISR installation flag per instance.
static IRQ_INSTALLED: [AtomicBool; UART_INSTANCES] =
    [const { AtomicBool::new(false) }; UART_INSTANCES];

unsafe extern "C" {
    /// Install the ISR of an NVIC line (provided by the RP2xxx runtime).
    fn irq_set_exclusive_handler(num: u32, handler: unsafe extern "C" fn());
    /// Enable or disable an NVIC line (provided by the RP2xxx runtime).
    fn irq_set_enabled(num: u32, enabled: bool);
}

/// Frame rate applied by [`Uart::new`], matching [`UartConfig::default`].
const DEFAULT_BAUD: u32 = 115_200;

/// Size of each instance's receive ring. Same order as the C++ driver's default
/// `rxBufferSize` (128 → 256 in the Rust backend), and a power of two.
pub const RX_RING_SIZE: usize = 256;

/// NVIC line of each UART (RP2040 §2.3.2 Table 24: UART0 = 20, UART1 = 21;
/// RP2350 §3.2: UART0 = 33, UART1 = 34).
#[cfg(feature = "rp2350")]
const fn nvic_irq(instance: u32) -> u32 {
    if instance == 0 { 33 } else { 34 }
}

/// See the `rp2350` variant above.
#[cfg(not(feature = "rp2350"))]
const fn nvic_irq(instance: u32) -> u32 {
    if instance == 0 { 20 } else { 21 }
}

/// Interrupts this driver arms on an interrupt-driven instance: receive FIFO
/// passed its level, receive timeout (a short frame), and overrun.
const UART_INT_RX_MASK: u32 =
    registers::UART_INT_RX | registers::UART_INT_RT | registers::UART_INT_OE;

/// Spins allowed while waiting for a FIFO flag: a byte at 115200 baud takes 87 µs,
/// so this is generous without being able to hang the caller forever.
const POLL_LIMIT: u32 = 2_000_000;

/// One PL011.
pub struct Uart {
    instance: u32,
    tx: Pin,
    rx: Pin,
}

impl Uart {
    /// Take `uart0`/`uart1` with the given pads (`instance` is `0` or `1`).
    ///
    /// The pads are muxed to the UART function; which instance a pad reaches is a
    /// property of the pad (datasheet §2.19.2 Table 279), so choosing a pad that
    /// belongs to the other UART is a board-table bug this driver cannot detect.
    /// The object comes up in the default 115200 8N1 frame, so it can be written
    /// to immediately.
    ///
    /// Panics on an instance the chip does not have, like the other family drivers.
    pub fn new(instance: u32, tx: Pin, rx: Pin) -> Self {
        assert!(base(instance).is_some(), "the RP2xxx has two UARTs");
        Rcu::enable(peripheral(instance));
        let mut uart = Self { instance, tx, rx };
        uart.tx.set_function(registers::FUNCSEL_UART);
        uart.rx.set_function(registers::FUNCSEL_UART);
        let _ = uart.program_baud(DEFAULT_BAUD);
        let _ = uart.program_frame(&UartConfig::default());
        uart
    }

    /// This UART's number (`0` or `1`).
    #[inline(always)]
    pub fn instance(&self) -> u32 {
        self.instance
    }

    /// Take `uart0`/`uart1` on the chip's default pads and drive the receiver
    /// from the interrupt.
    ///
    /// Default pads are the ones the RP2xxx SDK assigns: `UART0` = `GPIO0` (TX)
    /// / `GPIO1` (RX), `UART1` = `GPIO4` / `GPIO5`. The receive FIFO is enabled,
    /// `RXIM`/`RTIM`/`OEIM` are unmasked on the instance's NVIC line, and the
    /// ISR moves every byte into this instance's [`ring::Ring`] before calling
    /// [`rs_mcu_traits::uart::UartHandler::on_rx_available`] — so a task can
    /// block on an event group instead of polling [`uart::Uart::read`].
    ///
    /// This is the configuration a pass-through bridge needs; plain polling
    /// users call [`Uart::new`] with their own pads and get no interrupt.
    pub fn with_dma(instance: u32, config: &UartConfig) -> Result<Self> {
        let (tx, rx) = match instance {
            0 => (Pin::GPIO0, Pin::GPIO1),
            1 => (Pin::GPIO4, Pin::GPIO5),
            _ => return Err(Error::InvalidConfig),
        };
        let mut uart = Self::new(instance, tx, rx);
        uart::Uart::configure(&mut uart, config)?;
        RX_RINGS[instance as usize].clear();
        enable_rx_interrupt(instance);
        Ok(uart)
    }

    /// The pads, for a board that wants them back.
    #[inline(always)]
    pub fn pins(&self) -> (Pin, Pin) {
        (self.tx, self.rx)
    }
}

/// Register access by instance, for the code that has no `Uart` value (the
/// interrupt handlers).
fn reg_read(instance: u32, offset: u32) -> u32 {
    // SAFETY: `offset` comes from `reg!`, so it is a register of the PL011 block
    // at this instance's base (`base(instance)` was checked when the port was
    // opened).
    unsafe { core::ptr::read_volatile((base(instance).expect("instance") + offset) as *const u32) }
}

/// Write side of [`reg_read`].
fn reg_write(instance: u32, offset: u32, value: u32) {
    // SAFETY: same as `reg_read`.
    unsafe {
        core::ptr::write_volatile(
            (base(instance).expect("instance") + offset) as *mut u32,
            value,
        )
    }
}

/// Move everything the receive FIFO holds into `instance`'s ring and wake the
/// handler. Called from the instance's interrupt.
fn drain_fifo_into_ring(instance: u32) {
    let ring = &RX_RINGS[instance as usize];
    let mut got = 0;
    while reg_read(instance, reg!(fr)) & registers::UART_FR_RXFE == 0 {
        let byte = (reg_read(instance, reg!(dr)) & 0xFF) as u8;
        if !ring.push(byte) {
            // Ring full: stop draining, the task has to catch up. The PL011 FIFO
            // keeps the bytes, and `read` resumes from it.
            break;
        }
        got += 1;
    }
    if got > 0
        && let Some(handler) = unsafe { HANDLERS.isr_get(instance as usize) }
    {
        handler.on_rx_available();
    }
}

/// Shared body of the two receive ISRs.
fn uart_irq(instance: u32) {
    // Reading `RIS` and writing the same bits to `ICR` is the PL011's
    // acknowledge: `ICR` is write-1-to-clear for every source.
    let status = reg_read(instance, reg!(ris));
    reg_write(instance, reg!(icr), status);

    if status & registers::UART_INT_OE != 0
        && let Some(handler) = unsafe { HANDLERS.isr_get(instance as usize) }
    {
        handler.on_error(Error::Overrun);
    }
    if status & (registers::UART_INT_RX | registers::UART_INT_RT) != 0 {
        drain_fifo_into_ring(instance);
    }
}

/// ISR of `UART0`.
unsafe extern "C" fn uart0_irq() {
    uart_irq(0);
}

/// ISR of `UART1`.
unsafe extern "C" fn uart1_irq() {
    uart_irq(1);
}

/// Install (once) and unmask the receive interrupt of `instance`.
fn enable_rx_interrupt(instance: u32) {
    let index = instance as usize;
    if !IRQ_INSTALLED[index].swap(true, Ordering::SeqCst) {
        let isr = if instance == 0 { uart0_irq } else { uart1_irq };
        // SAFETY: task context, before the mask is set; the runtime owns the
        // vector table and this line was free.
        unsafe {
            irq_set_exclusive_handler(nvic_irq(instance), isr);
            irq_set_enabled(nvic_irq(instance), true);
        }
    }
    let imsc = reg_read(instance, reg!(imsc));
    reg_write(instance, reg!(imsc), imsc | UART_INT_RX_MASK);
}

/// The RCU entry of an instance.
fn peripheral(instance: u32) -> Peripheral {
    if instance == 0 {
        Peripheral::Uart0
    } else {
        Peripheral::Uart1
    }
}
/// Offset of a PL011 register inside the block (`registers.rs` owns the layout).
impl Uart {
    /// `clk_peri` in Hz — the clock the baud divisors are computed from.
    fn clock_hz(&self) -> u32 {
        rs_rcu_rp2xxx::get_clock(peripheral(self.instance)).max(1)
    }

    fn base(&self) -> u32 {
        base(self.instance).expect("instance checked in `new`")
    }

    fn read_reg(&self, offset: u32) -> u32 {
        // SAFETY: `offset` always comes from `reg!`, i.e. it is the offset of a
        // register of the PL011 block at this instance's base (registers.rs).
        unsafe { core::ptr::read_volatile((self.base() + offset) as *const u32) }
    }

    fn write_reg(&self, offset: u32, value: u32) {
        // SAFETY: same as `read_reg`, write side.
        unsafe { core::ptr::write_volatile((self.base() + offset) as *mut u32, value) }
    }

    fn fr(&self) -> u32 {
        self.read_reg(reg!(fr))
    }

    /// Block until `flag` clears; `false` when the poll budget ran out.
    fn wait_clear(&self, flag: u32) -> bool {
        for _ in 0..POLL_LIMIT {
            if self.fr() & flag == 0 {
                return true;
            }
            core::hint::spin_loop();
        }
        false
    }

    /// Program `IBRD`/`FBRD` for `baud`.
    fn program_baud(&self, baud: u32) -> Result<()> {
        let (ibrd, fbrd) = divisors(self.clock_hz(), baud).ok_or(Error::InvalidConfig)?;
        self.write_reg(reg!(ibrd), ibrd as u32);
        self.write_reg(reg!(fbrd), fbrd as u32);
        Ok(())
    }

    /// Program the frame format and enable the UART.
    fn program_frame(&self, config: &UartConfig) -> Result<()> {
        let mut lcr_h = registers::UART_LCR_H_FEN;
        lcr_h |= match config.data_bits {
            8 => registers::UART_LCR_H_WLEN_8,
            7 => registers::UART_LCR_H_WLEN_7,
            _ => return Err(Error::InvalidConfig),
        };
        match config.parity {
            Parity::None => {}
            Parity::Even => lcr_h |= registers::UART_LCR_H_PEN | registers::UART_LCR_H_EPS,
            Parity::Odd => lcr_h |= registers::UART_LCR_H_PEN,
        }
        if config.stop_bits == StopBits::Two {
            lcr_h |= registers::UART_LCR_H_STP;
        }
        self.write_reg(reg!(lcr_h), lcr_h);
        self.write_reg(
            reg!(cr),
            registers::UART_CR_UARTEN | registers::UART_CR_TXE | registers::UART_CR_RXE,
        );
        Ok(())
    }
}

/// `(IBRD, FBRD)` for `baud` on a `fclk_hz` clock, or `None` when the rate is
/// outside what the 16-bit integer divisor can express.
///
/// PL011: `baud = 4 * fclk / (IBRD * 64 + FBRD)`, so the 4×-scaled divisor is
/// `4 * fclk / baud`, rounded to the nearest integer.
fn divisors(fclk_hz: u32, baud: u32) -> Option<(u16, u8)> {
    if baud == 0 {
        return None;
    }
    let scaled = (8 * fclk_hz as u64 + baud as u64) / (2 * baud as u64);
    let ibrd = scaled >> 6;
    if ibrd == 0 || ibrd > u16::MAX as u64 {
        return None;
    }
    Some((ibrd as u16, (scaled & 0x3F) as u8))
}

impl uart::Uart for Uart {
    fn configure(&mut self, config: &UartConfig) -> Result<()> {
        // The UART must be disabled while its frame format changes.
        self.write_reg(reg!(cr), 0);
        self.program_baud(config.baudrate)?;
        self.program_frame(config)
    }

    fn set_baudrate(&mut self, baudrate: u32) -> Result<()> {
        let cr = self.read_reg(reg!(cr));
        self.write_reg(reg!(cr), cr & !registers::UART_CR_UARTEN);
        self.program_baud(baudrate)?;
        self.write_reg(reg!(cr), cr);
        Ok(())
    }

    /// Send every byte, then wait for the shifter to drain.
    fn write(&mut self, data: &[u8]) -> Result<()> {
        for byte in data {
            if !self.wait_clear(registers::UART_FR_TXFF) {
                return Err(Error::Timeout);
            }
            self.write_reg(reg!(dr), *byte as u32);
        }
        self.flush()
    }

    /// Queue as much as the transmit FIFO takes without blocking.
    fn try_write(&mut self, data: &[u8]) -> Result<usize> {
        let mut sent = 0;
        while sent < data.len() && self.fr() & registers::UART_FR_TXFF == 0 {
            self.write_reg(reg!(dr), data[sent] as u32);
            sent += 1;
        }
        Ok(sent)
    }

    fn flush(&mut self) -> Result<()> {
        if self.wait_clear(registers::UART_FR_BUSY) {
            Ok(())
        } else {
            Err(Error::Timeout)
        }
    }

    /// Copy what the receive FIFO holds, calling the instance handler per byte.
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize> {
        // Interrupt-driven instances keep the bytes in their ring; polling ones
        // have nothing there and read the FIFO directly.
        let mut read = RX_RINGS[self.instance as usize].pop_into(buffer);
        while read < buffer.len() && self.fr() & registers::UART_FR_RXFE == 0 {
            buffer[read] = (self.read_reg(reg!(dr)) & 0xFF) as u8;
            read += 1;
        }
        Ok(read)
    }

    /// Block until `buffer` is full. There is no idle detection yet — that needs
    /// the RX-timeout interrupt — so a short frame waits for the rest.
    fn read_until_idle(&mut self, buffer: &mut [u8]) -> Result<usize> {
        let mut read = 0;
        for slot in buffer.iter_mut() {
            let mut spins = 0;
            while self.fr() & registers::UART_FR_RXFE != 0 {
                if spins >= POLL_LIMIT {
                    return Ok(read);
                }
                spins += 1;
                core::hint::spin_loop();
            }
            *slot = (self.read_reg(reg!(dr)) & 0xFF) as u8;
            read += 1;
        }
        Ok(read)
    }

    fn enable_rx(&mut self, enabled: bool) -> Result<()> {
        let cr = self.read_reg(reg!(cr));
        let cr = if enabled {
            cr | registers::UART_CR_UARTEN | registers::UART_CR_RXE
        } else {
            cr & !registers::UART_CR_RXE
        };
        self.write_reg(reg!(cr), cr);
        Ok(())
    }

    fn set_handler(&mut self, handler: Option<&'static dyn UartHandler>) {
        HANDLERS.set(self.instance as usize, handler);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 115200 on the 125 MHz peripheral clock: `4 * fclk / baud` = 4340, i.e.
    /// `IBRD = 67`, `FBRD = 52` (0.006 % error).
    #[test]
    fn divisors_of_115200_at_125mhz() {
        assert_eq!(divisors(125_000_000, 115_200), Some((67, 52)));
    }

    /// The RP2350's 150 MHz peripheral clock still lands within a tenth of a
    /// percent of the requested rate.
    #[test]
    fn divisors_round_to_the_requested_baud() {
        let (ibrd, fbrd) = divisors(150_000_000, 115_200).expect("reachable");
        let divisor = ibrd as u32 * 64 + fbrd as u32;
        let baud = 4 * 150_000_000 / divisor;
        assert!(baud.abs_diff(115_200) < 200, "baud {baud} is off");
    }

    /// A zero rate and an impossible slow rate are refused instead of producing a
    /// divisor of zero (which silences the port).
    #[test]
    fn impossible_rates_are_refused() {
        assert_eq!(divisors(125_000_000, 0), None);
        assert_eq!(divisors(125_000_000, 1), None);
    }
}
