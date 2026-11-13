#![allow(unused_variables, unused_imports, unused_parens)]
#![allow(non_camel_case_types, non_snake_case, dead_code)]
#![no_std]

pub mod registers;
pub mod shim;
pub use shim::*;

/// Ring + DMA transmit buffer (Rust port of the C++
/// `lnSerialBpTxOnlyBufferedDma`), used by the Rust logger.
mod buffered_tx;
pub use buffered_tx::{BufferedTx, RING_SIZE as TX_RING_SIZE};

use core::ffi::c_void;
use core::ptr::{read_volatile, write_volatile};
use registers::*;
use rs_dma_bluepill::{DmaChannel, DmaEngine};
use rs_mcu_traits::handler::HandlerTable;
use rs_mcu_traits::uart;
use rs_mcu_traits::{Error, Result};
use rs_rcu_bluepill::{Peripheral, enable};
use rs_rtos::BinarySemaphore;
// The RX poll timer and the critical sections come from `rs_rtos` directly:
// `rs_timer_bluepill` is the *hardware* timer API, a
// different thing from a FreeRTOS software timer (`PeriodicTimer`).
use rs_rtos::{PeriodicTimer, with_critical};

pub(crate) fn uart_set_baudrate(instance: u32, speed: u32) -> bool {
    let regs = UartRegisters::ptr(instance);
    let periph = match instance {
        UART_INSTANCE_0 => Peripheral::Uart0,
        UART_INSTANCE_1 => Peripheral::Uart1,
        UART_INSTANCE_2 => Peripheral::Uart2,
        _ => Peripheral::Uart0,
    };
    let pclk = rs_rcu_bluepill::get_clock(periph);
    let usartdiv = (pclk + (speed / 2)) / speed;
    unsafe {
        write_volatile(&mut (*regs).baud, usartdiv);
    }
    true
}

/// Idiomatic struct representing a UART transmitter.
pub struct UartTx {
    pub(crate) instance: u32,
    pub(crate) use_dma: bool,
    pub(crate) dma: Option<DmaChannel>,
    sem: BinarySemaphore,
}

impl UartTx {
    pub fn new(instance: u32, dma: bool) -> Self {
        if instance == UART_INSTANCE_0 {
            enable(Peripheral::Uart0);
        } else if instance == UART_INSTANCE_1 {
            enable(Peripheral::Uart1);
        } else if instance == UART_INSTANCE_2 {
            enable(Peripheral::Uart2);
        }
        let dma_chan = if dma {
            let ch_idx = match instance {
                UART_INSTANCE_0 => UART0_DMA_TX_CHANNEL,
                UART_INSTANCE_1 => UART1_DMA_TX_CHANNEL,
                UART_INSTANCE_2 => UART2_DMA_TX_CHANNEL,
                _ => UART0_DMA_TX_CHANNEL,
            };
            Some(DmaChannel::new(DmaEngine::Dma0, ch_idx))
        } else {
            None
        };
        Self {
            instance,
            use_dma: dma,
            dma: dma_chan,
            sem: BinarySemaphore::new(),
        }
    }

    #[inline(always)]
    pub fn instance(&self) -> u32 {
        self.instance
    }

    pub fn init(&mut self) -> bool {
        let (tx_pin, _) = match self.instance {
            UART_INSTANCE_0 => (rs_gpio_bluepill::Pin::PA9, rs_gpio_bluepill::Pin::PA10),
            UART_INSTANCE_1 => (rs_gpio_bluepill::Pin::PA2, rs_gpio_bluepill::Pin::PA3),
            UART_INSTANCE_2 => (rs_gpio_bluepill::Pin::PB10, rs_gpio_bluepill::Pin::PB11),
            _ => (rs_gpio_bluepill::Pin::PA9, rs_gpio_bluepill::Pin::PA10),
        };
        rs_gpio_bluepill::set_mode(tx_pin, rs_gpio_bluepill::Mode::AlternatePushPull, 50);

        let regs = UartRegisters::ptr(self.instance);
        unsafe {
            let mut ctl0 = read_volatile(&mut (*regs).ctl0);
            ctl0 |= USART_CTL0_UEN | USART_CTL0_TEN;
            write_volatile(&mut (*regs).ctl0, ctl0);
        }
        true
    }

    pub fn set_speed(&mut self, speed: u32) -> bool {
        uart_set_baudrate(self.instance, speed)
    }

