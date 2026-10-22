//! Constants of the FreeRTOS kernel provided by ESP-IDF.
//!
//! Selected by `--features esp32` (see `mod.rs`).  This is *not* the kernel in
//! `esprit/FreeRTOS`: ESP-IDF ships its own build of FreeRTOS with its own
//! `FreeRTOSConfig.h` (`sdkconfig.h`), so neither the control block size nor
//! the tick rate may be assumed to match `esprit.rs`.  Measured values for
//! `sizeof(StaticQueue_t)`: 80 in ours, 84 here.

/// `sizeof(StaticQueue_t)` / `sizeof(StaticSemaphore_t)` of the ESP-IDF kernel.
///
/// Measured from the kernel itself rather than guessed: the DWARF of
/// `queue.c.obj` inside `libfreertos.a` says `DW_AT_byte_size = 84` for
/// `QueueDefinition` on every ESP-IDF variant we have a build of (esp32c3
/// riscv32imc, esp32s3 xtensa, esp32s3 riscv32imac, IDF 5.x and 6.0).  Re-measure
/// when the IDF version changes.
///
/// We cannot hook that measurement into ESP-IDF's build, so the ESP32 project
/// should keep the guard on its side: a `_Static_assert(sizeof(StaticQueue_t)
/// == 84, "update rs_rtos/src/kernel/esp32.rs")` next to its FreeRTOS glue
/// turns a kernel bump into a compile error there.
pub const STATIC_QUEUE_BYTES: usize = 84;

/// `CONFIG_FREERTOS_HZ` from `sdkconfig.h` (1000 by default, which happens to
/// match the in-tree kernel - it is a per-kernel setting, not a constant).
pub const TICK_RATE_HZ: u32 = 1000;
