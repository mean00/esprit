//! The USB device stack: the task that pumps tinyUSB, the descriptor plumbing and the
//! `tud_*` application callbacks that tinyUSB calls back into.
//!
//! This is the Rust replacement for `esprit/libraries/tinyUsb/lnSrc/lnUsbStack.cpp`
//! (see `usb.md`, P2). Everything below tinyUSB - the core, the class drivers and the
//! DCD/port layer - is untouched: this module only *calls* `tusb_init`/`tud_task` and
//! *provides* the `TU_ATTR_WEAK` callbacks tinyUSB would otherwise leave empty.
//!
//! Nothing here knows how many CDC interfaces there are, who uses them or what the
//! descriptors say: the pointers are handed in by the board (`rn_usb_stack_init`) and
//! returned verbatim from the callbacks, because tinyUSB keeps them until the transfer
//! completes.

use core::ffi::{c_char, c_void};
use core::sync::atomic::{AtomicBool, AtomicI32, AtomicPtr, Ordering};

use rs_rtos::task::{CTaskEntry, TaskHandle, TaskStorage};

// --- tinyUSB ---------------------------------------------------------------------
//
// Only real symbols: `tusb_init()` and `tud_task()` are macros/inline wrappers in
// tinyUSB (`tusb.h:150`, `usbd.h:68`), so the stack calls what they expand to.
unsafe extern "C" {
    fn tusb_rhport_init(rhport: u8, rh_init: *const core::ffi::c_void) -> bool;
    fn tud_task_ext(timeout_ms: u32, in_isr: bool);
    fn tud_mounted() -> bool;
    fn tud_suspended() -> bool;
    fn tud_speed_get() -> u32;
    fn vTaskDelete(xTaskToDelete: *mut c_void);
}

/// `tusb_init()` - tinyUSB's own expansion (`#define tusb_init() tusb_rhport_init(0, NULL)`).
#[inline(always)]
unsafe fn tusb_init() -> bool {
    unsafe { tusb_rhport_init(0, core::ptr::null()) }
}

/// `tud_task()` - `tud_task_ext(UINT32_MAX, false)`, i.e. block until an event arrives.
#[inline(always)]
unsafe fn tud_task() {
    unsafe { tud_task_ext(u32::MAX, false) }
}

/// `tud_ready()` - `mounted && !suspended` (`usbd.h:97`).
fn tud_ready() -> bool {
    unsafe { tud_mounted() && !tud_suspended() }
}

/// `tusb_speed_t` (`tinyUsb/src/src/common/tusb_types.h:87`).
pub const TUSB_SPEED_FULL: u32 = 0;
pub const TUSB_SPEED_LOW: u32 = 1;
pub const TUSB_SPEED_HIGH: u32 = 2;

/// Stack of the `usbd` task, in words: the 1500 B of the C++ `USBD_STACK_SIZE`
/// (`lnUsbStack.cpp`), rounded up. Static, because the task lives as long as the probe.
pub const USB_TASK_STACK_WORDS: usize = 384;
/// Priority of the `usbd` task: the C++ used `configMAX_PRIORITIES - 1`, i.e. the
/// highest one the kernel offers.
pub const USB_TASK_PRIORITY: u32 = 6;

/// Descriptors, handed in once by the board before [`rn_usb_stack_start`].
///
/// `device`, `qualifier`, `fs_config` and `hs_config` point at the `tusb_desc_*`
/// structures of the board (`swindle/private_include/lnBMP_usb_descriptor*.h`);
/// `strings` is the `const char *descriptor[]` table (index 0 = the language id).
#[repr(C)]
pub struct Descriptors {
    pub device: *const u8,
    pub qualifier: *const u8,
    pub fs_config: *const u8,
    pub hs_config: *const u8,
    pub strings: *const *const c_char,
    pub string_count: usize,
}

/// `void (*)(void *cookie, int event)`, the shape the C++ `lnUsbStackEventHandler` had.
pub type EventHandler = Option<unsafe extern "C" fn(cookie: *mut c_void, event: i32)>;

/// The events the stack reports - the C++ `lnUsbStack::lnUsbStackEvents`.
pub const USB_CONNECT: i32 = 0;
pub const USB_DISCONNECT: i32 = 1;
pub const USB_SUSPEND: i32 = 2;
pub const USB_RESUME: i32 = 3;

struct StringBuf(core::cell::UnsafeCell<[u16; 64]>);
// SAFETY: `tud_descriptor_string_cb` is only called from the `usbd` task, one request at a time.
unsafe impl Sync for StringBuf {}

struct DescStorage(core::cell::UnsafeCell<Descriptors>);
// SAFETY: written once in `rn_usb_stack_init` before `rn_usb_stack_start`, then read-only.
unsafe impl Sync for DescStorage {}