    pub fn transmit(&mut self, buffer: &[u8]) -> bool {
        if buffer.is_empty() {
            return true;
        }
        let regs = UartRegisters::ptr(self.instance);
        let size = buffer.len() as u32;

        unsafe {
            if self.use_dma {
                if let Some(ref mut dma) = self.dma {
                    self.sem.try_take();

                    let mut ctl2 = read_volatile(&mut (*regs).ctl2);
                    ctl2 |= USART_CTL2_DMAT;
                    write_volatile(&mut (*regs).ctl2, ctl2);

                    dma.attach_completion_semaphore(&self.sem);

                    dma.begin_tx_transfer(
                        &(*regs).data as *const _ as u32,
                        buffer.as_ptr() as u32,
                        size,
                        false,
                        false,
                    );

                    self.sem.take();

                    dma.end_transfer();
                    dma.detach_callback();

                    while (read_volatile(&mut (*regs).stat) & USART_STAT_TC) == 0 {
                        core::hint::spin_loop();
                    }
                }
            } else {
                for &byte in buffer {
                    while (read_volatile(&mut (*regs).stat) & USART_STAT_TBE) == 0 {}
                    write_volatile(&mut (*regs).data, byte as u32);
                }
                while (read_volatile(&mut (*regs).stat) & USART_STAT_TC) == 0 {}
            }
        }
        true
    }
}

unsafe extern "C" fn uart_rx_dma_callback(_is_half: bool, cookie: *mut c_void) {
    let instance = cookie as usize;
    if instance < UART_MAX_INSTANCES {
        unsafe {
            if let Some(ref mut rx) = RX_INSTANCES[instance] {
                rx.check_for_new_data();
            }
        }
    }
}

/// The RX poll tick, called by the FreeRTOS timer daemon task every
/// `RX_POLL_PERIOD_MS` (see `rs_rtos::timer::PeriodicTimer`).
///
/// The cookie is the UART instance number, so the callback needs no access to
/// the `UartRx` it belongs to.
fn uart_rx_timer_cb(instance: &u32) {
    let instance = *instance as usize;
    if instance < UART_MAX_INSTANCES {
        unsafe {
            if let Some(ref mut rx) = RX_INSTANCES[instance] {
                rx.check_for_new_data();
            }
        }
    }
}

/// Idiomatic struct representing a UART receiver with ring buffer and circular DMA.
pub struct UartRx {
    pub(crate) instance: u32,
    pub(crate) use_dma: bool,
    /// Ring buffer, held inline so that it lives in `.bss` with the rest of the
    /// receiver (`RX_INSTANCES[]` is a static): no heap allocation at init
    /// (malloc_used.md §0.7, C11).
    pub(crate) rx_buffer: [u8; RX_BUFFER_SIZE_MAX],
    pub(crate) rx_buffer_size: usize,
    pub(crate) rx_mask: usize,
    pub(crate) rx_head: usize,
    pub(crate) rx_tail: usize,
    pub(crate) dma: Option<DmaChannel>,
    /// Poll timer: a FreeRTOS software timer (`rs_rtos::timer::PeriodicTimer`)
    /// whose control block lives inside this struct.
    pub(crate) timer: PeriodicTimer<u32>,
    pub(crate) callback: ln_serial_event_cb,
    pub(crate) callback_cookie: *mut c_void,
    pub(crate) rx_enabled: bool,
}

