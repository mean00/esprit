# rp2xxx — RP2040 / RP2350 backends

These crates are the link-time backends for `rust_esprit` on RP2040 / RP2350.

Most of them are **not drivers**: they only give `rust_esprit` something to link
against for the peripherals that esprit implements in C++ *only* for the other
targets, and every function body is `unimplemented!(...)`, i.e. it panics.  The
exceptions are the ADC (`rs_adc_rp2xxx`), the GPIO driver `rs_gpio_rp2xxx`, the
external interrupts (`rs_exti_rp2xxx`), the UART (`rs_uart_rp2xxx`), the
DMA/SIO/RCU backends and the SPI driver — see
[ADC backend](#adc-backend-rs_adc_rp2xxx),
[EXTI backend](#external-interrupt-backend-rs_exti_rp2xxx) and
[GPIO backend](#gpio-backend-rs_gpio_rp2xxx) below.

## Why they exist

`esprit/rust/CMakeLists.txt` builds the C++ bridges conditionally:

```cmake
if(NOT (USE_RP2040 OR USE_RP2350))
  cond_build(LN_ENABLE_TIMING_ADC lnTiming_adc_c.cpp)
  cond_build(LN_ENABLE_I2C        lnI2C_c.cpp)
  target_sources(esprit_c_bindings PRIVATE ${C}/lnTimer_c.cpp ${C}/lnMultiPulse_c.cpp)
endif()
```

So on RP2xxx, none of `ln_timer_*`, `ln_multi_pulse_*`, `ln_timing_adc_*`,
`lni2c_*` exists — neither as C++ nor as a Rust backend (the `rs_*_bluepill`
crates are only enabled by the `bluepill` cargo feature). The ADC needs no bridge
at all: `rs_adc_rp2xxx` is a driver of its own (see
[ADC backend](#adc-backend-rs_adc_rp2xxx)), which is why the
`ln_rp_simple_adc_c.cpp` shim is gone.

`rust_esprit` still has to *compile and link* for RP2xxx, because the same
crate is used for every target.  Each crate here therefore exposes exactly the
same public Rust entry points as its Bluepill sibling — same names, same
signatures, same opaque handle types — but every function body is
`unimplemented!(...)`, i.e. it panics (the ADC is the exception, see
[ADC backend](#adc-backend-rs_adc_rp2xxx)).

The result: using such a peripheral on RP2xxx now gives a clear, immediate
panic naming the missing feature, instead of an obscure
`undefined reference to '_Z19ln_timer_create…'` at link time (which is what
happens when the symbol is simply absent).

## ADC backend (`rs_adc_rp2xxx`)

A **pure-Rust driver** on the ADC block: no C driver, no C ABI, no opaque handle.
It implements the esprit super traits of `rs_mcu_traits::adc`:

| Rust entry point | What it is |
|------------------|------------|
| `rs_adc_rp2xxx::SimpleAdc` | `adc::Adc` — one pin, one polled conversion (the probe's target-voltage sense) |
| `rs_adc_rp2xxx::TimingAdc` | `adc::ScanAdc` — N inputs × M samples per input, moved by DMA |

`TimingAdc::multi_read()` blocks on a `BinarySemaphore` given by the DMA
interrupt; `TimingAdc::start_async()` returns at once and runs the completion **in
the DMA interrupt**, either the `AdcHandler::on_scan_complete(&[u16])` handler
installed with `set_handler()` or the raw `callback(cookie)` pair. The whole
sequence is the datasheet's (RP2040 §4.9.2.5, RP2350 §12.4.4.4): program `DIV` and
`CS.AINSEL`/`CS.RROBIN`, enable the sample FIFO with its DREQ (`FCS.EN |
FCS.DREQ_EN`, `THRESH = 1`), start the DMA on `DREQ_ADC` with 16-bit samples, then
`CS.START_MANY`; on completion clear `START_MANY`, wait for `CS.READY` and drain the
FIFO.

Only the numbers are chip-specific, and they live in `registers_rp2040.rs` /
`registers_rp2350.rs`: the widths of `CS.AINSEL` / `CS.RROBIN` (3+5 bits and 5
analog inputs there, 4+9 bits and 9 there), `DREQ_ADC` (36 / 48) and the pin →
channel table (`GPIO26` … `GPIO29` on both chips, plus `GPIO40` … `GPIO47` on the
RP2350's QFN-80, where the hardware renumbers `AINSEL` for the package).

`set_smpt()` is a no-op, matching the fact that the RP2xxx SAR has no sample-time
knob (unlike the GD32 `SMPT` field), and `TimingAdc` ignores the `timer` /
`channel` arguments of the `TimingAdc` trait: this ADC free-runs from its own clock
divider instead of a hardware trigger. The usable rate range is
`clk_adc/256 … clk_adc` (187 kHz … 48 MHz at the default clock), and `fq = 0`
means back-to-back conversions.

`rust_esprit::AdcTiming` is a thin typed front-end over this driver (no handle, no
`unsafe` at the call site); the C bridge `ln_rp_simple_adc_c.cpp` and the C++
`lnSimpleADC` it wrapped are no longer part of a Rust build.


## External-interrupt backend (`rs_exti_rp2xxx`)

The `rs_mcu_traits::exti::Exti` super trait on IO_BANK0. On an RP2xxx a pin
interrupt *is* the GPIO interrupt: each pad owns one nibble of `INTR` /
`PROC0_INTE` (level-low, level-high, edge-low, edge-high), and every pad shares
one NVIC line (`IO_IRQ_BANK0`), so the crate installs a single ISR that walks
`PROC0_INTS` and calls the handler armed for the pad that fired.

`attach(pin, edge, handler)` publishes the handler, clears any pending edge and
enables the pad's edge bits (it also installs and enables the IRQ line the first
time); `disable` masks the pad but keeps the handler, `detach` forgets it. The two
chips differ (from the datasheets, not from a guess):

| | RP2040 | RP2350 |
|---|---|---|
| `INTR0` offset in IO_BANK0 | `0x0f0` | `0x230` |
| interrupt words / pads | 4 / 30 | 6 / 48 |
| `IO_IRQ_BANK0` NVIC line | 13 | 21 |

The pad has to be an input first — that is the `Gpio` contract
(`set_mode(.., PinMode::InputPullUp)`), not this one. Swindle's W5500 INT line is
the first user: it arms a falling edge, masks the pad in the handler and lets the
coordinator task re-arm it, which is how the level-held INT pin becomes edge
driven.

## UART backend (`rs_uart_rp2xxx`)

The `rs_mcu_traits::uart::Uart` super trait on the two PL011s. `Uart::new(instance,
tx, rx)` is the polling driver (frame/baud from `clk_peri`); `Uart::with_dma(instance,
&config)` is the bridged one and is what swindle's USB↔UART bridge opens:

| | polling (`new`) | bridged (`with_dma`) |
|---|---|---|
| pads | caller's | `UART0` = `GPIO0`/`GPIO1`, `UART1` = `GPIO4`/`GPIO5` |
| receive | `read` drains the FIFO | `RXIM`/`RTIM`/`OEIM` on the instance's NVIC line (20/21 on the RP2040, 33/34 on the RP2350); the ISR moves the FIFO into a per-instance `ring::Ring` (256 B, SPSC, lock-free) and calls `UartHandler::on_rx_available` |
| `read` | FIFO | ring first, then the FIFO |
| `try_write` | TX FIFO free depth | same |

`ring.rs` is the SPSC ring the ISR produces into and `read` consumes from; its four
host tests cover wrap-around of the 16-bit indices, the full/empty edge and the
short-consumer case.

## GPIO backend (`rs_gpio_rp2xxx`)

The second real driver, and the only crate here that is a full peripheral driver:
it implements the `rs_mcu_traits::gpio::Gpio` super trait for `rs_gpio_rp2xxx::Pin`
(`GPIO0` … `GPIO29`) straight on the SIO / IO_BANK0 / PADS_BANK0 registers
(`registers.rs`, numbers from `rp2040.md`; the RP2350 is treated as identical).
Pin type, mode mapping, open-drain sequencing and registers all live in that crate,
exactly like `rs_gpio_bluepill` does for the bluepill parts.

That `Pin` is *the* pin type of the RP2xxx Rust stack: `rust_esprit` re-exports it
as its `Pin`, and `rust_esprit`'s C bindings use it as their `lnPin`
(`c_api/rn_gpio_rp2040_c.rs` re-exports it, and `pin_types.rs` hands the same type
to the timer/ADC/exti bindings), so a board-table entry and a `lnPin` argument are
the same type.

It still exports **no** C ABI: the `lnDigitalWrite` / `lnPinMode` /
`lnOpenDrainClose` symbols are owned by esprit's C++ GPIO
(`mcus/rp_common/src/ln_rp_gpio.cpp`) on RP2xxx. The mode mapping and the
open-drain sequence mirror that file bit for bit, so the two can be mixed in one
firmware (C++ SWD pins + Rust target-reset line, which is what
`rs_swindle_board`'s `rp2xxx::reset_pin` does).

It is reached as `rust_esprit::gpio_rp2xxx` (enabled by the `rp2040` feature). The
first consumer is the probe board table: `swindle/rs/rs_swindle_board/src/rp2xxx/`.

## RCU backend (`rs_rcu_rp2xxx`)

Pure-Rust Reset and Clock Unit driver for RP2040 and RP2350:
implements the `rs_mcu_traits::rcu::Rcu` super trait on the RP2xxx RESETS and CLOCKS
blocks (`registers.rs`). It is the RP2xxx counterpart of `rs_rcu_bluepill`,
providing `enable()`, `disable()`, `reset()`, and `get_clock()`.

It is used by peripheral drivers such as `rs_spi_rp2xxx` to unreset peripherals
and query bus/peripheral frequencies via `Rcu::enable()` and `Rcu::get_clock()`.

## Naming

Crate names mirror the Bluepill siblings (`rs_adc_bluepill` → `rs_adc_rp2xxx`)
so the pairing is obvious.  `rust_esprit/src/lib.rs` selects between them, per
platform, in the `pub(crate) use … as rn_*_c;` block:

| `rust_esprit` name | GD32 / CH32 (`bluepill`) | ESP32 | RP2040 / RP2350 (`rp2040`) |
|--------------------|--------------------------|-------|----------------------------|
| `rn_i2c_c`         | `rs_i2c_bluepill`        | `c_api` | **`rs_i2c_rp2xxx`** (`I2c` = `Error::Unsupported` until the DW_apb driver lands) |
| `rn_spi_c`         | `rs_spi_bluepill`        | `c_api` | **`rs_spi_rp2xxx`**      |
| `rn_serial_c`      | `rs_uart_bluepill`       | `c_api` | `c_api` (C++ `rp_serial`) — the Rust `Uart` impl is `rs_uart_rp2xxx` |
| `rn_timing_adc_c`  | `rs_adc_bluepill`        | `c_api` | — (pure-Rust driver, used directly) |
| `rn_simple_adc_c`  | `rs_adc_bluepill`        | `c_api` | — (pure-Rust driver, used directly) |
| `rn_hw_stopwatch_c`| `rs_timer_bluepill`      | `c_api` | **`rs_timer_rp2xxx`**    |
| `rn_multi_pulse_c` | `c_api` (C++ built)      | `c_api` | **`rs_multi_pulse_rp2xxx`** |
| `rn_timer_c`       | `c_api` (C++ built)      | `c_api` | `c_api` (kept — `task.rs` uses the real `lnGetUs`/`lnDelay_C` helpers) |

The crates are enabled by the `rp2040` cargo feature of `rust_esprit`
(`rp2350 = ["rp2040", …]`, so RP2350 builds select it too).

## Chip selection (`rp2040` / `rp2350`)

The two chips do not share an address map: IO_BANK0 / PADS_BANK0, SPI, RESETS and
CLOCKS sit at different addresses on the RP2350. Every driver therefore takes its
bases from `rs_rp2xxx`'s **root** re-export, and which one that re-export holds is
a cargo feature:

| crate | feature | effect |
|-------|---------|--------|
| `rs_rp2xxx` | `rp2040` / `rp2350` | `lib.rs` re-exports `rp2040::*` or `rp2350::*` at the root |
| `rs_gpio_rp2xxx` | `rp2040` / `rp2350` | `IO_BANK0_BASE`, `PADS_BANK0_BASE` |
| `rs_spi_rp2xxx` | `rp2040` / `rp2350` | `SPI0_BASE`, `SPI1_BASE` |
| `rs_dma_rp2xxx` | `rp2040` / `rp2350` | `RESETS_BASE` (`DMA_BASE` is shared) |
| `rs_rcu_rp2xxx` | `rp2040` / `rp2350` | compile-time `is_rp2040()`; with neither set it probes `SYSINFO.CHIP_ID` |

The chain is CMake → cargo: `USE_RP2040` / `USE_RP2350`
(`swindle/rs/CMakeLists.txt`) sets `SWINDLE_RUST_PLATFORM_FEATURE` to `rp2040` or
`rp2350`, `rs_swindle_wrapper` / `rs_swindle` forward it, and `rust_esprit`'s
`rp2040` feature enables `rs_*_rp2xxx/rp2040` while its `rp2350` feature enables
`rs_*_rp2xxx/rp2350` **as well** (`rp2350` implies `rp2040` because that is the
"this is an RP2xxx" selector every `#[cfg]` in `rust_esprit` tests). The backends
therefore see both flags on an RP2350 build and resolve them as **`rp2350` wins**;
a build with neither is a `compile_error!` in `rs_rp2xxx`.

### Where a differing constant goes

A value that differs between the chips lives in a **chip file**, everything else in
the shared one; the driver bodies carry no chip `#[cfg]` at all. The selection
happens once per crate, in its `lib.rs`:

| crate | shared | chip files |
|-------|--------|------------|
| `rs_rp2xxx` | `common.rs` | `rp2040.rs` / `rp2350.rs` (base addresses) |
| `rs_gpio_rp2xxx` | `registers.rs` | — (bases read from the family root) |
| `rs_spi_rp2xxx` | `registers.rs` | — (PL022 layout is identical, bases from the root) |
| `rs_dma_rp2xxx` | `registers.rs` | `registers_rp2040.rs` / `registers_rp2350.rs`: channel count (12 / 16), the global block, the `CTRL_TRIG` field layout, the DREQ numbers |
| `rs_rcu_rp2xxx` | `registers.rs` | `registers_rp2040.rs` / `registers_rp2350.rs`: reset-bit table, USB/ADC clock indices, `get_reset_bit()` |
| `rs_adc_rp2xxx` | `registers.rs` | `registers_rp2040.rs` / `registers_rp2350.rs`: `CS.AINSEL`/`CS.RROBIN` widths, `ADC_INPUT_COUNT`, the pin → channel table, `DREQ_ADC` (36 / 48) |

The DMA is the example that matters: the RP2350 has 16 channels, four interrupt
groups, `TREQ_SEL` at 22:17 instead of 20:15 and DREQ numbers 8 higher (PIO2 sits
in between), so none of that can be shared.

## Note on the pin parameters

A few entry points take a pin (`ln_timer_create_from_pin`, `ln_multi_pulse_create`,
`ln_timing_adc_set_source`).  The RP2xxx `lnPin` enum is bindgen-generated
*inside* `rust_esprit`, so a sibling crate cannot name it without a circular
dependency; those parameters are therefore generic (`<P>` by value, `<T>` for
the `*const T` of `set_source`).  The bodies never touch the value, so this is
behaviourally irrelevant.

Because of that genericity those three entry points are plain `pub fn` and are
**not** part of the C alias system — a generic function cannot carry
`#[unsafe(no_mangle)]`.  Every other entry point keeps the usual
`#[unsafe(no_mangle)] pub extern "C" fn` alias (codingrules.md §6), exactly like
its Bluepill sibling, so legacy C/C++ callers get the same panicking stub rather
than an undefined symbol.  That is safe here because the C++ bridges that would
otherwise define those symbols are not built on RP2xxx.