static DESCRIPTORS: AtomicPtr<Descriptors> = AtomicPtr::new(core::ptr::null_mut());
static EVENT_HANDLER: AtomicPtr<c_void> = AtomicPtr::new(core::ptr::null_mut());
static EVENT_COOKIE: AtomicPtr<c_void> = AtomicPtr::new(core::ptr::null_mut());
static LAST_EVENT: AtomicI32 = AtomicI32::new(-1);
static RUNNING: AtomicBool = AtomicBool::new(false);
static STARTED: AtomicBool = AtomicBool::new(false);

static USB_TASK: TaskStorage<USB_TASK_STACK_WORDS> = TaskStorage::new();

#[inline]
fn descriptors() -> Option<&'static Descriptors> {
    let p = DESCRIPTORS.load(Ordering::Acquire);
    if p.is_null() {
        None
    } else {
        Some(unsafe { &*p })
    }
}

fn post_event(event: i32) {
    LAST_EVENT.store(event, Ordering::Release);
    let cb = EVENT_HANDLER.load(Ordering::Acquire);
    if !cb.is_null() {
        let cookie = EVENT_COOKIE.load(Ordering::Acquire);
        // SAFETY: the handler pointer is only ever set from `rn_usb_stack_set_event_handler`
        // with the very type asserted here, and it stays valid for the life of the probe.
        // Note: `post_event` runs inside the high-priority `usbd` task (`tud_*` callbacks);
        // registered handlers must never block or acquire locks (such as `LOGGER_MUTEX`)
        // held by lower-priority tasks waiting on USB I/O.
        let handler: unsafe extern "C" fn(*mut c_void, i32) = unsafe { core::mem::transmute(cb) };
        unsafe { handler(cookie, event) };
    }
}

// --- the task ---------------------------------------------------------------------

unsafe extern "C" fn usb_task(_param: *mut c_void) {
    unsafe { tusb_init() };
    while RUNNING.load(Ordering::Acquire) {
        unsafe { tud_task() };
    }
    // A FreeRTOS task must never return (lnUsbStack::task() deleted itself too).
    unsafe { vTaskDelete(core::ptr::null_mut()) };
}

// --- `tud_*` application callbacks ------------------------------------------------

/// Invoked when the device is mounted (bus powered, ready).
#[unsafe(no_mangle)]
pub extern "C" fn tud_mount_cb() {
    post_event(USB_CONNECT);
}

/// Invoked when the device is unmounted.
#[unsafe(no_mangle)]
pub extern "C" fn tud_umount_cb() {
    post_event(USB_DISCONNECT);
}

/// Invoked when the bus is suspended.
#[unsafe(no_mangle)]
pub extern "C" fn tud_suspend_cb(_remote_wakeup_en: bool) {
    post_event(USB_SUSPEND);
}

/// Invoked when the bus is resumed.
#[unsafe(no_mangle)]
pub extern "C" fn tud_resume_cb() {
    post_event(USB_RESUME);
}

/// GET DEVICE DESCRIPTOR.
#[unsafe(no_mangle)]
pub extern "C" fn tud_descriptor_device_cb() -> *const u8 {
    descriptors().map_or(core::ptr::null(), |d| d.device)
}

/// GET DEVICE QUALIFIER DESCRIPTOR.
#[unsafe(no_mangle)]
pub extern "C" fn tud_descriptor_device_qualifier_cb() -> *const u8 {
    descriptors().map_or(core::ptr::null(), |d| d.qualifier)
}

/// GET CONFIGURATION DESCRIPTOR - the one that matches the negotiated speed.
#[unsafe(no_mangle)]
pub extern "C" fn tud_descriptor_configuration_cb(_index: u8) -> *const u8 {
    let Some(d) = descriptors() else {
        return core::ptr::null();
    };
    if unsafe { tud_speed_get() } == TUSB_SPEED_HIGH {
        d.hs_config
    } else {
        d.fs_config
    }
}

/// GET OTHER SPEED CONFIGURATION DESCRIPTOR - the *other* one.
#[unsafe(no_mangle)]
pub extern "C" fn tud_descriptor_other_speed_configuration_cb(_index: u8) -> *const u8 {
    let Some(d) = descriptors() else {
        return core::ptr::null();
    };
    if unsafe { tud_speed_get() } == TUSB_SPEED_HIGH {
        d.fs_config
    } else {
        d.hs_config
    }
}