impl UartRx {
    pub fn new(instance: u32, buffer_size: u32, dma: bool) -> Self {
        if instance == UART_INSTANCE_0 {
            enable(Peripheral::Uart0);
        } else if instance == UART_INSTANCE_1 {
            enable(Peripheral::Uart1);
        } else if instance == UART_INSTANCE_2 {
            enable(Peripheral::Uart2);
        }
        let size = if buffer_size == 0 {
            RX_BUFFER_SIZE_DEFAULT
        } else {
            buffer_size.next_power_of_two() as usize
        };
        // The ring is a fixed `.bss` array (malloc_used.md §0.7, C11); a caller
        // that needs a bigger one has to raise RX_BUFFER_SIZE_MAX.
        assert!(
            size <= RX_BUFFER_SIZE_MAX,
            "uart_rx: RX buffer size above RX_BUFFER_SIZE_MAX"
        );
        let rx_mask = size - 1;

        let dma_chan = if dma {
            let ch_idx = match instance {
                UART_INSTANCE_0 => UART0_DMA_RX_CHANNEL,
                UART_INSTANCE_1 => UART1_DMA_RX_CHANNEL,
                UART_INSTANCE_2 => UART2_DMA_RX_CHANNEL,
                _ => UART0_DMA_RX_CHANNEL,
            };
            Some(DmaChannel::new(DmaEngine::Dma0, ch_idx))
        } else {
            None
        };

        // The RX poll timer.  `PeriodicTimer` keeps the kernel control block in
        // an inline `.bss` slot that is filled on first use, so the timer
        // survives the move into `RX_INSTANCES[]` and the driver needs no timer
        // storage of its own (malloc_used.md §8.1 S3b).
        assert!(
            (instance as usize) < UART_MAX_INSTANCES,
            "uart_rx: instance out of range"
        );
        let timer = PeriodicTimer::new(c"uart rx", RX_POLL_PERIOD_MS, instance, uart_rx_timer_cb);

        Self {
            instance,
            use_dma: dma,
            rx_buffer: [0u8; RX_BUFFER_SIZE_MAX],
            rx_buffer_size: size,
            rx_mask,
            rx_head: 0,
            rx_tail: 0,
            dma: dma_chan,
            timer,
            callback: None,
            callback_cookie: core::ptr::null_mut(),
            rx_enabled: false,
        }
    }

    #[inline(always)]
    pub fn instance(&self) -> u32 {
        self.instance
    }

    pub fn init(&mut self) -> bool {
        let (_, rx_pin) = match self.instance {
            UART_INSTANCE_0 => (rs_gpio_bluepill::Pin::PA9, rs_gpio_bluepill::Pin::PA10),
            UART_INSTANCE_1 => (rs_gpio_bluepill::Pin::PA2, rs_gpio_bluepill::Pin::PA3),
            UART_INSTANCE_2 => (rs_gpio_bluepill::Pin::PB10, rs_gpio_bluepill::Pin::PB11),
            _ => (rs_gpio_bluepill::Pin::PA9, rs_gpio_bluepill::Pin::PA10),
        };
        rs_gpio_bluepill::set_mode(rx_pin, rs_gpio_bluepill::Mode::InputFloating, 0);

        let regs = UartRegisters::ptr(self.instance);
        unsafe {
            let mut ctl0 = read_volatile(&mut (*regs).ctl0);
            ctl0 |= USART_CTL0_UEN | USART_CTL0_REN;
            write_volatile(&mut (*regs).ctl0, ctl0);
        }
        true
    }

    pub fn set_speed(&mut self, speed: u32) -> bool {
        uart_set_baudrate(self.instance, speed)
    }

    pub fn enable_rx(&mut self, enabled: bool) -> bool {
        let regs = UartRegisters::ptr(self.instance);

        // State and hardware change under a critical section (`rs_rtos::critical`):
        // that is what `get_read_pointer()`/`consume()` and the poll timer race
        // with.  The timer commands stay outside -- `start()`/`stop()` may wait
        // for the timer daemon, which cannot run while the scheduler is masked
        // (the old code called `xTimerStop()`/`xTimerStart()` inside the section
        // and could therefore block with interrupts off).
        with_critical(|| {
            self.rx_enabled = enabled;
            if enabled {
                self.rx_head = 0;
                self.rx_tail = 0;

                unsafe {
                    while (read_volatile(&mut (*regs).stat) & USART_STAT_RBNE) != 0 {
                        let _ = read_volatile(&mut (*regs).data);
                    }

                    let mut ctl0 = read_volatile(&mut (*regs).ctl0);
                    ctl0 |= USART_CTL0_UEN | USART_CTL0_REN;
                    write_volatile(&mut (*regs).ctl0, ctl0);

                    if self.use_dma {
                        let mut ctl2 = read_volatile(&mut (*regs).ctl2);
                        ctl2 |= USART_CTL2_DMAR;
                        write_volatile(&mut (*regs).ctl2, ctl2);

                        if let Some(ref mut dma) = self.dma {
                            dma.attach_callback(
                                uart_rx_dma_callback,
                                self.instance as usize as *mut c_void,
                            );
                            dma.begin_circular_rx_transfer(
                                &(*regs).data as *const _ as u32,
                                self.rx_buffer.as_ptr() as u32,
                                self.rx_buffer_size as u32,
                                false,
                                false,
                            );
                        }
                    }
                }
            } else {
                unsafe {
                    if self.use_dma {
                        if let Some(ref mut dma) = self.dma {
                            dma.end_transfer();
                        }
                        let mut ctl2 = read_volatile(&mut (*regs).ctl2);
                        ctl2 &= !USART_CTL2_DMAR;
                        write_volatile(&mut (*regs).ctl2, ctl2);
                    }
                    let mut ctl0 = read_volatile(&mut (*regs).ctl0);
                    ctl0 &= !USART_CTL0_REN;
                    write_volatile(&mut (*regs).ctl0, ctl0);
                }
            }
        });

        // Only a DMA-driven receiver needs the poll timer: without DMA
        // `current_dma_pos()` reports `rx_head`, which never moves.
        if self.use_dma {
            if enabled {
                self.timer.start();
            } else {
                self.timer.stop();
            }
        }
        true
    }

