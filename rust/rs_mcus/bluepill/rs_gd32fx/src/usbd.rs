#![allow(dead_code)]
//! GD32F303 USBD peripheral & TinyUSB DCD driver.
//!
//! Replaces `esprit/cpp/mcus/arm_gd32fx/src/lnUSBD.cpp`,
//! `esprit/cpp/mcus/arm_gd32fx/tinyUsb/lnTinyUsbImpl.cpp`, and
//! `esprit/cpp/mcus/arm_gd32fx/tinyUsb/lnUsbLL.cpp` when `LN_ENABLE_RUST` and
//! `LN_RUST_USB` are enabled on `arm_gd32fx`.

use core::cell::UnsafeCell;
use core::ffi::c_void;
use core::mem::{offset_of, size_of};
use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicI32, Ordering};

use rs_gpio_bluepill::{Mode, Pin};
use rs_rcu_bluepill::Peripheral;

// ---------------------------------------------------------------------------
// Hardware registers & constants (GD32F303 USBD, 0x4000_5C00 / PMA 0x4000_6000)
// ---------------------------------------------------------------------------

const USBD0_BASE: usize = 0x4000_5C00;
const USBD0_SRAM_BASE: usize = 0x4000_6000;
const USBD_MAX_ENDPOINT: usize = 8;
const PMA_SIZE_BYTES: usize = 512;
const CFG_TUD_ENDPOINT0_SIZE: u16 = 64;

const LN_IRQ_USB_HP_CAN_TX: i32 = 19;
const LN_IRQ_USB_LP_CAN_RX0: i32 = 20;
const LN_IRQ_USBWAKEUP: i32 = 42;

#[repr(C)]
struct UsbdRegisters {
    epcs: [u32; 8],        // 0x00..0x1C
    reserved_20: [u32; 8], // 0x20..0x3C
    ctl: u32,              // 0x40
    intf: u32,             // 0x44
    stat: u32,             // 0x48
    daddr: u32,            // 0x4C
    baddr: u32,            // 0x50
}

const _: () = assert!(offset_of!(UsbdRegisters, ctl) == 0x40);
const _: () = assert!(offset_of!(UsbdRegisters, baddr) == 0x50);

#[repr(C)]
struct BTableEntry {
    tx_adr: u32,
    tx_size: u32,
    rx_adr: u32,
    rx_size: u32,
}

const _: () = assert!(size_of::<BTableEntry>() == 16);

// USBD_CTL bits
const USBD_CTL_SETRST: u32 = 1 << 0;
const USBD_CTL_CLOSE: u32 = 1 << 1;
const USBD_CTL_LOWM: u32 = 1 << 2;
const USBD_CTL_SETSPS: u32 = 1 << 3;
const USBD_CTL_RSREQ: u32 = 1 << 4;
const USBD_CTL_ESOFIE: u32 = 1 << 8;
const USBD_CTL_SOFIE: u32 = 1 << 9;
const USBD_CTL_RSTIE: u32 = 1 << 10;
const USBD_CTL_SPSIE: u32 = 1 << 11;
const USBD_CTL_WKUPIE: u32 = 1 << 12;
const USBD_CTL_ERRIE: u32 = 1 << 13;
const USBD_CTL_PMOUIE: u32 = 1 << 14;
const USBD_CTL_STIE: u32 = 1 << 15;

// USBD_INTF bits
const USBD_INTF_EPNUM_MASK: u32 = 0x0f;
const USBD_INTF_DIR: u32 = 1 << 4;
const USBD_INTF_ESOFIF: u32 = 1 << 8;
const USBD_INTF_SOFIF: u32 = 1 << 9;
const USBD_INTF_RSTIF: u32 = 1 << 10;
const USBD_INTF_SPSIF: u32 = 1 << 11;
const USBD_INTF_WKUPIF: u32 = 1 << 12;
const USBD_INTF_ERRIF: u32 = 1 << 13;
const USBD_INTF_PMOUIF: u32 = 1 << 14;
const USBD_INTF_STIF: u32 = 1 << 15;

// USBD_EPxCS bits
const USBD_EPXCS_TX_DTG: u32 = 1 << 6;
const USBD_EPXCS_TX_ST: u32 = 1 << 7;
const USBD_EPXCS_EP_KCTL: u32 = 1 << 8;
const USBD_EPXCS_EPCTL_MASK: u32 = 3 << 9;
const USBD_EPXCS_EPCTL_CONTROL: u32 = 1 << 9;
const USBD_EPXCS_EPCTL_INTERRUPT: u32 = 3 << 9;
const USBD_EPXCS_SETUP: u32 = 1 << 11;
const USBD_EPXCS_RX_DTG: u32 = 1 << 14;
const USBD_EPXCS_RX_ST: u32 = 1 << 15;