/// GET STRING DESCRIPTOR.
///
/// Same serialisation as the C++ (`lnUsbStack::getDeviceDescriptor(int)`): a UTF-16LE
/// string capped at 64 code units *including* the length word, index 0 being the
/// two-byte language id.
#[unsafe(no_mangle)]
pub extern "C" fn tud_descriptor_string_cb(index: u8, _langid: u16) -> *const u16 {
    const MAX_STRING: usize = 64;
    const TUSB_DESC_STRING: u16 = 0x03;
    static DESC_STR: StringBuf = StringBuf(core::cell::UnsafeCell::new([0; MAX_STRING]));

    let Some(d) = descriptors() else {
        return core::ptr::null();
    };
    if d.strings.is_null() || index as usize >= d.string_count {
        return core::ptr::null();
    }
    // SAFETY: tinyUSB calls this from the `usbd` task only, one request at a time.
    let buf = DESC_STR.0.get() as *mut u16;

    unsafe {
        if index == 0 {
            let lang = *d.strings as *const u8;
            if lang.is_null() {
                return core::ptr::null();
            }
            *buf = (TUSB_DESC_STRING << 8) | 4;
            // descriptor[0] is the two-byte language id ({0x09, 0x04} = 0x0409 LE),
            // copied exactly as the C++ memcpy'd it.
            *buf.add(1) = u16::from_le_bytes([*lang, *lang.add(1)]);
            return buf;
        }
        let mut str = *d.strings.add(index as usize);
        if str.is_null() {
            return core::ptr::null();
        }
        // The string length, capped the way the C++ `MAX_STRING - 1` capped it.
        let mut len = 0usize;
        while *str.add(len) != 0 {
            len += 1;
            if len >= MAX_STRING - 1 {
                break;
            }
        }
        *buf = (TUSB_DESC_STRING << 8) | (2 * len as u16 + 2);
        for i in 0..len {
            *buf.add(1 + i) = (*str as u8) as u16;
            str = str.add(1);
        }
        buf
    }
}

// --- C ABI for the board ---------------------------------------------------------

/// Hand the descriptors to the stack. Must run before [`rn_usb_stack_start`].
///
/// # Safety
/// Every pointer must stay valid for the life of the firmware: they come from the
/// board's `static` descriptor tables (`lnBMP_usb_descriptor*.h`), which is what
/// `bmp_usb.cpp` passes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rn_usb_stack_init(
    device: *const u8,
    qualifier: *const u8,
    fs_config: *const u8,
    hs_config: *const u8,
    strings: *const *const c_char,
    string_count: usize,
) {
    static STORAGE: DescStorage = DescStorage(core::cell::UnsafeCell::new(Descriptors {
        device: core::ptr::null(),
        qualifier: core::ptr::null(),
        fs_config: core::ptr::null(),
        hs_config: core::ptr::null(),
        strings: core::ptr::null(),
        string_count: 0,
    }));
    // SAFETY: written once, before the task starts; the task only ever reads it.
    unsafe {
        let d = STORAGE.0.get();
        (*d).device = device;
        (*d).qualifier = qualifier;
        (*d).fs_config = fs_config;
        (*d).hs_config = hs_config;
        (*d).strings = strings;
        (*d).string_count = string_count;
        DESCRIPTORS.store(d, Ordering::Release);
    }
}

/// Register the board's event handler - the C++ `lnUsbStack::setEventHandler`.
#[unsafe(no_mangle)]
pub extern "C" fn rn_usb_stack_set_event_handler(cookie: *mut c_void, cb: EventHandler) {
    EVENT_COOKIE.store(cookie, Ordering::Release);
    EVENT_HANDLER.store(
        cb.map_or(core::ptr::null_mut(), |f| f as *mut c_void),
        Ordering::Release,
    );
}

/// Spawn the `usbd` task. The first call wins; later ones are ignored so a double init
/// cannot create two tasks on one static stack.
///
/// The one-shot flag is test-and-set under a critical section rather than an
/// `AtomicBool::swap`: RP2040 is ARMv6-M, which has no compare-and-swap, and this
/// crate is compiled for it too (usb.md P7).
#[unsafe(no_mangle)]
pub extern "C" fn rn_usb_stack_start() {
    {
        let _section = rs_rtos::critical::CriticalSection::enter();
        if STARTED.load(Ordering::Acquire) {
            return;
        }
        STARTED.store(true, Ordering::Release);
    }
    RUNNING.store(true, Ordering::Release);
    let _handle: TaskHandle = USB_TASK.spawn(
        "usbd",
        USB_TASK_PRIORITY,
        usb_task as CTaskEntry,
        core::ptr::null_mut(),
    );
}

/// Ask the task to leave its loop; it then deletes itself (as the C++ did).
#[unsafe(no_mangle)]
pub extern "C" fn rn_usb_stack_stop() {
    RUNNING.store(false, Ordering::Release);
}

#[unsafe(no_mangle)]
pub extern "C" fn rn_usb_is_mounted() -> bool {
    unsafe { tud_mounted() }
}

#[unsafe(no_mangle)]
pub extern "C" fn rn_usb_is_suspended() -> bool {
    unsafe { tud_suspended() }
}

#[unsafe(no_mangle)]
pub extern "C" fn rn_usb_is_ready() -> bool {
    tud_ready()
}

/// Last event posted by the callbacks (the C++ handed it to a registered handler; the
/// value is kept too so a reader that missed the callback can catch up).
#[unsafe(no_mangle)]
pub extern "C" fn rn_usb_last_event() -> i32 {
    LAST_EVENT.load(Ordering::Acquire)
}
