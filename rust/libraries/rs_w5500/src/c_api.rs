//! C-ABI exports for initialization of W5500 driver from C++ boot code.

use crate::coordinator::{
    w5500_coordinator_task, COORDINATOR, DEVICE,
};
use crate::device::{W5500Device, W5500Pins};
use core::ffi::c_void;
use core::mem::MaybeUninit;
use rust_esprit::{GpioPin, Pin};

#[repr(C)]
pub struct W5500PinsC {
    pub miso: i32,
    pub mosi: i32,
    pub clk: i32,
    pub cs: i32,
    pub reset: i32,
    pub intr: i32,
}

static mut COORDINATOR_STACK: [u32; 1024] = [0; 1024];
static mut COORDINATOR_TCB: MaybeUninit<rs_rtos::c_freertos::StaticTask_t> =
    MaybeUninit::uninit();

unsafe extern "C" fn coordinator_entry(param: *mut c_void) {
    w5500_coordinator_task(param);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn w5500_init_c(spi_instance: i32, pins_ptr: *const W5500PinsC) -> bool {
    if pins_ptr.is_null() {
        return false;
    }
    let p = unsafe { &*pins_ptr };

    let pins = W5500Pins {
        spi_instance: spi_instance as u32,
        miso: GpioPin::new(unsafe { core::mem::transmute(p.miso) }),
        mosi: GpioPin::new(unsafe { core::mem::transmute(p.mosi) }),
        clk: GpioPin::new(unsafe { core::mem::transmute(p.clk) }),
        cs: GpioPin::new(unsafe { core::mem::transmute(p.cs) }),
        reset: GpioPin::new(unsafe { core::mem::transmute(p.reset) }),
        intr: GpioPin::new(unsafe { core::mem::transmute(p.intr) }),
    };

    let dev = W5500Device::new(pins);
    *DEVICE.lock() = Some(dev);

    true
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn w5500_set_mac_c(mac_ptr: *const u8) {
    if mac_ptr.is_null() {
        return;
    }
    let mut mac = [0u8; 6];
    unsafe {
        core::ptr::copy_nonoverlapping(mac_ptr, mac.as_mut_ptr(), 6);
    }

    let mut coord = COORDINATOR.lock();
    coord.mac = mac;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn w5500_start_c(cb: extern "C" fn(u32, *mut c_void), arg: *mut c_void) -> bool {
    {
        let mut coord = COORDINATOR.lock();
        coord.net_cb = Some(cb);
        coord.net_arg = arg;
    }

    // Launch coordinator FreeRTOS task
    let handle = unsafe {
        rs_rtos::c_freertos::xTaskCreateStatic(
            Some(coordinator_entry),
            b"w5500\0".as_ptr() as *const cty::c_char,
            1024,
            core::ptr::null_mut(),
            5,
            core::ptr::addr_of_mut!(COORDINATOR_STACK) as *mut u32,
            core::ptr::addr_of_mut!(COORDINATOR_TCB) as *mut rs_rtos::c_freertos::StaticTask_t,
        )
    };

    !handle.is_null()
}