const USBD_EPXCS_RX_STA_MASK: u32 = 3 << 12;
const USBD_EPXCS_RX_STA_DISABLED: u32 = 0 << 12;
const USBD_EPXCS_RX_STA_STALL: u32 = 1 << 12;
const USBD_EPXCS_RX_STA_NAK: u32 = 2 << 12;
const USBD_EPXCS_RX_STA_VALID: u32 = 3 << 12;

const USBD_EPXCS_TX_STA_MASK: u32 = 3 << 4;
const USBD_EPXCS_TX_STA_DISABLED: u32 = 0 << 4;
const USBD_EPXCS_TX_STA_STALL: u32 = 1 << 4;
const USBD_EPXCS_TX_STA_NAK: u32 = 2 << 4;
const USBD_EPXCS_TX_STA_VALID: u32 = 3 << 4;

const MASK_CLEAR: u32 = 0xffff
    & !USBD_EPXCS_RX_DTG
    & !USBD_EPXCS_TX_DTG
    & !USBD_EPXCS_RX_STA_MASK
    & !USBD_EPXCS_TX_STA_MASK;

// ---------------------------------------------------------------------------
// TinyUSB C ABI types
// ---------------------------------------------------------------------------

const TUSB_DIR_OUT: u8 = 0;
const TUSB_DIR_IN: u8 = 1;

const TUSB_XFER_CONTROL: u8 = 0;
const TUSB_XFER_ISOCHRONOUS: u8 = 1;
const TUSB_XFER_BULK: u8 = 2;
const TUSB_XFER_INTERRUPT: u8 = 3;

const TUSB_REQ_TYPE_STANDARD: u8 = 0;
const TUSB_REQ_RCPT_DEVICE: u8 = 0;
const TUSB_REQ_SET_ADDRESS: u8 = 5;

const XFER_RESULT_SUCCESS: u8 = 0;

const DCD_EVENT_BUS_RESET: u8 = 1;
const DCD_EVENT_SUSPEND: u8 = 4;
const DCD_EVENT_RESUME: u8 = 5;
const DCD_EVENT_SETUP_RECEIVED: u8 = 6;
const DCD_EVENT_XFER_COMPLETE: u8 = 7;

#[repr(C, align(4))]
#[derive(Copy, Clone)]
struct DcdEvent {
    rhport: u8,
    event_id: u8,
    _pad: [u8; 2],
    payload: [u32; 2],
}

const _: () = assert!(size_of::<DcdEvent>() == 12);

#[repr(C, packed)]
pub struct TusbDescEndpoint {
    b_length: u8,
    b_descriptor_type: u8,
    b_endpoint_address: u8,
    bm_attributes: u8,
    w_max_packet_size: u16,
    b_interval: u8,
}

#[repr(C, packed)]
pub struct TusbControlRequest {
    bm_request_type: u8,
    b_request: u8,
    w_value: u16,
    w_index: u16,
    w_length: u16,
}

static EP0_OUT_DESC: TusbDescEndpoint = TusbDescEndpoint {
    b_length: 7,
    b_descriptor_type: 5,
    b_endpoint_address: 0x00,
    bm_attributes: TUSB_XFER_CONTROL,
    w_max_packet_size: CFG_TUD_ENDPOINT0_SIZE,
    b_interval: 0,
};

static EP0_IN_DESC: TusbDescEndpoint = TusbDescEndpoint {
    b_length: 7,
    b_descriptor_type: 5,
    b_endpoint_address: 0x80,
    bm_attributes: TUSB_XFER_CONTROL,
    w_max_packet_size: CFG_TUD_ENDPOINT0_SIZE,
    b_interval: 0,
};

use crate::irq::deadEnd;

unsafe extern "C" {
    fn dcd_event_handler(event: *const DcdEvent, in_isr: bool);
    fn lnDelay_C(ms: u32);
}

// ---------------------------------------------------------------------------
// Register & PMA helpers
// ---------------------------------------------------------------------------

#[inline(always)]
fn usbd() -> *mut UsbdRegisters {
    USBD0_BASE as *mut UsbdRegisters
}

#[inline(always)]
fn btable(ep: usize) -> *mut BTableEntry {
    // SAFETY: `USBD0_SRAM_BASE` holds the 8-row BTable at offset 0.
    unsafe { (USBD0_SRAM_BASE as *mut BTableEntry).add(ep & 7) }
}

#[inline(always)]
fn dcd_event_bus_signal(rhport: u8, eid: u8, in_isr: bool) {
    let ev = DcdEvent {
        rhport,
        event_id: eid,
        _pad: [0; 2],
        payload: [0; 2],
    };
    unsafe { dcd_event_handler(&raw const ev, in_isr) };
}

