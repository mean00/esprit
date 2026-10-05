# Esprit Rust Coding Rules

This document outlines the architectural guidelines and coding standards for the Rust implementation of the Esprit hardware abstraction layer.

## 1. No Magic Numbers
- The coding style should strictly avoid "magic numbers" in the logic (e.g., raw bitshifts like `(1 << 5)` or literal hex addresses).
- Use a style similar to the existing C++ drivers by extracting constants, masks, and memory addresses into dedicated configuration files (like `registers.rs`).

## 2. Two-Tier Architecture
The drivers must be split into two distinct layers:
- **User-Facing Crate (`rust_esprit`)**: Provides a high-level API that is completely independent from the MCU/platform.
- **Platform-Specific Implementations (e.g., `rs_xxxx_bluepill`)**: The underlying implementation crates that are tightly tied to a specific chip or chip family.

## 3. Re-exporting Types
- Needed types (such as `Pin`, `Mode`, etc.) must be re-exported through the user-facing API so the application layer has a consistent interface regardless of the underlying MCU.

## 4. Examples are Mandatory
- Each crate **must** contain a basic example (e.g., in the `examples/` directory) to show how the driver works. 

## 5. Scope and API Parity
- Do not restrict the Rust crates to only the exact functions currently consumed by the Swindle application.
- Try to propose and implement APIs that reflect the full capabilities and features of the original C++ versions.

## 6. Idiomatic Rust & Legacy C++ Compatibility
- Make the internal code highly idiomatic for Rust (leveraging enums, safe abstractions, etc.).
- **Alias System**: Provide an alias system (via `#[unsafe(no_mangle)] pub extern "C" fn`) to remain completely compatible with the C-API used for wrapping other legacy C++ code in the project.

## 7. Target build
We have several build targets : GD32F3, CH32V3xx, RP2040, RP2350 which must be dealt with as features.
The GD32F3 & CH32V3xx automatically enable an internal feature called bluepill.
When the bluepill feature is enabled, the rust driver in rust/bluepill are enabled and built

## 8. Enhanced no_std
The general code is no_std BUT it is perfectly fine to use the FreeRTOS methods, directly or indirectly

