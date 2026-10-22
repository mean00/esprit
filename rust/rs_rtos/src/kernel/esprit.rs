//! Constants of the FreeRTOS kernel shipped in `esprit/FreeRTOS`, built with
//! `esprit/freertos_config/FreeRTOSConfig.h`.
//!
//! This is the kernel of every in-tree target (GD32F303, CH32V307, RP2040,
//! RP2350).  All of them are 32-bit with 4-byte pointers and run the same
//! config, so the numbers are the same on all of them; what varies per target
//! (`configMINIMAL_STACK_SIZE`, `configTOTAL_HEAP_SIZE`) is not used by Rust.
//!
//! The numbers are not written here: `generated.rs` next to this file is
//! produced by `esprit/cmake/lnFreeRTOS_structs.cmake` from a probe that the
//! build compiles with the very flags and `FreeRTOSConfig.h` of this kernel
//! (`esprit/cmake/lnFreeRTOS_structs_probe.c`), and it is rewritten whenever the
//! measurement changes - so a kernel layout change propagates to Rust on the
//! next build instead of leaving a stale constant behind.  This module only
//! documents what those numbers mean; the contract they have to satisfy is in
//! `super` (`kernel/mod.rs`).

mod generated;

pub(crate) use generated::*;
