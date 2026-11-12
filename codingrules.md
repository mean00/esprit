# Esprit Rust Coding Rules

This document outlines the architectural guidelines and coding standards for the Rust implementation of the Esprit hardware abstraction layer.

## 1. No Magic Numbers
- The coding style should strictly avoid "magic numbers" in the logic (e.g., raw bitshifts like `(1 << 5)` or literal hex addresses).
- Use a style similar to the existing C++ drivers by extracting constants, masks, and memory addresses into dedicated configuration files (like `registers.rs`).

## 2. Two-Tier Architecture
The drivers must be split into two distinct layers:
- **Contract crate (`rs_mcu_traits`, `rust/rs_mcus/traits/`)**: the MCU-agnostic
  super traits, one file per peripheral (`gpio.rs`, `uart.rs`, `adc.rs`, …), with
  no dependency at all and no chip name in a type. Design: `supertraits.md`.
- **Kernel crate (`rs_rtos`)**: tasks, mutexes/semaphores/queues/events, software
  timers, critical sections and the task-side `sync_cell::SyncCell`. No
  peripheral knowledge.
- **User-Facing Crate (`rust_esprit`)**: Provides a high-level API that is completely independent from the MCU/platform.
- **Platform-Specific Implementations (e.g., `rs_xxxx_bluepill`)**: The underlying implementation crates that are tightly tied to a specific chip or chip family.
  They depend on `rs_mcu_traits` (+ `rs_rtos` when they block), never on
  `rust_esprit`. Ownership and the dependency direction: `layers.md`.

## 3. Re-exporting Types
- Needed types (such as `Pin`, `Mode`, etc.) must be re-exported through the user-facing API so the application layer has a consistent interface regardless of the underlying MCU.

## 4. Examples are Mandatory
- Each crate **must** contain a basic example (e.g., in the `examples/` directory) to show how the driver works. 

## 5. Scope and API Parity
- Do not restrict the Rust crates to only the exact functions currently consumed by the Swindle application.
- Try to propose and implement APIs that reflect the full capabilities and features of the original C++ versions.

## 6. Idiomatic Rust & Legacy C++ Compatibility
- Make the internal code highly idiomatic for Rust (leveraging enums, safe abstractions, etc.).
- **Alias System**: a `#[unsafe(no_mangle)] pub extern "C" fn` alias may exist so legacy C++ keeps
  linking, but it is an **optional adapter, never a design constraint**:
  - the trait (`rs_mcu_traits`, see `supertraits.md`) is the contract; a C-shaped signature
    (`bool` out, `*mut c_void` cookie, `*_c` handle) never shapes it;
  - the adapter lives in its own `src/shim.rs`, behind a `c_abi` cargo feature, one per family
    crate; a pure-Rust build turns it off and loses nothing;
  - no new C entry point without a real C caller in the tree, and no `unimplemented!` behind an
    `Error::Unsupported` — a missing capability returns the error, a missing *adapter* simply is
    not compiled.

## 7. Target build
We have several build targets : GD32F3, CH32V3xx, RP2040, RP2350 which must be dealt with as features.
The GD32F3 & CH32V3xx automatically enable an internal feature called bluepill.
When the bluepill feature is enabled, the rust driver in rust/rs_mcus/bluepill are enabled and built

## 8. Enhanced no_std
The general code is no_std BUT it is perfectly fine to use the FreeRTOS methods, directly or indirectly