#[inline(always)]
fn dcd_event_bus_reset(rhport: u8, in_isr: bool) {
    // TUSB_SPEED_FULL == 0
    let ev = DcdEvent {
        rhport,
        event_id: DCD_EVENT_BUS_RESET,
        _pad: [0; 2],
        payload: [0; 2],
    };
    unsafe { dcd_event_handler(&raw const ev, in_isr) };
}

#[inline(always)]
fn dcd_event_setup_received(rhport: u8, setup: &[u8; 8], in_isr: bool) {
    let mut payload = [0u32; 2];
    // SAFETY: `setup` is 8 bytes, `payload` is 8 bytes.
    unsafe {
        core::ptr::copy_nonoverlapping(setup.as_ptr(), payload.as_mut_ptr() as *mut u8, 8);
    }
    let ev = DcdEvent {
        rhport,
        event_id: DCD_EVENT_SETUP_RECEIVED,
        _pad: [0; 2],
        payload,
    };
    unsafe { dcd_event_handler(&raw const ev, in_isr) };
}

#[inline(always)]
fn dcd_event_xfer_complete(rhport: u8, ep_addr: u8, len: u32, result: u8, in_isr: bool) {
    let word0 = (ep_addr as u32) | ((result as u32) << 8);
    let ev = DcdEvent {
        rhport,
        event_id: DCD_EVENT_XFER_COMPLETE,
        _pad: [0; 2],
        payload: [word0, len],
    };
    unsafe { dcd_event_handler(&raw const ev, in_isr) };
}

// ---------------------------------------------------------------------------
// Endpoint PMA & transfer state
// ---------------------------------------------------------------------------

#[derive(Copy, Clone)]
struct XferDescriptor {
    buffer: *mut u8,
    total_len: u16,
    queued_len: u16,
    pma_ptr: u16,
    max_packet_size: u16,
    pma_alloc_size: u16,
}

impl XferDescriptor {
    const fn zero() -> Self {
        Self {
            buffer: core::ptr::null_mut(),
            total_len: 0,
            queued_len: 0,
            pma_ptr: 0,
            max_packet_size: 0,
            pma_alloc_size: 0,
        }
    }
}

struct UsbdState {
    xfer_status: [XferDescriptor; USBD_MAX_ENDPOINT * 2],
    ep_buffer_tail: usize,
    ep_nb_ep: usize,
    setup_packet: [u32; 6],
}

impl UsbdState {
    const fn new() -> Self {
        Self {
            xfer_status: [XferDescriptor::zero(); USBD_MAX_ENDPOINT * 2],
            ep_buffer_tail: btable_size(),
            ep_nb_ep: 2,
            setup_packet: [0; 6],
        }
    }
}

const fn btable_size() -> usize {
    let cdc_interfaces = if cfg!(feature = "cdc3") { 3 } else { 2 };
    (2 * cdc_interfaces + 1) * size_of::<BTableEntry>()
}

struct SyncUsbdState(UnsafeCell<UsbdState>);
// SAFETY: Accessed only from USBD ISR (when IRQ enabled) or TinyUSB task with IRQ disabled.
unsafe impl Sync for SyncUsbdState {}

static STATE: SyncUsbdState = SyncUsbdState(UnsafeCell::new(UsbdState::new()));
static REMOTE_WAKE_COUNTDOWN: AtomicI32 = AtomicI32::new(4);

#[inline(always)]
unsafe fn state_mut() -> &'static mut UsbdState {
    unsafe { &mut *STATE.0.get() }
}

fn reset_endpoints(partial: bool) {
    // SAFETY: Caller serializes access (bus reset or endpoint close).
    let st = unsafe { state_mut() };
    if !partial {
        st.ep_nb_ep = 2;
    }
    st.ep_buffer_tail = btable_size();
    let start = if partial { 2 } else { 0 };
    for desc in &mut st.xfer_status[start..] {
        *desc = XferDescriptor::zero();
    }
}

fn reset_all_eps() {
    for i in 0..USBD_MAX_ENDPOINT {
        // SAFETY: `USBD0_BASE` epcs[0..8] are valid 16-bit-in-32-bit registers.
        unsafe { write_volatile(&raw mut (*usbd()).epcs[i], 0) };
    }
    reset_endpoints(false);
}

fn set_rx_buffer_size(ep: usize, size: u32) {
    let out = if size > 62 {
        let nb_block = (size + 31) / 32;
        (1 << 15) + (nb_block << 10)
    } else {
        let nb_block = (size + 1) / 2;
        nb_block << 10
    };
    // SAFETY: `btable(ep)` is in USBD PMA SRAM.
    unsafe { write_volatile(&raw mut (*btable(ep)).rx_size, out) };
}