    #[inline]
    pub fn current_dma_pos(&self) -> usize {
        if self.use_dma {
            if let Some(ref dma) = self.dma {
                let rem = dma.remaining_transfers() as usize;
                if rem <= self.rx_buffer_size {
                    self.rx_buffer_size - rem
                } else {
                    0
                }
            } else {
                0
            }
        } else {
            self.rx_head & self.rx_mask
        }
    }

    pub fn check_for_new_data(&mut self) {
        if !self.rx_enabled {
            return;
        }
        let count = self.current_dma_pos();
        let masked_head = self.rx_head & self.rx_mask;
        if count == masked_head {
            return;
        }
        let t = self.rx_head & self.rx_mask;
        self.rx_head = (self.rx_head & !self.rx_mask) + count;
        if self.rx_head.saturating_sub(self.rx_tail) > self.rx_buffer_size {
            self.rx_tail = self.rx_head; // overflow recovery
        }
        if t >= count {
            self.rx_head += self.rx_buffer_size;
            if self.rx_head.saturating_sub(self.rx_tail) > self.rx_buffer_size {
                self.rx_tail = self.rx_head;
            }
        }
        if let Some(cb) = self.callback {
            unsafe {
                cb(self.callback_cookie, 0); // 0 = SerialEvent::DataAvailable
            }
        }
    }

    pub fn get_read_pointer(&mut self) -> (i32, *mut u8) {
        with_critical(|| {
            let h = self.rx_head & self.rx_mask;
            let t = self.rx_tail & self.rx_mask;

            let nb = if h >= t {
                if self.rx_head.saturating_sub(self.rx_tail) > self.rx_buffer_size {
                    self.rx_tail = self.rx_head;
                    0
                } else {
                    self.rx_head - self.rx_tail
                }
            } else {
                self.rx_buffer_size - t
            };

            let ptr = if nb > 0 {
                unsafe { self.rx_buffer.as_mut_ptr().add(t) }
            } else {
                core::ptr::null_mut()
            };

            (nb as i32, ptr)
        })
    }

    pub fn consume(&mut self, n: u32) {
        with_critical(|| {
            self.rx_tail += n as usize;
            if self.rx_tail > self.rx_buffer_size && self.rx_head > self.rx_buffer_size {
                self.rx_tail -= self.rx_buffer_size;
                self.rx_head -= self.rx_buffer_size;
            }
        });
    }

    pub fn purge_rx(&mut self) {
        with_critical(|| {
            self.rx_head = 0;
            self.rx_tail = 0;
        });
    }

    /// Install the "new data" notification of this receiver.
    ///
    /// It is called from the receive poll (a FreeRTOS software timer, i.e.
    /// interrupt/timer-daemon context) with the receiver's event code
    /// (`0` = data available, mirroring `lnSerialCore::Event`); `cookie` is
    /// passed straight through.
    pub fn set_event_callback(&mut self, cb: ln_serial_event_cb, cookie: *mut c_void) {
        self.callback = cb;
        self.callback_cookie = cookie;
    }

    pub fn read(&mut self, to: &mut [u8]) -> usize {
        let (nb, ptr) = self.get_read_pointer();
        if nb <= 0 || ptr.is_null() {
            return 0;
        }
        let count = core::cmp::min(nb as usize, to.len());
        unsafe {
            core::ptr::copy_nonoverlapping(ptr, to.as_mut_ptr(), count);
        }
        self.consume(count as u32);
        count
    }
}

