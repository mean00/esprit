//! Minimal libc / libstdc++ / FreeRTOS fault stubs that halt via `deadEnd`.

use core::ffi::{c_char, c_int, c_void};

use crate::platform::mcu::deadEnd;

#[unsafe(no_mangle)]
pub extern "C" fn handle_trap(_mcause: usize, _sp: usize) -> usize {
    unsafe { deadEnd(0xffff) }
}

#[unsafe(no_mangle)]
pub extern "C" fn unhandledException() -> ! {
    unsafe { deadEnd(0x1000) }
}

#[unsafe(no_mangle)]
pub extern "C" fn vApplicationStackOverflowHook(_x_task: *mut c_void, _pc_task_name: *mut c_char) -> ! {
    unsafe { deadEnd(0x1001) }
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