fn init_ep_ram(address: u8, size: u16) -> u16 {
    let epnum = (address & 0x7f) as usize;
    let dir = usize::from((address & 0x80) != 0);
    // SAFETY: `epnum < 8` and `dir < 2`.
    let st = unsafe { state_mut() };
    let desc = &mut st.xfer_status[2 * epnum + dir];
    if desc.pma_alloc_size != 0 {
        if size > (desc.pma_alloc_size & 0x1ff) {
            deadEnd(0xA000);
        }
        return desc.pma_ptr;
    }
    let ret = st.ep_buffer_tail;
    st.ep_buffer_tail += size as usize;
    if st.ep_buffer_tail >= PMA_SIZE_BYTES {
        deadEnd(0xA000);
    }
    desc.pma_ptr = ret as u16;
    desc.pma_alloc_size = size;
    st.ep_nb_ep += 1;
    ret as u16
}

fn copy_from_sram(dest: *mut u8, src_offset: usize, bytes: usize) {
    if (src_offset & 1) != 0 {
        deadEnd(0xA000);
    }
    // SAFETY: PMA SRAM is 16-bit data every 32-bit word (`ram += 2` in u16 units).
    unsafe {
        let mut ram = (USBD0_SRAM_BASE as *const u16).add(src_offset);
        let nb_word = bytes / 2;
        for i in 0..nb_word {
            let temp = read_volatile(ram);
            ram = ram.add(2);
            *dest.add(2 * i) = (temp & 0xff) as u8;
            *dest.add(2 * i + 1) = (temp >> 8) as u8;
        }
        if (bytes & 1) != 0 {
            let temp = read_volatile(ram);
            *dest.add(bytes - 1) = (temp & 0xff) as u8;
        }
    }
}

fn copy_to_sram(dest_offset: usize, src: *const u8, bytes: usize) {
    if (dest_offset & 1) != 0 {
        deadEnd(0xA000);
    }
    let nb_words = (bytes + 1) / 2;
    // SAFETY: PMA SRAM is 16-bit data every 32-bit word (`ram += 2` in u16 units).
    unsafe {
        let mut ram = (USBD0_SRAM_BASE as *mut u16).add(dest_offset);
        for i in 0..nb_words {
            let b0 = *src.add(2 * i) as u16;
            let b1 = if 2 * i + 1 < bytes {
                *src.add(2 * i + 1) as u16
            } else {
                0
            };
            write_volatile(ram, b0 | (b1 << 8));
            ram = ram.add(2);
        }
    }
}

// ---------------------------------------------------------------------------
// Endpoint register manipulation
// ---------------------------------------------------------------------------

#[inline(always)]
fn read_epcs(ep: usize) -> u32 {
    unsafe { read_volatile(&raw const (*usbd()).epcs[ep & 7]) & 0xffff }
}

#[inline(always)]
fn write_epcs(ep: usize, val: u32) {
    unsafe { write_volatile(&raw mut (*usbd()).epcs[ep & 7], val & 0xffff) };
}

fn clear_tx_rx(ep: usize, is_tx: bool) {
    let mut reg = read_epcs(ep) & MASK_CLEAR;
    if is_tx {
        reg &= !USBD_EPXCS_TX_ST;
        reg |= USBD_EPXCS_RX_ST;
    } else {
        reg &= !USBD_EPXCS_RX_ST;
        reg |= USBD_EPXCS_TX_ST;
    }
    write_epcs(ep, reg);
}

fn set_ep_kind(ep: usize, set: bool) {
    let mut reg = read_epcs(ep) & MASK_CLEAR;
    if set {
        reg |= USBD_EPXCS_EP_KCTL;
    } else {
        reg &= !USBD_EPXCS_EP_KCTL;
    }
    reg |= USBD_EPXCS_RX_ST | USBD_EPXCS_TX_ST;
    write_epcs(ep, reg);
}

fn set_ep_address(ep: usize, adr: u32) {
    let mut reg = (read_epcs(ep) & MASK_CLEAR) | adr;
    reg |= USBD_EPXCS_RX_ST | USBD_EPXCS_TX_ST;
    write_epcs(ep, reg);
}

fn set_ep_status(ep: usize, is_tx: bool, status: u32) {
    let mut reg = read_epcs(ep);
    let c = if is_tx {
        reg &= MASK_CLEAR | USBD_EPXCS_TX_STA_MASK;
        USBD_EPXCS_TX_STA_MASK & status
    } else {
        reg &= MASK_CLEAR | USBD_EPXCS_RX_STA_MASK;
        USBD_EPXCS_RX_STA_MASK & status
    };
    reg ^= c;
    reg |= USBD_EPXCS_RX_ST | USBD_EPXCS_TX_ST;
    write_epcs(ep, reg);
}

