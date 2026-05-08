#![no_std]
//use rnarduino::rn_os_helper;
use rust_esprit::delay_ms;
use rust_esprit::{GpioMode::lnINPUT_PULLUP, digital_write, pin_mode};
use rust_esprit::{lnLogger, lnLogger_init, pin};

lnLogger_init!();
#[cfg(any(feature = "rp2040"))]
const PIN: pin = pin::GPIO10;
#[cfg(not(any(feature = "rp2040")))]
const PIN: pin = pin::PA4;

struct MyStruct<T> {
    context: T,
}
impl<T> rust_esprit::PinCallback for MyStruct<T> {
    fn on_interrupt(&mut self, pin: rust_esprit::pin) {
        panic!("on_interrupt\n");
    }
}

/**
 *
 *
 */
#[unsafe(no_mangle)]
extern "C" fn user_init() {
    lnLogger!("Hello there !\n");

    pin_mode(PIN, lnINPUT_PULLUP);
    let s: MyStruct<bool> = MyStruct { context: true };
    rust_esprit::exti_attach_interrupt_typed(PIN, rust_esprit::pin_edge::LN_EDGE_BOTH, &s);
    let mut on: bool = false;
    for _i in 0..5 {
        digital_write(PIN, on);
        on = !on;
        delay_ms(1000);
    }

    lnLogger!("--end--\n");
}

// EOF
