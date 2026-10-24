//! Constants of the FreeRTOS **kernel instance** we are built against.
//!
//! The same Rust source can be linked against two different kernels: the one
//! esprit compiles itself (`esprit/FreeRTOS`, used by GD32F303, CH32V307,
//! RP2040/RP2350 - see `esprit.rs`) and the one a platform provides (ESP-IDF,
//! used by the esp32 build - see `esp32.rs`).  They do **not** agree: the
//! control block of a queue is 80 bytes in ours and 84 in ESP-IDF's, because
//! the two kernels are different code with different config.  The CPU does not
//! matter, the kernel does.
//!
//! So the numbers live in one module per kernel, and exactly one module is
//! compiled: the same cargo feature that already selects the platform selects
//! the kernel (`rs_swindle/esp32 -> rust_esprit/esp32 -> rs_rtos/esp32`).  The
//! default is the in-tree kernel, which is what every build of this repository
//! links, so a build that does not ask for a platform kernel cannot get the
//! wrong numbers.
//!
//! # Contract
//!
//! Every kernel file defines the whole set below, because `crate::sync` and
//! `crate::task` use it unconditionally:
//!
//! * [`STATIC_QUEUE_BYTES`] - `sizeof(StaticQueue_t)` (`sizeof(StaticSemaphore_t)`
//!   is the same number: both names are `Queue_t` under a different spelling)
//! * [`STATIC_TIMER_BYTES`] - `sizeof(StaticTimer_t)`, the storage behind a
//!   statically created software timer (`crate::timer`)
//! * [`TICK_RATE_HZ`] - `configTICK_RATE_HZ` of that kernel
//!
//! Values that do not depend on the kernel stay where the kernel is declared:
//! the `queueQUEUE_TYPE_*` tags and the `*Static()` prototypes are part of the
//! FreeRTOS API and live in `sync.rs` / the bindgen output.
//!
//! # Keeping a file honest
//!
//! A wrong number here is silent memory corruption, so no number is typed by
//! hand where the build can measure it:
//!
//! * in-tree kernel: `esprit/cmake/lnFreeRTOS_structs_probe.c` measures the
//!   sizes with the build's own flags and `lnFreeRTOS_structs.cmake` *writes*
//!   them into `esprit/generated.rs`, which `esprit.rs` re-exports.  The file is
//!   rewritten only when a number changes (so cargo's fingerprint is stable),
//!   and the consumer makes the measurement a prerequisite of its cargo build
//!   (`swindle/rs/CMakeLists.txt`) - a `FreeRTOSConfig.h` change therefore lands
//!   in the crate on the next build, and a parallel build cannot compile the
//!   previous layout;
//! * ESP-IDF kernel: we do not own that build system, so it can measure nothing
//!   for us - those numbers stay hand-written in `esp32.rs` and the check
//!   belongs next to the kernel they describe: a `_Static_assert()` in the ESP32
//!   project's C glue (see the comment in `esp32.rs`).
cfg_if::cfg_if! {
    if #[cfg(feature = "esp32")] {
        mod esp32;
        pub(crate) use esp32::*;
    } else {
        mod esprit;
        pub(crate) use esprit::*;
    }
}