fn clear_dtg(ep: usize, is_tx: bool) {
    let reg = read_epcs(ep);
    let change = if is_tx {
        if (reg & USBD_EPXCS_TX_DTG) != 0 {
            USBD_EPXCS_TX_DTG
        } else {
            0
        }
    } else if (reg & USBD_EPXCS_RX_DTG) != 0 {
        USBD_EPXCS_RX_DTG
    } else {
        0
    };
    if change != 0 {
        let next = (reg & MASK_CLEAR) | USBD_EPXCS_RX_ST | USBD_EPXCS_TX_ST | change;
        write_epcs(ep, next);
    }
}

fn set_ep_type(ep: usize, ep_type: u32) {
    let mut reg = read_epcs(ep) & (MASK_CLEAR & !USBD_EPXCS_EPCTL_MASK);
    reg |= ep_type | USBD_EPXCS_RX_ST | USBD_EPXCS_TX_ST;
    write_epcs(ep, reg);
}

fn set_address(address: i32) {
    // SAFETY: `USBD0_BASE` daddr is valid.
    unsafe {
        write_volatile(&raw mut (*usbd()).daddr, 0);
        if address >= 0 {
            write_volatile(&raw mut (*usbd()).daddr, 0x80 + ((address as u32) & 0x7f));
        }
    }
}

// ---------------------------------------------------------------------------
// DCD event / transfer handlers
// ---------------------------------------------------------------------------

fn dcd_transmit_packet(ep_ix: usize) {
    // SAFETY: Single endpoint access serialized by USBD IRQ/task lock.
    let st = unsafe { state_mut() };
    let xfer = &mut st.xfer_status[2 * ep_ix + TUSB_DIR_IN as usize];
    let mut len = xfer.total_len - xfer.queued_len;
    if len > xfer.max_packet_size {
        len = xfer.max_packet_size;
    }
    let old_addr = unsafe { read_volatile(&raw const (*btable(ep_ix)).tx_adr) } as usize;
    if len > 0 && !xfer.buffer.is_null() {
        copy_to_sram(old_addr, unsafe { xfer.buffer.add(xfer.queued_len as usize) }, len as usize);
    }
    xfer.queued_len = xfer.queued_len.wrapping_add(len);
    unsafe { write_volatile(&raw mut (*btable(ep_ix)).tx_size, len as u32) };
    set_ep_status(ep_ix, true, USBD_EPXCS_TX_STA_VALID);
}

fn dcd_handle_bus_reset() {
    set_address(-1);
    reset_all_eps();
    dcd_edpt_open(0, &raw const EP0_OUT_DESC);
    dcd_edpt_open(0, &raw const EP0_IN_DESC);
    set_address(0);
}

fn dcd_ep_ctr_tx_handler(ep: usize) {
    clear_tx_rx(ep, true);
    let st = unsafe { state_mut() };
    let xfer = &st.xfer_status[2 * ep + TUSB_DIR_IN as usize];
    if xfer.total_len != xfer.queued_len {
        dcd_transmit_packet(ep);
    } else {
        dcd_event_xfer_complete(0, 0x80 + ep as u8, xfer.total_len as u32, XFER_RESULT_SUCCESS, true);
    }
}

fn dcd_ep_ctr_rx_handler(ep: usize) {
    let ep_reg = read_epcs(ep);
    let count = unsafe { read_volatile(&raw const (*btable(ep)).rx_size) } & 0x3ff;
    let st = unsafe { state_mut() };
    let xfer = &mut st.xfer_status[2 * ep + TUSB_DIR_OUT as usize];

    if ep == 0 && (ep_reg & USBD_EPXCS_SETUP) != 0 {
        if count == 8 {
            set_ep_status(0, false, USBD_EPXCS_RX_STA_NAK);
            set_ep_status(0, true, USBD_EPXCS_TX_STA_NAK);
            let mut setup_buf = [0u8; 8];
            let rx_adr = unsafe { read_volatile(&raw const (*btable(ep)).rx_adr) } as usize;
            copy_from_sram(setup_buf.as_mut_ptr(), rx_adr, 8);
            dcd_event_setup_received(0, &setup_buf, true);
        }
    } else {
        if ep != 0 {
            clear_tx_rx(ep, false);
        }
        if count != 0 {
            let rx_adr = unsafe { read_volatile(&raw const (*btable(ep)).rx_adr) } as usize;
            copy_from_sram(
                unsafe { xfer.buffer.add(xfer.queued_len as usize) },
                rx_adr,
                count as usize,
            );
            xfer.queued_len = xfer.queued_len.wrapping_add(count as u16);
        }
        if (count as u16) < xfer.max_packet_size || xfer.queued_len == xfer.total_len {
            dcd_event_xfer_complete(0, ep as u8, xfer.queued_len as u32, XFER_RESULT_SUCCESS, true);
        } else {
            let remaining = (xfer.total_len - xfer.queued_len) as u32;
            let sz = remaining.min(xfer.max_packet_size as u32);
            set_rx_buffer_size(ep, sz);
            set_ep_status(ep, false, USBD_EPXCS_RX_STA_VALID);
        }
    }

    if ep == 0 {
        set_rx_buffer_size(0, CFG_TUD_ENDPOINT0_SIZE as u32);
        clear_tx_rx(0, false);
    }
}

