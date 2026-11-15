//! Core Rust runtime, timebase, allocator, libc stubs, FastEventGroup, and FreeRTOS debug table.
//!
//! Replaces `lnSystemTime.cpp`, `lnStopWatch.cpp`, `systemHelper.cpp`,
//! `systemAlloc.cpp`, `ln_libc.cpp`, `lnFreeRTOSDebug.cpp`, and `lnFast_EventGroup_c.cpp`
//! when `LN_ENABLE_RUST` is enabled on GD32F3x, CH32V3x, RP2040, or RP2350.

use core::ffi::{c_char, c_int, c_uint, c_void};
use core::mem::offset_of;
use core::sync::atomic::{AtomicU32, Ordering};

use rs_rtos::c_freertos as freertos;

use crate::platform::mcu;

unsafe extern "C" {
    fn pvPortMalloc(xSize: usize) -> *mut c_void;
    fn vPortFree(pv: *mut c_void);
    #[link_name = "\u{1}_Z5setupv"]
    fn cpp_setup();
    #[link_name = "\u{1}_Z4loopv"]
    fn cpp_loop();
}

pub(crate) unsafe extern "C" fn entry_task(_param: *mut c_void) {
    unsafe {
        crate::logger_c::LoggerInit();
        cpp_setup();
        loop {
            cpp_loop();
        }
    }
}

// ---------------------------------------------------------------------------
// Millisecond timebase (replaces lnSystemTime.cpp)
// ---------------------------------------------------------------------------

static MY_TICK: AtomicU32 = AtomicU32::new(0);

#[unsafe(no_mangle)]
pub extern "C" fn vApplicationTickHook() {
    // Called from SysTick ISR; Relaxed fetch_add/load+store is safe on single-core M4.
    let v = MY_TICK.load(Ordering::Relaxed).wrapping_add(1);
    MY_TICK.store(v, Ordering::Relaxed);
}

#[unsafe(export_name = "_Z7lnGetMsv")]
pub extern "C" fn ln_get_ms() -> u32 {
    MY_TICK.load(Ordering::Relaxed)
}

#[unsafe(no_mangle)]
pub extern "C" fn lnGetMs_c() -> u32 {
    MY_TICK.load(Ordering::Relaxed)
}

// ---------------------------------------------------------------------------
// System helpers & delays (replaces systemHelper.cpp)
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub extern "C" fn lnInterrupts() {
    unsafe { freertos::vPortExitCritical() }
}

#[unsafe(no_mangle)]
pub extern "C" fn lnNoInterrupt() {
    unsafe { freertos::vPortEnterCritical() }
}

#[unsafe(no_mangle)]
pub extern "C" fn lnDelayUs(wait: u32) {
    let target = mcu::ln_get_us64().wrapping_add(wait as u64);
    loop {
        if mcu::ln_get_us64() > target {
            return;
        }
        // SAFETY: `nop` has no side effects.
        unsafe { core::arch::asm!("nop", options(nomem, nostack, preserves_flags)) };
    }
}

#[unsafe(export_name = "_Z9lnDelayUsj")]
pub extern "C" fn ln_delay_us_j(wait: u32) {
    lnDelayUs(wait);
}

#[unsafe(export_name = "_Z9lnDelayUsm")]
pub extern "C" fn ln_delay_us_m(wait: u32) {
    lnDelayUs(wait);
}

#[unsafe(export_name = "_Z6xDelayj")]
pub extern "C" fn x_delay_j(wait: u32) {
    lnDelay_C(wait);
}

#[unsafe(export_name = "_Z6xDelaym")]
pub extern "C" fn x_delay_m(wait: u32) {
    lnDelay_C(wait);
}

#[unsafe(no_mangle)]
pub extern "C" fn lnDelay_C(ms: u32) {
    let dely = 1 + ms; // portTICK_PERIOD_MS == 1 (configTICK_RATE_HZ == 1000)
    unsafe { freertos::vTaskDelay(dely) };
}

#[unsafe(export_name = "_Z7lnDelayj")]
pub extern "C" fn ln_delay_j(ms: u32) {
    lnDelay_C(ms);
}

#[unsafe(export_name = "_Z7lnDelaym")]
pub extern "C" fn ln_delay_m(ms: u32) {
    lnDelay_C(ms);
}

