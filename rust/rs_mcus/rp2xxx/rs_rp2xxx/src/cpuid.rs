//! Shared `lnCpuID::*` C ABI helpers for RP2040 / RP2350.

const LN_VENDOR_RP: i32 = 5;
const RP_FLASH_SIZE_KB: i32 = 2048;

#[repr(C)]
struct PicoUniqueBoardId {
    id: [u8; 8],
}

unsafe extern "C" {
    fn pico_get_unique_board_id(id_out: *mut PicoUniqueBoardId);
}

#[unsafe(export_name = "_ZN7lnCpuID8identifyEv")]
pub extern "C" fn identify() {}

#[unsafe(export_name = "_ZN7lnCpuID6vendorEv")]
pub extern "C" fn vendor() -> i32 {
    LN_VENDOR_RP
}

#[unsafe(export_name = "_ZN7lnCpuID9flashSizeEv")]
pub extern "C" fn flash_size() -> i32 {
    RP_FLASH_SIZE_KB
}

#[unsafe(export_name = "_ZN7lnCpuID11getSerialIDEv")]
pub extern "C" fn get_serial_id() -> u32 {
    let mut id = PicoUniqueBoardId { id: [0; 8] };
    // SAFETY: `pico_get_unique_board_id` fills the 8-byte `id` buffer.
    unsafe { pico_get_unique_board_id(&mut id) };
    u32::from_le_bytes([id.id[0], id.id[1], id.id[2], id.id[3]])
}