// ---------------------------------------------------------------------------
// USBD IRQ handler
// ---------------------------------------------------------------------------

fn usb_irq() {
    // SAFETY: `USBD0_BASE` registers are valid.
    unsafe {
        let ctl = read_volatile(&raw const (*usbd()).ctl);
        let flags_original = read_volatile(&raw const (*usbd()).intf);
        let mut flags = flags_original & ctl & 0xff00;

        if (flags & USBD_INTF_RSTIF) != 0 {
            write_volatile(&raw mut (*usbd()).intf, flags_original & !USBD_INTF_RSTIF);
            dcd_handle_bus_reset();
            dcd_event_bus_reset(0, true);
            return;
        }

        if (flags & USBD_INTF_STIF) != 0 {
            let mut f = flags_original;
            flags &= !USBD_INTF_STIF;
            while (f & USBD_INTF_STIF) != 0 {
                let ep = (f & USBD_INTF_EPNUM_MASK) as usize;
                let reg = read_epcs(ep);
                if (f & USBD_INTF_DIR) != 0 {
                    if (reg & USBD_EPXCS_RX_ST) != 0 {
                        dcd_ep_ctr_rx_handler(ep);
                    }
                } else if (reg & USBD_EPXCS_TX_ST) != 0 {
                    dcd_ep_ctr_tx_handler(ep);
                }
                f = read_volatile(&raw const (*usbd()).intf);
            }
        }

        if (flags & USBD_INTF_ERRIF) != 0 {
            let cur = read_volatile(&raw const (*usbd()).intf);
            write_volatile(&raw mut (*usbd()).intf, cur & !USBD_INTF_ERRIF);
            flags &= !USBD_INTF_ERRIF;
        }

        if (flags & USBD_INTF_WKUPIF) != 0 {
            let c = read_volatile(&raw const (*usbd()).ctl);
            write_volatile(&raw mut (*usbd()).ctl, c & !(USBD_CTL_LOWM | USBD_CTL_SETSPS));
            let cur = read_volatile(&raw const (*usbd()).intf);
            write_volatile(&raw mut (*usbd()).intf, cur & !USBD_INTF_WKUPIF);
            flags &= !USBD_INTF_WKUPIF;
            dcd_event_bus_signal(0, DCD_EVENT_RESUME, true);
        }

        if (flags & USBD_INTF_SPSIF) != 0 {
            let c = read_volatile(&raw const (*usbd()).ctl);
            write_volatile(&raw mut (*usbd()).ctl, c | USBD_CTL_SETSPS | USBD_CTL_LOWM);
            let cur = read_volatile(&raw const (*usbd()).intf);
            write_volatile(&raw mut (*usbd()).intf, cur & !USBD_INTF_SPSIF);
            flags &= !USBD_INTF_SPSIF;
            dcd_event_bus_signal(0, DCD_EVENT_SUSPEND, true);
        }

        if (flags & USBD_INTF_SOFIF) != 0 {
            let cur = read_volatile(&raw const (*usbd()).intf);
            write_volatile(&raw mut (*usbd()).intf, cur & !USBD_INTF_SOFIF);
            let c = read_volatile(&raw const (*usbd()).ctl);
            write_volatile(&raw mut (*usbd()).ctl, c & !USBD_CTL_SOFIE);
            flags &= !USBD_INTF_SOFIF;
        }

        if (flags & USBD_INTF_ESOFIF) != 0 {
            let cnt = REMOTE_WAKE_COUNTDOWN.load(Ordering::Relaxed);
            if cnt == 1 {
                let c = read_volatile(&raw const (*usbd()).ctl);
                write_volatile(&raw mut (*usbd()).ctl, c & !USBD_CTL_RSREQ);
            }
            if cnt > 0 {
                REMOTE_WAKE_COUNTDOWN.store(cnt - 1, Ordering::Relaxed);
            }
            let cur = read_volatile(&raw const (*usbd()).intf);
            write_volatile(&raw mut (*usbd()).intf, cur & !USBD_INTF_ESOFIF);
            flags &= !USBD_INTF_ESOFIF;
        }

        if flags != 0 {
            deadEnd(0xA000);
        }
    }
}

#[unsafe(export_name = "_Z17USB_TX_IRQHandlerv")]
pub extern "C" fn usb_tx_irq_handler() {
    usb_irq();
}

