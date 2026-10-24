//! The Esprit logger's C ABI (`lnDebug.cpp`), in Rust.
//!
//! Built only with the `rust_logger` feature (CMake: `USE_RUST_LOGGER`).  In
//! that configuration `esprit/src/CMakeLists.txt` drops `lnDebug.cpp` and this
//! module provides the same symbols with the same semantics:
//!
//! | symbol              | C++ original                          |
//! |---------------------|---------------------------------------|
//! | `Logger`            | `Logger(const char *fmt, ...)`        |
//! | `Logger_C`          | `int Logger_C(const char *fmt, ...)`  |
//! | `Logger_chars`      | `Logger_chars(int, const char *)`     |
//! | `setLogger`         | `setLogger(lnLoggerFunction *)`       |
//! | `LoggerInit`        | `LoggerInit()`                        |
//! | `LoggerInitMutex`   | `LoggerInitMutex()`                   |
//! | `Logger_crash`      | `Logger_crash(const char *)`          |
//! | `lnScratchRegister` | `volatile uint32_t lnScratchRegister` |
//!
//! The output path is [`BufferedTx`] on UART0 -- a 128 byte ring drained by DMA,
//! i.e. the driver `createLnSerialTxOnly(0, /*dma*/ true, /*buffered*/ true)`
//! would build -- so the bytes on the wire are the same as the C++ logger's.
//! `setLogger()` installs an override (that is what `mon redirect` uses); a NULL
//! override means "write to the UART", exactly like the C++ `loggerFunction ==
//! NULL` case, so `mon redirect 0` returns to the UART instead of going dead.
//!
//! Variadic: `Logger` and `Logger_C` are real C-variadic functions.  Rust has
//! had them since 1.99 (`core::ffi::VaList`), and the code generated for the
//! prototype matches clang's on both `thumbv7em-none-eabihf` and
//! `riscv32imafc` -- same prologue, same `va_list` handed to the formatter.
//!
//! The formatter is esprit's embedded printf (`embedded_printf/printf.c`), NOT
//! newlib's: `lnPrintf.h` -> `embedded_printf/printf.h` does
//! `#define vsnprintf vsnprintf_`, so the C++ `Logger` links the `vsnprintf_`
//! symbol and the newlib `printf` family is never referenced.  Rust has to spell
//! `vsnprintf_` out (no preprocessor), otherwise it pulls in newlib's
//! `vsnprintf` -> `vfprintf`, which `ln_libc.cpp` replaces with
//! `do_assert("vfprintf")`.

use core::cell::UnsafeCell;
use core::ffi::{CStr, VaList, c_char, c_int};
use rs_rtos::Mutex;
use rs_uart_bluepill::BufferedTx;

/// The C++ logger's `static char buffer[128]` (`Logger` and `Logger_C` each have
/// their own, so keep two).
const LINE_SIZE: usize = 128;

/// The C++ logger runs the debug UART at a fixed speed (`setSpeed(115200)`).
const BAUDRATE: u32 = 115200;

// UART the logger writes to when nothing is redirected: `lnDebug.cpp`'s
// `int debugUart = 0;` (overridable by CMake `LN_DEBUG_UART`, which maps to
// these features), i.e. USART0 on PA9.
#[cfg(feature = "logger_uart1")]
const LOGGER_UART: u32 = 1;
#[cfg(feature = "logger_uart2")]
const LOGGER_UART: u32 = 2;
#[cfg(not(any(feature = "logger_uart1", feature = "logger_uart2")))]
const LOGGER_UART: u32 = 0;
#[cfg(all(feature = "logger_uart1", feature = "logger_uart2"))]
compile_error!("logger_uart1 and logger_uart2 are mutually exclusive");

/// The C++ `lnLoggerFunction`: `void (*)(int n, const char *data)`.
pub type LoggerFn = Option<unsafe extern "C" fn(n: c_int, data: *const c_char)>;

/// Serialises every log line and protects the two format buffers below, exactly
/// like the C++ `static lnMutex loggerMutex`.  `Mutex` is recursive and keeps
/// its kernel object in its own storage, so this is plain `.bss` (no heap,
/// malloc_used.md §8.1 B1).
static LOGGER_MUTEX: Mutex<()> = Mutex::new(());

/// The C++ `static lnLoggerFunction *loggerFunction`, i.e. the `setLogger()`
/// override.  Read under `LOGGER_MUTEX`, written without it -- same as the C++
/// code, where `setLogger()` does not lock either.
static mut OVERRIDE: LoggerFn = None;

/// The output path: `createLnSerialTxOnly(debugUart, true, true)` + `init()`.
static SINK: BufferedTx = BufferedTx::new(LOGGER_UART);

/// One `static char buffer[128]`.
struct LineBuffer(UnsafeCell<[u8; LINE_SIZE]>);

// SAFETY: a buffer is only touched while its owner holds `LOGGER_MUTEX`
// (`Logger`), or for the single formatting step of `Logger_C`, which is exactly
// how the C++ static buffers are used.
unsafe impl Sync for LineBuffer {}

impl LineBuffer {
    const fn new() -> Self {
        Self(UnsafeCell::new([0; LINE_SIZE]))
    }

    /// Start of the line, as a writable byte pointer.
    fn as_mut_ptr(&self) -> *mut u8 {
        self.0.get() as *mut u8
    }
}