impl Drop for UartRx {
    fn drop(&mut self) {
        // Disable the receiver: this stops the DMA and posts `stop` to the poll
        // timer.  The timer's control block is not deleted (see `rs_rtos::timer`:
        // a delete command is processed by the daemon *after* the call returns,
        // so it would write into a structure that is already gone).  Here that
        // is harmless: `RX_INSTANCES[]` is a `.bss` array, so the memory the
        // kernel still knows about stays valid, and a late expiry finds
        // `RX_INSTANCES[instance] == None` and does nothing.
        self.enable_rx(false);
    }
}

/// One handler slot per USART instance.
pub(crate) static HANDLERS: HandlerTable<dyn uart::UartHandler, UART_MAX_INSTANCES> = HandlerTable::new();

/// Buffered transmitters, one per instance: a [`BufferedTx`] must live in a
/// `static` because its DMA completion callback uses its address as cookie.
/// Only the instances opened with [`Uart::with_dma`] are initialised.
static BUFFERED_TX: [BufferedTx; UART_MAX_INSTANCES] = [
    BufferedTx::new(UART_INSTANCE_0),
    BufferedTx::new(UART_INSTANCE_1),
    BufferedTx::new(UART_INSTANCE_2),
];

/// Receiver notification, called from the RX poll (a FreeRTOS software timer)
/// when the DMA ring grew: it only wakes the handler, the bytes stay in the
/// ring for [`uart::Uart::read`] (`on_rx_available`'s contract).
///
/// The cookie is the instance number, exactly like `uart_rx_timer_cb`'s.
extern "C" fn rx_available_trampoline(cookie: *mut c_void, _event: i32) {
    let instance = cookie as usize;
    if instance >= UART_MAX_INSTANCES {
        return;
    }
    // SAFETY: `set_handler` published the handler before the receiver was
    // enabled, and the slot is a `&'static dyn UartHandler` the poll only copies.
    if let Some(handler) = unsafe { HANDLERS.isr_get(instance) } {
        handler.on_rx_available();
    }
}

/// The receive ring size of the super-trait view: the same default the C shim
/// uses, so a caller gets the same buffering either way.
const RX_BUFFER_SIZE: u32 = 256;

/// One USART as the super trait sees it: the TX and RX halves of the same
/// instance, behind one object.
///
/// The split [`UartTx`] / [`UartRx`] types stay public for the C ABI and for a
/// caller that needs only one direction; this type is what
/// [`rs_mcu_traits::uart::Uart`] is implemented for.
pub struct Uart {
    tx: UartTx,
    rx: UartRx,
    /// Buffered (DMA-ring) transmitter, when the port was opened with
    /// [`Uart::with_dma`]. `None` means the plain blocking [`UartTx`].
    buffered: Option<&'static BufferedTx>,
}

impl Uart {
    /// Create both halves of `instance` (`0` … `2`) and apply the frame.
    ///
    /// The GD32 USART of this driver is 8N1 at the configured rate: any other
    /// frame is refused by [`uart::Uart::configure`] instead of being silently
    /// coerced.
    pub fn new(instance: u32, config: &uart::UartConfig) -> Result<Self> {
        let mut tx = UartTx::new(instance, false);
        let mut rx = UartRx::new(instance, RX_BUFFER_SIZE, false);
        tx.init();
        rx.init();
        let mut uart = Self {
            tx,
            rx,
            buffered: None,
        };
        uart::Uart::configure(&mut uart, config)?;
        uart::Uart::enable_rx(&mut uart, true)?;
        Ok(uart)
    }

    /// The bridged configuration: a DMA-drained transmit **ring** (so
    /// [`uart::Uart::try_write`] can be partial) on top of a DMA receive ring of
    /// `RX_BUFFER_SIZE` bytes, whose growth is reported to the handler as
    /// [`uart::UartHandler::on_rx_available`].
    ///
    /// This is the Rust equivalent of the C++ `createLnSerialRxTx(instance,
    /// rx_buffer_size, /*dma*/ true)`: what a pass-through bridge needs. The
    /// transmitter is the crate's [`BufferedTx`] (the same one the Rust logger
    /// drives), so the bytes leave in the background instead of blocking the
    /// caller for every byte.
    pub fn with_dma(instance: u32, config: &uart::UartConfig) -> Result<Self> {
        let index = instance as usize;
        if index >= UART_MAX_INSTANCES {
            return Err(Error::InvalidConfig);
        }
        let buffered = &BUFFERED_TX[index];
        buffered.init(config.baudrate);

        // The transmitter half only drives the frame format and `flush`; the
        // bytes go through `buffered`.
        let mut tx = UartTx::new(instance, false);
        let mut rx = UartRx::new(instance, RX_BUFFER_SIZE, true);
        tx.init();
        rx.init();
        // Report "the ring grew" to the handler instead of keeping it private.
        rx.set_event_callback(Some(rx_available_trampoline), index as *mut c_void);

        let mut uart = Self {
            tx,
            rx,
            buffered: Some(buffered),
        };
        uart::Uart::configure(&mut uart, config)?;
        uart::Uart::enable_rx(&mut uart, true)?;
        Ok(uart)
    }