#[unsafe(export_name = "_Z17USB_RX_IRQHandlerv")]
pub extern "C" fn usb_rx_irq_handler() {
    usb_irq();
}

#[unsafe(export_name = "_Z21USB_WAKEUP_IRQHandlerv")]
pub extern "C" fn usb_wakeup_irq_handler() {
    usb_irq();
}

// ---------------------------------------------------------------------------
// TinyUSB DCD C ABI
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub extern "C" fn lnUsbDcdInit() {
    dcd_int_disable(0);
    rs_rcu_bluepill::usb_stm_gd32::enable_usb_48mhz();
    reset_endpoints(false);
    Pin::PA12.set_mode(Mode::InputFloating, 0);
    Pin::PA11.set_mode(Mode::InputFloating, 0);
    rs_rcu_bluepill::enable(Peripheral::Usb);
    rs_rcu_bluepill::reset(Peripheral::Usb);
}

#[unsafe(no_mangle)]
pub extern "C" fn dcd_init(_rhport: u8, _rh_init: *const c_void) -> bool {
    // Pull D+ low for 10 ms to force host re-enumeration.
    Pin::PA12.write(false);
    Pin::PA12.set_mode(Mode::Output, 0);
    Pin::PA12.write(false);
    unsafe { lnDelay_C(10) };
    Pin::PA12.set_mode(Mode::InputFloating, 0);

    // SAFETY: `USBD0_BASE` registers are valid.
    unsafe {
        let ctl = read_volatile(&raw const (*usbd()).ctl);
        write_volatile(&raw mut (*usbd()).ctl, ctl & !(USBD_CTL_CLOSE | USBD_CTL_SETRST));
        let intf = read_volatile(&raw const (*usbd()).intf);
        write_volatile(&raw mut (*usbd()).intf, intf & USBD_INTF_RSTIF);
        write_volatile(&raw mut (*usbd()).baddr, 0);
        write_volatile(&raw mut (*usbd()).daddr, 0);
        write_volatile(&raw mut (*usbd()).stat, 0);
        let ctl2 = read_volatile(&raw const (*usbd()).ctl);
        write_volatile(
            &raw mut (*usbd()).ctl,
            ctl2 | USBD_CTL_RSTIE | USBD_CTL_PMOUIE | USBD_CTL_STIE,
        );
    }
    true
}