static LOGGER_LINE: LineBuffer = LineBuffer::new();
static LOGGER_C_LINE: LineBuffer = LineBuffer::new();

unsafe extern "C" {
    /// esprit's embedded `vsnprintf_` (`embedded_printf/printf.c`), the function
    /// the C++ side calls through `printf.h`'s `#define vsnprintf vsnprintf_`.
    /// `VaList` is ABI-compatible with C's `va_list` on both targets.
    fn vsnprintf_(s: *mut c_char, n: usize, format: *const c_char, ap: VaList<'_>) -> c_int;
}

/// `strlen()` of a formatted line.  `Logger`/`Logger_C` always terminate the
/// buffer, so the scan stops at the latest at `LINE_SIZE - 1`.
unsafe fn line_len(line: *const u8) -> c_int {
    let mut n = 0;
    while unsafe { *line.add(n) } != 0 {
        n += 1;
    }
    n as c_int
}

/// Format `fmt`/`args` into `line` (leaving room for the terminator), then send
/// it.  The C++ versions call `vsnprintf(buffer, 127, ...)` -- which the
/// preprocessor turns into `vsnprintf_` -- force `buffer[127] = 0` and then use
/// `strlen()`: same thing here.
unsafe fn format_line(line: *mut u8, fmt: *const c_char, args: VaList<'_>) {
    // The return value is the length that *would* have been written; the C++
    // code ignores it and re-measures with strlen, so this does too.
    unsafe {
        vsnprintf_(line as *mut c_char, LINE_SIZE - 1, fmt, args);
        *line.add(LINE_SIZE - 1) = 0;
    }
}

/// The single funnel every producer goes through (`Logger`, `Logger_C`, the Rust
/// `logger!` macro, ...): the `setLogger()` override if there is one, the UART
/// otherwise.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Logger_chars(n: c_int, data: *const c_char) {
    // A zero sized DMA transfer does not work (C++: `if (!n) return;`).
    if n <= 0 || data.is_null() {
        return;
    }
    let _guard = LOGGER_MUTEX.lock();
    // SAFETY: `OVERRIDE` is only assigned by `setLogger()`.
    let target = unsafe { OVERRIDE };
    match target {
        Some(f) => unsafe { f(n, data) },
        None => {
            // SAFETY: the caller promises `n` readable bytes.
            let bytes = unsafe { core::slice::from_raw_parts(data as *const u8, n as usize) };
            SINK.write(bytes);
        }
    }
}

/// `Logger(const char *fmt, ...)`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Logger(fmt: *const c_char, args: ...) {
    if fmt.is_null() {
        return;
    }
    // SAFETY: `fmt` is a C string; the C++ code reads `fmt[0]` unconditionally.
    if unsafe { *fmt } == 0 {
        return;
    }
    let _guard = LOGGER_MUTEX.lock();
    let line = LOGGER_LINE.as_mut_ptr();
    unsafe {
        format_line(line, fmt, args);
        Logger_chars(line_len(line), line as *const c_char);
    }
}

/// `int Logger_C(const char *fmt, ...)`; always returns 0, like the C++ one.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Logger_C(fmt: *const c_char, args: ...) -> c_int {
    if fmt.is_null() {
        return 0;
    }
    // No lock around the formatting: the C++ `Logger_C` has none either (its
    // buffer is separate from `Logger`'s, and `Logger_chars` locks below).
    let line = LOGGER_C_LINE.as_mut_ptr();
    unsafe {
        format_line(line, fmt, args);
        Logger_chars(line_len(line), line as *const c_char);
    }
    0
}

/// Install (or with `None`, remove) the output override.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn setLogger(f: LoggerFn) {
    // SAFETY: single word store, and the reader runs from task context; the C++
    // `setLogger()` is equally unsynchronised.
    unsafe { OVERRIDE = f };
}

/// The logger's mutex needs no initialisation: `LOGGER_MUTEX` is a `.bss` object
/// that creates its kernel object on first use, like the C++ `lnMutex` it
/// replaces.  Kept as a symbol because `ln_esp_sysinit.cpp` calls it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LoggerInitMutex() {}

/// Set the debug UART up.  The C++ version allocates a
/// `lnSerialBpTxOnlyBufferedDma` on the heap and calls `init()`/`setSpeed()`;
/// here the sink already lives in `.bss` (`SINK`) and `init()` is idempotent.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn LoggerInit() {
    // SAFETY: plain calls, no invariants to uphold.
    unsafe {
        LoggerInitMutex();
        SINK.init(BAUDRATE);
    }
}

/// Last-resort output for a crash report: C++
/// `serial0->init(); serial0->setSpeed(115200); serial0->rawWrite(strlen(st), st)`.
/// `raw_write` is polled (no interrupt, no DMA), so it works whatever state the
/// system is in.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Logger_crash(st: *const c_char) {
    if st.is_null() {
        return;
    }
    SINK.init(BAUDRATE);
    // SAFETY: `st` is a NUL terminated C string (the C++ code runs strlen on it).
    let text = unsafe { CStr::from_ptr(st) }.to_bytes();
    SINK.raw_write(text);
}

/// `volatile uint32_t lnScratchRegister` (esprit.h): the global the per-MCU code
/// pokes to keep a store from being optimised away.  Volatility is a property of
/// the accesses -- the C ones stay volatile -- so a plain word is the same
/// object.
#[unsafe(no_mangle)]
pub static mut lnScratchRegister: u32 = 0;

// EOF