// ---------------------------------------------------------------------------
// Allocator & C++ new/delete (replaces systemAlloc.cpp)
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub extern "C" fn sysAlloc(size: c_int) -> *mut c_void {
    if size < 0 {
        mcu::deadEnd(0xA000);
    }
    let ptr = unsafe { pvPortMalloc(size as usize) };
    if ptr.is_null() {
        mcu::deadEnd(0xA000);
    }
    ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn sysFree(p: *mut c_void) {
    unsafe { vPortFree(p) }
}

#[unsafe(no_mangle)]
pub extern "C" fn malloc(size: usize) -> *mut c_void {
    sysAlloc(size as c_int)
}

#[unsafe(no_mangle)]
pub extern "C" fn free(p: *mut c_void) {
    sysFree(p)
}

#[unsafe(no_mangle)]
pub extern "C" fn calloc(n: usize, elem_size: usize) -> *mut c_void {
    let total = n.saturating_mul(elem_size);
    let ptr = sysAlloc(total as c_int);
    if !ptr.is_null() && total > 0 {
        // SAFETY: `sysAlloc` returned a valid non-null allocation of `total` bytes.
        unsafe { core::ptr::write_bytes(ptr as *mut u8, 0, total) };
    }
    ptr
}

#[unsafe(export_name = "_Znwj")]
pub extern "C" fn cpp_new(size: usize) -> *mut c_void {
    sysAlloc(size as c_int)
}

#[unsafe(export_name = "_Znaj")]
pub extern "C" fn cpp_new_array(size: usize) -> *mut c_void {
    sysAlloc(size as c_int)
}

#[unsafe(export_name = "_ZdlPv")]
pub extern "C" fn cpp_delete(p: *mut c_void) {
    sysFree(p)
}

#[unsafe(export_name = "_ZdlPvj")]
pub extern "C" fn cpp_delete_sized(p: *mut c_void, _n: usize) {
    sysFree(p)
}

#[unsafe(export_name = "_ZdaPv")]
pub extern "C" fn cpp_delete_array(p: *mut c_void) {
    sysFree(p)
}

// ---------------------------------------------------------------------------
// Libc / libstdc++ stubs (replaces ln_libc.cpp)
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub static stderr: usize = 0;

#[unsafe(no_mangle)]
pub extern "C" fn _getpid() -> c_int {
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn _kill() -> c_int {
    0
}

// ---------------------------------------------------------------------------
// FreeRTOS debug info table (replaces lnFreeRTOSDebug.cpp)
// ---------------------------------------------------------------------------

const LN_FREERTOS_MAGIC: i32 = 0x1FEE_BAE;
const LAYOUT_ARM_FPU: i32 = if mcu::runtime::HAS_FPU { 1 } else { 0 };
const CONFIG_MAX_PRIORITIES: i32 = 16;
const CONFIG_MAX_TASK_NAME_LEN: usize = 16;

#[repr(C)]
struct CopyOfTcbStruct {
    px_top_of_stack: *mut freertos::StackType_t,
    x_state_list_item: freertos::ListItem_t,
    x_event_list_item: freertos::ListItem_t,
    ux_priority: freertos::UBaseType_t,
    px_stack: *mut freertos::StackType_t,
    pc_task_name: [u8; CONFIG_MAX_TASK_NAME_LEN],
    ux_tcb_number: freertos::UBaseType_t,
    ux_task_number: freertos::UBaseType_t,
}

#[repr(C)]
pub struct LnFreeRtosDebug {
    pub magic: i32,
    pub list_size: i32,
    pub offset_list_item_next: i32,
    pub offset_list_item_owner: i32,
    pub offset_list_number_of_item: i32,
    pub offset_list_index: i32,
    pub nb_of_priorities: i32,
    pub layout_type: i32,
    pub max_task_name_len: i32,
    pub offset_task_name: i32,
    pub offset_task_num: i32,
}

#[used]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".rodata.freertos_debug")]
pub static freeRTOSDebug: LnFreeRtosDebug = LnFreeRtosDebug {
    magic: LN_FREERTOS_MAGIC,
    list_size: core::mem::size_of::<freertos::List_t>() as i32,
    offset_list_item_next: offset_of!(freertos::xLIST_ITEM, pxNext) as i32,
    offset_list_item_owner: offset_of!(freertos::xLIST_ITEM, pvOwner) as i32,
    offset_list_number_of_item: offset_of!(freertos::xLIST, uxNumberOfItems) as i32,
    offset_list_index: offset_of!(freertos::xLIST, pxIndex) as i32,
    nb_of_priorities: CONFIG_MAX_PRIORITIES,
    layout_type: LAYOUT_ARM_FPU,
    max_task_name_len: CONFIG_MAX_TASK_NAME_LEN as i32,
    offset_task_name: offset_of!(CopyOfTcbStruct, pc_task_name) as i32,
    offset_task_num: offset_of!(CopyOfTcbStruct, ux_tcb_number) as i32,
};

#[unsafe(no_mangle)]
pub extern "C" fn lnGetFreeRTOSDebug() -> *const u32 {
    (&raw const freeRTOSDebug) as *const u32
}

// ---------------------------------------------------------------------------
// Static task creation (replaces remaining lnFreeRTOS_pp.cpp exports)
// ---------------------------------------------------------------------------

pub(crate) fn create_task_static(
    task: unsafe extern "C" fn(*mut c_void),
    name: &'static core::ffi::CStr,
    stack_size_in_bytes: u32,
    priority: freertos::UBaseType_t,
    stack: &'static mut [freertos::StackType_t],
    tcb: &'static mut freertos::StaticTask_t,
) {
    let stack_depth = if mcu::runtime::HAS_FPU {
        (stack_size_in_bytes / 4) + 32
    } else {
        stack_size_in_bytes / 4
    };
    assert!(stack.len() >= stack_depth as usize);
    // SAFETY: `stack` and `tcb` have `'static` lifetimes and `stack` is at least `stack_depth` words.
    let handle = unsafe {
        freertos::xTaskCreateStatic(
            Some(task),
            name.as_ptr(),
            stack_depth,
            core::ptr::null_mut(),
            priority,
            stack.as_mut_ptr(),
            tcb as *mut freertos::StaticTask_t,
        )
    };
    if handle.is_null() {
        mcu::deadEnd(0xA000);
    }
}
