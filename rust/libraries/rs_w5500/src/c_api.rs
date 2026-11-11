//! C ABI seam: the board setup hands the pin map, MAC and network callback to
//! the Rust driver, and gets the coordinator task in return.

use crate::coordinator::NET;
use crate::coordinator::{w5500_coordinator_task, COORDINATOR};
use crate::device::{W5500Device, W5500Pins};
use core::ffi::c_void;
use rs_rtos::task::TaskStorage;

/// Network state callback, i.e. the C++ `lnLwIpSysCallback`.
pub type NetCallback = extern "C" fn(event: u32, arg: *mut c_void);

/// Coordinator task stack, in words (4 KiB), and its priority.
const COORDINATOR_STACK_WORDS: usize = 1024;
const COORDINATOR_PRIORITY: u32 = 5;

/// Stack and control block of the coordinator task, in `.bss`: the task takes
/// nothing from the FreeRTOS heap.
static COORDINATOR_STORAGE: TaskStorage<COORDINATOR_STACK_WORDS> = TaskStorage::new();

/// Initialise the W5500 and start the coordinator task that drives the PHY,
/// the DHCP client and the socket interrupts.
///
/// `pins` is a C++ `lnW5500SPI`, `mac` six bytes. Returns `false` if a pointer is
/// null; a task that cannot be created is fatal (it aborts, like any other
/// out-of-resources failure at boot).
///
/// # Safety
/// `pins` must point to an `lnW5500SPI` and `mac` to six readable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn w5500_begin_c(
    spi_instance: u32,
    pins: *const W5500Pins,
    mac: *const u8,
    cb: Option<NetCallback>,
    arg: *mut c_void,
) -> bool {
    if pins.is_null() || mac.is_null() {
        return false;
    }

    let mut mac_bytes = [0u8; 6];
    // SAFETY: `mac` points to `mac_bytes.len()` readable bytes (checked above).
    unsafe { core::ptr::copy_nonoverlapping(mac, mac_bytes.as_mut_ptr(), mac_bytes.len()) };

    // SAFETY: `pins` points to an `lnW5500SPI`, which has the layout of `W5500Pins`.
    let device = W5500Device::new(spi_instance, unsafe { *pins });
    NET.lock().device = Some(device);

    COORDINATOR.lock().configure(mac_bytes, cb, arg);

    COORDINATOR_STORAGE.spawn(
        "w5500",
        COORDINATOR_PRIORITY,
        w5500_coordinator_task,
        core::ptr::null_mut(),
    );

    true
}

