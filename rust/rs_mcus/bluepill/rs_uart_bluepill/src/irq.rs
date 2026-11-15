//! Vector entries the platform's interrupt table still demands.
//!
//! The bluepill vector tables reference `USARTn_IRQHandler` unconditionally and are
//! not weak: `esprit/mcus/arm_gd32fx/include/lnIRQ_arm_vector.h` on GD32,
//! `esprit/mcus/riscv_ch32v3x/include/ch32v3x_interrupt_table.h` on CH32. The C++
//! file that used to define them (`esprit/mcus/common_bluepill/lnSerialCore.cpp`) is
//! not compiled in a Rust build (`LN_ENABLE_RUST` + `USE_RUST_LOGGER`), so the driver
//! owns the symbols - see `drop_C.md`, phase P1.
//!
//! There is nothing to service: this driver never sets a USART interrupt-enable bit -
//! TX and RX are DMA and polling (`buffered_tx.rs`, the ring in `lib.rs`) - so an
//! entry here would be spurious.
//!
//! Calling convention: a normal `ret`, on both families.
//!   * Cortex-M: the vector points straight at the handler and the core restores the
//!     stacked context from `EXC_RETURN` on `bx lr`.
//!   * CH32V3x: `ch32v3x_interrupt_table.cpp` points the table at
//!     `USARTn_IRQHandler_relay` when `USE_CH32v3x_HW_IRQ_STACK` is set (it is, by
//!     default: `esprit/CMakeLists.txt:57`), and that relay is a plain
//!     `jal USARTn_IRQHandler; mret`. This is the same split the DMA vectors use:
//!     `lnIRQ_riscv_ch32v3x.cpp` generates the handler that calls
//!     `dmaIrqHandler()`, which is this crate's neighbour in `rs_dma_bluepill`.

macro_rules! usart_irq {
    ($name:ident) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn $name() {}
    };
}

usart_irq!(USART0_IRQHandler);
usart_irq!(USART1_IRQHandler);
usart_irq!(USART2_IRQHandler);
