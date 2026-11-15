//! RP2040 / RP2350 Rust startup and entry task.
//!
//! Replaces `arm_rp2040/src/rp.cpp` and `arm_rp2350/src/rp.cpp` in
//! `LN_ENABLE_RUST` builds, leaving only minimal C/pico-sdk boot glue in
//! `rp_rust_boot.c`.

use core::ffi::{c_char, c_void};

use rs_rtos::c_freertos as freertos;

use crate::platform::mcu;
use crate::runtime::{create_task_static, entry_task};

const ENTRY_TASK_STACK_BYTES: u32 = 8 * 1024;
const ENTRY_TASK_PRIORITY: freertos::UBaseType_t = 2;
const ENTRY_TASK_STACK_WORDS: usize = if mcu::runtime::HAS_FPU {
    (ENTRY_TASK_STACK_BYTES as usize / 4) + 32
} else {
    ENTRY_TASK_STACK_BYTES as usize / 4
};

static mut ENTRY_TASK_TCB: freertos::StaticTask_t = unsafe { core::mem::zeroed() };
static mut ENTRY_TASK_STACK: [freertos::StackType_t; ENTRY_TASK_STACK_WORDS] =
    [0; ENTRY_TASK_STACK_WORDS];

#[unsafe(no_mangle)]
pub extern "C" fn rp_rust_main() -> ! {
    // SAFETY: Hardware init, IRQ handlers, and static task buffers live for firmware lifetime.
    unsafe {
        mcu::sys_init();
        create_task_static(
            entry_task,
            c"entryTask",
            ENTRY_TASK_STACK_BYTES,
            ENTRY_TASK_PRIORITY,
            &mut *(&raw mut ENTRY_TASK_STACK),
            &mut *(&raw mut ENTRY_TASK_TCB),
        );
        freertos::vTaskStartScheduler();
    }
    mcu::deadEnd(25)
}

/// C++ `lnUsbStack.cpp` on RP2xxx still calls `lnCreateTaskStatic`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lnCreateTaskStatic(
    task: freertos::TaskFunction_t,
    name: *const c_char,
    stack_size_in_bytes: u32,
    param: *mut c_void,
    priority: freertos::UBaseType_t,
    stack: *mut freertos::StackType_t,
    tcb: *mut freertos::StaticTask_t,
) -> bool {
    let stack_depth = if mcu::runtime::HAS_FPU {
        (stack_size_in_bytes / 4) + 32
    } else {
        stack_size_in_bytes / 4
    };
    let handle = unsafe {
        freertos::xTaskCreateStatic(task, name, stack_depth, param, priority, stack, tcb)
    };
    !handle.is_null()
}
