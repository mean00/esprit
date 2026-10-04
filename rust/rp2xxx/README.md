# rp2xxx — RP2040 / RP2350 backends

These crates are the link-time backends for `rust_esprit` on RP2040 / RP2350.

Most of them are **not drivers**: they only give `rust_esprit` something to link
against for the peripherals that esprit implements in C++ *only* for the other
targets, and every function body is `unimplemented!(...)`, i.e. it panics.  The
exception is the simple ADC (`ln_simple_adc_*`), which forwards to the C++
RP2xxx driver — see [ADC backends](#adc-backends-rs_adc_rp2xxx) below.

## Why they exist

`esprit/rust/CMakeLists.txt` builds the C++ bridges conditionally:

```cmake
if(NOT (USE_RP2040 OR USE_RP2350))
  cond_build(LN_ENABLE_TIMING_ADC lnTiming_adc_c.cpp)
  cond_build(LN_ENABLE_I2C        lnI2C_c.cpp)
  target_sources(esprit_c_bindings PRIVATE ${C}/lnTimer_c.cpp ${C}/lnMultiPulse_c.cpp)
else()
  target_sources(esprit_c_bindings PRIVATE ${C}/ln_rp_simple_adc_c.cpp)
endif()
```

So on RP2xxx, none of `ln_timer_*`, `ln_multi_pulse_*`, `ln_timing_adc_*`,
`lni2c_*` exists — neither as C++ nor as a Rust backend (the `rs_*_bluepill`
crates are only enabled by the `bluepill` cargo feature).

`rust_esprit` still has to *compile and link* for RP2xxx, because the same
crate is used for every target.  Each crate here therefore exposes exactly the
same public Rust entry points as its Bluepill sibling — same names, same
signatures, same opaque handle types — but every function body is
`unimplemented!(...)`, i.e. it panics (the simple ADC is the exception, see
[ADC backends](#adc-backends-rs_adc_rp2xxx)).

The result: using such a peripheral on RP2xxx now gives a clear, immediate
panic naming the missing feature, instead of an obscure
`undefined reference to '_Z19ln_timer_create…'` at link time (which is what
happens when the symbol is simply absent).

## ADC backends (`rs_adc_rp2xxx`)

Unlike the other crates, `rs_adc_rp2xxx` covers two different situations:

| C API | RP2xxx behaviour |
|-------|------------------|
| `ln_simple_adc_*` | **works** — delegates to the C++ `lnSimpleADC` driver |
| `ln_timing_adc_*` | panics — esprit has no RP2xxx `lnTimingAdc` |

esprit's RP2xxx ADC support is the C++
[`lnSimpleADC`](../rust_esprit/../mcus/arm_rp2040/include/lnADC.h) class
(`mcus/arm_rpX/src/ln_rp_adc.cpp`) — the same driver swindle uses in
`swindle/src/platform/rp2040/bmp_adc_rp2040.cpp` for target-voltage measurement.
It has no `extern "C"` interface, and `lnTimingAdc` / `lnBaseAdc` only exist for
the Bluepill targets (`mcus/common_bluepill/`), so:

* `rust_esprit/c_interface/ln_rp_simple_adc_c.cpp` is a small `extern "C"` bridge
  (built for RP2xxx only) around `lnSimpleADC`, and `rs_adc_rp2xxx` calls it from
  `ln_simple_adc_create/read/destroy`.  The shims are named `ln_rp_simple_adc_*`
  rather than `ln_simple_adc_*` because the latter C ABI belongs to the Rust
  backends (`rs_adc_bluepill`, `rs_adc_rp2xxx`).
* `ln_timing_adc_*` stays unsupported: porting it would mean writing a new
  timer-triggered DMA ADC driver for RP2xxx, not just calling existing code.

`ln_simple_adc_set_smpt()` is a no-op, matching both `lnSimpleADC` (no sample-time
knob on RP2xxx) and the Bluepill Rust backend.


## Naming

Crate names mirror the Bluepill siblings (`rs_adc_bluepill` → `rs_adc_rp2xxx`)
so the pairing is obvious.  `rust_esprit/src/lib.rs` selects between them, per
platform, in the `pub(crate) use … as rn_*_c;` block:

| `rust_esprit` name | GD32 / CH32 (`bluepill`) | ESP32 | RP2040 / RP2350 (`rp2040`) |
|--------------------|--------------------------|-------|----------------------------|
| `rn_i2c_c`         | `rs_i2c_bluepill`        | `c_api` | **`rs_i2c_rp2xxx`**      |
| `rn_spi_c`         | `rs_spi_bluepill`        | `c_api` | **`rs_spi_rp2xxx`**      |
| `rn_timing_adc_c`  | `rs_adc_bluepill`        | `c_api` | **`rs_adc_rp2xxx`**      |
| `rn_simple_adc_c`  | `rs_adc_bluepill`        | `c_api` | **`rs_adc_rp2xxx`** (→ C++ `lnSimpleADC`) |
| `rn_hw_stopwatch_c`| `rs_timer_bluepill`      | `c_api` | **`rs_timer_rp2xxx`**    |
| `rn_multi_pulse_c` | `c_api` (C++ built)      | `c_api` | **`rs_multi_pulse_rp2xxx`** |
| `rn_timer_c`       | `c_api` (C++ built)      | `c_api` | `c_api` (kept — `task.rs` uses the real `lnGetUs`/`lnDelay_C` helpers) |

The crates are enabled by the `rp2040` cargo feature of `rust_esprit`
(`rp2350 = ["rp2040"]`, so RP2350 builds select it too).

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
