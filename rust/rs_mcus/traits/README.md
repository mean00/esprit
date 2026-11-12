# rs_mcu_traits — the MCU-agnostic super traits

One file per peripheral, no chip knowledge, no dependencies. This crate is the
contract every esprit Rust driver implements; `rust_esprit` re-exports it under
the names the application (swindle) uses. Ownership of each layer: `layers.md`.

Design and migration plan: [`supertraits.md`](../../../../supertraits.md).

## The traits

| File | Trait(s) | Family identity is… | Implemented by |
|------|----------|---------------------|----------------|
| `gpio.rs` | `Gpio` | the family `Pin` type + `u32` pin number (the pad only) | `rs_gpio_*` |
| `exti.rs` | `Exti`, `ExtiHandler`, `Edge` | `u32` pin number (all pin interrupts) | `rs_exti_*` |
| `exti.rs` | `Exti`, `ExtiHandler` | `u32` pin number | `rs_exti_*` |
| `uart.rs` | `Uart`, `UartHandler` | the driver instance | `rs_uart_*` |
| `spi.rs` | `Spi`, `SpiHandler` | the driver instance | `rs_spi_*` |
| `i2c.rs` | `I2c`, `I2cHandler` | the driver instance | `rs_i2c_*` |
| `adc.rs` | `Adc`, `ScanAdc`, `AdcHandler`, `AdcBuffer` | source/channel ids (`u32`) | `rs_adc_*` |
| `timer.rs` | `Timer`, `DelayTimer`, their handlers | the driver instance | `rs_timer_*` |
| `pwm.rs` | `Pwm`, `PwmHandler` | channel index (`u8`) | `rs_multi_pulse_*` |
| `rcu.rs` | `Rcu` | **associated** `Peripheral` type | `rs_rcu_*` |
| `dma.rs` | `Dma`, `DmaHandler` | **associated** `Channel` type | `rs_dma_*` |

Plus two infrastructure files: `error.rs` (`Error` / `Result`) and `handler.rs`
(one handler slot **per instance**, so UART0 and UART1 can have different
handlers).

## Rules

1. **No MCU in a contract.** Chip identity is an associated type or a plain
   number. `grep -iE 'stm32|gd32|ch32|rp20|rp23' src/` must stay empty.
2. **`Result`, not C.** No `bool`/`i32` status codes, no out-parameters.
3. **Typed handlers**, `Option<&'static dyn XHandler>`, per instance through
   `handler::HandlerTable`.
4. **No `unsafe` in a signature.** The family crate owns the `unsafe`.

## Adding a family

For each trait the family can implement, one `impl` block in that family's
crate; the ones with an interrupt also need one `HandlerTable` sized to the
family's instance count (one slot per pin for EXTI, per port for the rest):

```rust
use rs_mcu_traits::exti::{Edge, Exti, ExtiHandler};
use rs_mcu_traits::gpio::{Gpio, PinMode};
use rs_mcu_traits::handler::HandlerTable;

static HANDLERS: HandlerTable<dyn ExtiHandler, PIN_COUNT> = HandlerTable::new();

impl Gpio for MyPin {
    fn pin_number(&self) -> u32 { /* … */ }
    fn set_mode(&mut self, mode: PinMode) -> rs_mcu_traits::Result<()> { /* … */ }
    // … write / read / toggle / open_drain
}

impl Exti for MyExti {
    fn attach(&mut self, pin: u32, edge: Edge, handler: Option<&'static dyn ExtiHandler>) -> rs_mcu_traits::Result<()> {
        HANDLERS.set(pin as usize, handler);
        /* program the line, then enable it */
        Ok(())
    }
    // … enable / disable / detach
}
```

`Gpio` is the pad only — mode, level, direction. Everything about an interrupt
(`attach`, `enable`, `disable`, `detach`, the handler, the `Edge`) lives in
`exti`, because that is a separate controller on most families and a separate
block (`IO_BANK0`) on the others.

A capability the family does not have returns `Error::Unsupported` instead of
panicking — never a fake success.

## Verification

```bash
cargo test --target x86_64-unknown-linux-gnu
cargo clippy --target thumbv6m-none-eabi
cargo clippy --target thumbv7m-none-eabi
```