    /// This USART's number.
    #[inline(always)]
    pub fn instance(&self) -> u32 {
        self.tx.instance
    }

    /// The transmit half, for a caller that needs the DMA path.
    #[inline(always)]
    pub fn transmitter(&mut self) -> &mut UartTx {
        &mut self.tx
    }

    /// The receive half, for a caller that needs the ring buffer directly.
    #[inline(always)]
    pub fn receiver(&mut self) -> &mut UartRx {
        &mut self.rx
    }

    /// Tell the handler that the transmitter is idle again, i.e. everything
    /// queued has left (or the plain transmitter finished its transfer).
    fn notify_tx_idle(&self) {
        let idle = self.buffered.map(|b| b.pending() == 0).unwrap_or(true);
        if idle && let Some(handler) = HANDLERS.get(self.instance() as usize) {
            handler.on_tx_complete();
        }
    }
}

impl uart::Uart for Uart {
    fn configure(&mut self, config: &uart::UartConfig) -> Result<()> {
        if config.data_bits != 8
            || config.parity != uart::Parity::None
            || config.stop_bits != uart::StopBits::One
        {
            return Err(Error::Unsupported);
        }
        if !self.tx.set_speed(config.baudrate) {
            return Err(Error::InvalidConfig);
        }
        Ok(())
    }

    fn set_baudrate(&mut self, baudrate: u32) -> Result<()> {
        if self.tx.set_speed(baudrate) {
            Ok(())
        } else {
            Err(Error::InvalidConfig)
        }
    }

    /// Send into the transmit ring (`with_dma`) or block on the hardware
    /// transmitter.
    fn write(&mut self, data: &[u8]) -> Result<()> {
        if let Some(buffered) = self.buffered {
            buffered.write(data);
            self.notify_tx_idle();
            return Ok(());
        }
        if self.tx.transmit(data) {
            self.notify_tx_idle();
            Ok(())
        } else {
            Err(Error::Io)
        }
    }

    /// Queue what the ring accepts and return the count; a partial count means
    /// the caller has to come back (that is the bridge's `Blocked` state).
    fn try_write(&mut self, data: &[u8]) -> Result<usize> {
        let Some(buffered) = self.buffered else {
            // The non-buffered transmitter has no queue of its own: it either
            // sends everything or reports the failure.
            return Err(Error::Unsupported);
        };
        let accepted = buffered.try_write(data);
        self.notify_tx_idle();
        Ok(accepted)
    }

    /// Wait until the transmit ring has drained (`with_dma`) — the plain
    /// transmitter already waited inside `write`.
    fn flush(&mut self) -> Result<()> {
        if let Some(buffered) = self.buffered {
            buffered.flush();
        }
        Ok(())
    }

    /// Drain what the receive ring holds.
    ///
    /// No handler call here: the notification is the receive poll's
    /// [`uart::UartHandler::on_rx_available`], which fires when the ring grows.
    /// Telling the handler again after the caller took the bytes would make it
    /// look like more data arrived.
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize> {
        Ok(self.rx.read(buffer))
    }

    /// The ring is filled by the receiver's interrupt/DMA, so "until idle" is the
    /// same drain: there is no separate idle state to wait for.
    fn read_until_idle(&mut self, buffer: &mut [u8]) -> Result<usize> {
        self.read(buffer)
    }

    fn enable_rx(&mut self, enabled: bool) -> Result<()> {
        if self.rx.enable_rx(enabled) {
            Ok(())
        } else {
            Err(Error::Unsupported)
        }
    }

    fn set_handler(&mut self, handler: Option<&'static dyn uart::UartHandler>) {
        HANDLERS.set(self.instance() as usize, handler);
    }
}
