//! Bluepill (GD32F3x / CH32V3x) `main()`, initial FreeRTOS heap, and peripheral startup.
//!
//! Replaces `common_bluepill/lnSystemInit.cpp`.

use core::ffi::c_int;

use rs_rcu_bluepill::Peripheral;
use rs_rtos::c_freertos as freertos;

use crate::platform::mcu;
use crate::runtime::{create_task_static, entry_task};

const CONFIG_TOTAL_HEAP_SIZE: usize = mcu::runtime::CONFIG_TOTAL_HEAP_SIZE;
const ENTRY_TASK_STACK_BYTES: u32 = 1024;
const ENTRY_TASK_PRIORITY: freertos::UBaseType_t = 2;
const ENTRY_TASK_STACK_WORDS: usize = (ENTRY_TASK_STACK_BYTES as usize / 4) + 32;

#[used]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".initial_heap")]
pub static mut ucHeap: [u8; CONFIG_TOTAL_HEAP_SIZE] = [0; CONFIG_TOTAL_HEAP_SIZE];

static mut ENTRY_TASK_TCB: freertos::StaticTask_t = unsafe { core::mem::zeroed() };
static mut ENTRY_TASK_STACK: [freertos::StackType_t; ENTRY_TASK_STACK_WORDS] =
    [0; ENTRY_TASK_STACK_WORDS];

#[inline]
fn reset_and_enable(p: Peripheral) {
    rs_rcu_bluepill::reset(p);
    rs_rcu_bluepill::enable(p);
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> c_int {
    mcu::runtime::early_init();

    for p in [
        Peripheral::GpioA,
        Peripheral::GpioB,
        Peripheral::GpioC,
        Peripheral::Afio,
        Peripheral::Dma0,
        Peripheral::Dma1,
    ] {
        reset_and_enable(p);
    }

    for p in [Peripheral::I2c0, Peripheral::I2c1] {
        rs_rcu_bluepill::disable(p);
    }

    rs_exti_bluepill::sys_init();

    for p in [
        Peripheral::Spi0,
        Peripheral::Spi1,
        Peripheral::Spi2,
        Peripheral::Timer1,
        Peripheral::Timer2,
        Peripheral::Timer3,
        Peripheral::Timer4,
        Peripheral::Timer5,
        Peripheral::Timer6,
        Peripheral::Dac,
    ] {
        rs_rcu_bluepill::disable(p);
    }

    reset_and_enable(Peripheral::Adc0);
    reset_and_enable(Peripheral::Adc1);

    mcu::runtime::post_peripheral_init();

    // SAFETY: Static TCB and stack live in `.bss` and are only borrowed here once before the scheduler starts.
    unsafe {
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
    mcu::deadEnd(25);
}