#[unsafe(no_mangle)]
pub extern "C" fn dcd_int_enable(_rhport: u8) {
    for irq in [LN_IRQ_USB_HP_CAN_TX, LN_IRQ_USB_LP_CAN_RX0, LN_IRQ_USBWAKEUP] {
        // SAFETY: Valid USB IRQ indices in `LN_VECTOR_TABLE`.
        unsafe { crate::irq::ln_enable_interrupt(&raw const irq) };
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn dcd_int_disable(_rhport: u8) {
    for irq in [LN_IRQ_USB_HP_CAN_TX, LN_IRQ_USB_LP_CAN_RX0, LN_IRQ_USBWAKEUP] {
        // SAFETY: Valid USB IRQ indices in `LN_VECTOR_TABLE`.
        unsafe { crate::irq::ln_disable_interrupt(&raw const irq) };
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn dcd_remote_wakeup(_rhport: u8) {
    unsafe {
        let c = read_volatile(&raw const (*usbd()).ctl);
        write_volatile(&raw mut (*usbd()).ctl, c | USBD_CTL_RSREQ);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn dcd_edpt_close_all(_rhport: u8) {}

#[unsafe(no_mangle)]
pub extern "C" fn dcd_set_address(rhport: u8, _dev_addr: u8) {
    dcd_edpt_xfer(rhport, 0x80, core::ptr::null_mut(), 0);
}

#[unsafe(no_mangle)]
pub extern "C" fn dcd_edpt0_status_complete(_rhport: u8, request: *const TusbControlRequest) {
    if request.is_null() {
        return;
    }
    // SAFETY: Caller passes a valid `tusb_control_request_t`.
    let (bm_req_type, b_req, w_val) = unsafe {
        ((*request).bm_request_type, (*request).b_request, (*request).w_value)
    };
    let recipient = bm_req_type & 0x1f;
    let req_type = (bm_req_type >> 5) & 0x03;
    if recipient == TUSB_REQ_RCPT_DEVICE && req_type == TUSB_REQ_TYPE_STANDARD && b_req == TUSB_REQ_SET_ADDRESS {
        set_address(w_val as i32);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn dcd_edpt_open(_rhport: u8, p_endpoint_desc: *const TusbDescEndpoint) -> bool {
    if p_endpoint_desc.is_null() {
        return false;
    }
    // SAFETY: Caller passes a valid `tusb_desc_endpoint_t`.
    let (ep_addr, bm_attr, max_pkt) = unsafe {
        (
            (*p_endpoint_desc).b_endpoint_address,
            (*p_endpoint_desc).bm_attributes,
            (*p_endpoint_desc).w_max_packet_size,
        )
    };
    let epnum = (ep_addr & 0x0f) as usize;
    let dir = u8::from((ep_addr & 0x80) != 0);
    let xfer_type = bm_attr & 0x03;

    if xfer_type == TUSB_XFER_ISOCHRONOUS || epnum >= USBD_MAX_ENDPOINT {
        deadEnd(0xA000);
    }

    let w_type = match xfer_type {
        TUSB_XFER_CONTROL | TUSB_XFER_BULK => USBD_EPXCS_EPCTL_CONTROL,
        TUSB_XFER_INTERRUPT => USBD_EPXCS_EPCTL_INTERRUPT,
        _ => deadEnd(0xA000),
    };

    set_ep_type(epnum, w_type);
    set_ep_address(epnum, epnum as u32);
    set_ep_kind(0, false);

    let pma_addr = init_ep_ram(ep_addr, max_pkt);
    if dir == TUSB_DIR_IN {
        unsafe {
            write_volatile(&raw mut (*btable(epnum)).tx_adr, pma_addr as u32);
            write_volatile(&raw mut (*btable(epnum)).tx_size, max_pkt as u32);
        }
        clear_dtg(epnum, true);
        set_ep_status(epnum, true, USBD_EPXCS_TX_STA_NAK);
    } else {
        unsafe { write_volatile(&raw mut (*btable(epnum)).rx_adr, pma_addr as u32) };
        set_rx_buffer_size(epnum, max_pkt as u32);
        clear_dtg(epnum, false);
        set_ep_status(epnum, false, USBD_EPXCS_RX_STA_NAK);
    }

    let st = unsafe { state_mut() };
    st.xfer_status[2 * epnum + dir as usize].max_packet_size = max_pkt;
    true
}

#[unsafe(no_mangle)]
pub extern "C" fn dcd_edpt_close(_rhport: u8, ep_addr: u8) {
    let epnum = (ep_addr & 0x0f) as usize;
    let dir = u8::from((ep_addr & 0x80) != 0);
    if dir == TUSB_DIR_IN {
        set_ep_status(epnum, true, USBD_EPXCS_TX_STA_DISABLED);
    } else {
        set_ep_status(epnum, false, USBD_EPXCS_RX_STA_DISABLED);
    }
    let st = unsafe { state_mut() };
    if st.ep_nb_ep <= 2 {
        deadEnd(0xA000);
    }
    st.ep_nb_ep -= 1;
    if st.ep_nb_ep == 2 {
        reset_endpoints(true);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn dcd_edpt_xfer(_rhport: u8, ep_addr: u8, buffer: *mut u8, total_bytes: u16) -> bool {
    let epnum = (ep_addr & 0x0f) as usize;
    let dir = u8::from((ep_addr & 0x80) != 0);
    let st = unsafe { state_mut() };
    let setup_ptr = st.setup_packet.as_mut_ptr() as *mut u8;
    let xfer = &mut st.xfer_status[2 * epnum + dir as usize];
    xfer.buffer = if dir == TUSB_DIR_OUT && epnum == 0 && buffer.is_null() {
        setup_ptr
    } else {
        buffer
    };
    xfer.total_len = total_bytes;
    xfer.queued_len = 0;

    if dir == TUSB_DIR_OUT {
        let sz = (total_bytes as u32).min(xfer.max_packet_size as u32);
        set_rx_buffer_size(epnum, sz);
        set_ep_status(epnum, false, USBD_EPXCS_RX_STA_VALID);
    } else {
        dcd_transmit_packet(epnum);
    }
    true
}

#[unsafe(no_mangle)]
pub extern "C" fn dcd_edpt_stall(_rhport: u8, ep_addr: u8) {
    let epnum = (ep_addr & 0x7f) as usize;
    if (ep_addr & 0x80) != 0 {
        set_ep_status(epnum, true, USBD_EPXCS_TX_STA_STALL);
    } else {
        set_ep_status(epnum, false, USBD_EPXCS_RX_STA_STALL);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn dcd_edpt_clear_stall(_rhport: u8, ep_addr: u8) {
    let epnum = (ep_addr & 0x7f) as usize;
    if (ep_addr & 0x80) != 0 {
        set_ep_status(epnum, true, USBD_EPXCS_TX_STA_NAK);
        clear_dtg(epnum, true);
    } else {
        clear_dtg(epnum, false);
        set_ep_status(epnum, false, USBD_EPXCS_RX_STA_NAK);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn dcd_sof_enable(_rhport: u8, _en: bool) {}
