//! Minimal libc / libstdc++ / FreeRTOS fault stubs that halt via `deadEnd`.

use core::ffi::{c_char, c_int, c_void};

use crate::platform::mcu::deadEnd;

unsafe extern "C" {
    /// Polled write straight to the debug UART (`esprit/cpp/src/lnDebug.cpp`): no kernel
    /// object, no mutex, no allocation, so it is safe to call from the scheduler.
    fn Logger_crash(st: *const c_char);
}

/// Name of the task whose stack overflowed, kept for post-mortem inspection.
///
/// `configMAX_TASK_NAME_LEN` is 16, so 17 bytes hold a full name plus its NUL. The hook
/// parks the CPU right after recording it, so a debugger (or the next boot's log) can tell
/// which task to give more stack.
const OVERFLOW_TASK_LEN: usize = 17;
static mut OVERFLOW_TASK: [u8; OVERFLOW_TASK_LEN] = [0; OVERFLOW_TASK_LEN];

#[unsafe(no_mangle)]
pub extern "C" fn handle_trap(_mcause: usize, _sp: usize) -> usize {
    unsafe { deadEnd(0xffff) }
}

#[unsafe(no_mangle)]
pub extern "C" fn unhandledException() -> ! {
    unsafe { deadEnd(0x1000) }
}

/// `configCHECK_FOR_STACK_OVERFLOW = 2` lands here with the offending task.
///
/// The previous version parked the CPU silently, which left a wedged probe and nothing to
/// go on; report the task name over the crash UART first (and keep it in `OVERFLOW_TASK`),
/// then park exactly like every other fatal path.
#[unsafe(no_mangle)]
pub extern "C" fn vApplicationStackOverflowHook(_x_task: *mut c_void, pc_task_name: *mut c_char) -> ! {
    unsafe {
        Logger_crash(c"**** STACK OVERFLOW: ".as_ptr());
        if pc_task_name.is_null() {
            Logger_crash(c"(no task name)\n".as_ptr());
        } else {
            let mut len = 0;
            while len < OVERFLOW_TASK_LEN - 1 && *pc_task_name.add(len) != 0 {
                OVERFLOW_TASK[len] = *pc_task_name.add(len) as u8;
                len += 1;
            }
            OVERFLOW_TASK[len] = 0;
            Logger_crash(pc_task_name);
            Logger_crash(c"\n".as_ptr());
        }
        deadEnd(0x1001)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn vApplicationMallocFailedHook() -> ! {
    unsafe { deadEnd(0xA000) }
}

#[unsafe(no_mangle)]
pub extern "C" fn realloc(_ptr: *mut c_void, _size: usize) -> *mut c_void {
    unsafe { deadEnd(0xA000) }
}

#[unsafe(no_mangle)]
pub extern "C" fn _exit(code: c_int) -> ! {
    unsafe { deadEnd(code.wrapping_add(0x2000)) }
}

#[unsafe(no_mangle)]
pub extern "C" fn panic(_fmt: *const c_char) -> ! {
    unsafe { deadEnd(0xA000) }
}

#[unsafe(no_mangle)]
pub extern "C" fn getpid() -> c_int {
    unsafe { deadEnd(0xA000) }
}

#[unsafe(no_mangle)]
pub extern "C" fn kill(_pid: c_int, _sig: c_int) -> c_int {
    unsafe { deadEnd(0xA000) }
}

#[unsafe(no_mangle)]
pub extern "C" fn getppid() -> c_int {
    unsafe { deadEnd(0xA000) }
}

#[unsafe(no_mangle)]
pub extern "C" fn __cxa_pure_virtual() -> ! {
    unsafe { deadEnd(0xA000) }
}

#[unsafe(no_mangle)]
pub extern "C" fn core_exception_handler(_mcause: usize, _sp: usize) -> u32 {
    unsafe { deadEnd(0xA000) }
}

#[unsafe(no_mangle)]
pub extern "C" fn abort() -> ! {
    unsafe { deadEnd(0xA000) }
}

#[unsafe(no_mangle)]
pub extern "C" fn vfprintf(_stream: *mut c_void) -> c_int {
    unsafe { deadEnd(0xA000) }
}

#[unsafe(no_mangle)]
pub extern "C" fn fprintf(_stream: *mut c_void) -> c_int {
    unsafe { deadEnd(0xA000) }
}

#[unsafe(export_name = "_ZSt20__throw_out_of_rangePKc")]
pub extern "C" fn std_throw_out_of_range(_a: *const c_char) -> ! {
    unsafe { deadEnd(0xA000) }
}

#[unsafe(export_name = "_ZSt20__throw_length_errorPKc")]
pub extern "C" fn std_throw_length_error(_a: *const c_char) -> ! {
    unsafe { deadEnd(0xA000) }
}

#[unsafe(no_mangle)]
pub extern "C" fn do_assert(_a: *const c_char) -> ! {
    unsafe { deadEnd(0xA000) }
}
