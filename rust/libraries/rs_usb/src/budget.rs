//! The USB endpoint budget, checked at compile time (`usb.md` P4).
//!
//! A CDC-ACM interface consumes **two endpoint numbers**: one IN-only notification
//! endpoint and one number shared by the data IN/OUT pair. The descriptors prove it -
//! `swindle/private_include/lnBMP_usb_descriptor_3cdc.h` uses `0x81`, `0x02`+`0x82`,
//! `0x83`, `0x04`+`0x84`, `0x85`, `0x06`+`0x86` (EP1..EP6 for three CDCs) - so the
//! budget is `1 + 2 * CDC_COUNT` endpoint numbers, EP0 included.
//!
//! The controller's total is a platform property and comes from the MCU definition:
//! `rs_bluepill::USB_MAX_ENDPOINT` (8: GD32F303 "up to 8 configurable bidirectional
//! endpoints", CH32V307 "16 unidirectional or 8 bidirectional") and 16 on RP2xxx.
//! Asking for more CDC interfaces than that fails the build, which is the whole point:
//! there is no way to discover this at run time.

/// Endpoint numbers one CDC-ACM interface needs.
pub const EP_PER_CDC: usize = 2;
/// EP0.
pub const CONTROL_ENDPOINTS: usize = 1;

/// Endpoint numbers `cdc_count` interfaces need, EP0 included.
pub const fn endpoints_for(cdc_count: usize) -> usize {
    CONTROL_ENDPOINTS + cdc_count * EP_PER_CDC
}

/// The most CDC interfaces a controller with `max_endpoints` numbers can host.
pub const fn max_cdc(max_endpoints: usize) -> usize {
    (max_endpoints - CONTROL_ENDPOINTS) / EP_PER_CDC
}

/// Compile-time check - call it from a `const _: () = ...;` so an over-subscribed
/// configuration stops the build with the message below.
pub const fn check(cdc_count: usize, max_endpoints: usize) {
    assert!(
        endpoints_for(cdc_count) <= max_endpoints,
        "too many CDC interfaces for this MCU's endpoint count"
    );
}
