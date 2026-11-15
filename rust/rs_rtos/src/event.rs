#![allow(dead_code)]

//! FreeRTOS-backed event group with shared ownership.
//!
//! [`EventGroup`] is now `Clone` — cloning creates a new reference to the
//! same underlying FreeRTOS event group. The kernel object is freed when
//! the last clone is dropped.
//!
//! All methods take `&self` (not `&mut self`), making it safe to use from
//! a `static` or behind an `Arc`.
//!
//! Two ways to get one:
//!
//! * [`EventGroup::new`] — owns the kernel object through an [`Arc`]; the last
//!   clone frees it. Costs one heap block for the `Arc` itself.
//! * [`EventGroup::uninit`] + [`EventGroup::init_static`] — for a group that
//!   lives in a `static`: no heap at all (the kernel object comes from the C
//!   side's `.bss` pool, `lnFast_EventGroup_c.cpp`) and nothing is freed, which
//!   is what a boot-time singleton wants.

use crate::prelude::*;
use crate::sync::Arc;
use core::ffi::c_void;

#[cfg(any(feature = "esp32", feature = "external_std", not(target_os = "none")))]
mod backend {
    use core::ffi::c_void;
    use crate::c_fast_event as rn_fast_event_c;

    pub fn create() -> *mut c_void {
        let raw = unsafe { rn_fast_event_c::lnfast_event_group_create() };
        assert!(!raw.is_null(), "lnfast_event_group_create returned NULL");
        raw as *mut c_void
    }

    pub unsafe fn delete(raw: *mut c_void) {
        unsafe { rn_fast_event_c::lnfast_event_group_delete(raw as *mut rn_fast_event_c::lnfast_event_group_c) };
    }

    pub unsafe fn take_ownership(raw: *mut c_void) {
        unsafe { rn_fast_event_c::lnfast_event_group_takeOwnership(raw as *mut rn_fast_event_c::lnfast_event_group_c) };
    }

    pub unsafe fn set_events(raw: *mut c_void, bits: u32) {
        unsafe { rn_fast_event_c::lnfast_event_group_set_events(raw as *mut rn_fast_event_c::lnfast_event_group_c, bits) };
    }

    pub unsafe fn wait_events(raw: *mut c_void, bits: u32, timeout_ms: i32) -> u32 {
        unsafe { rn_fast_event_c::lnfast_event_group_wait_events(raw as *mut rn_fast_event_c::lnfast_event_group_c, bits, timeout_ms) }
    }

    pub unsafe fn read_events(raw: *mut c_void, mask: u32) -> u32 {
        unsafe { rn_fast_event_c::lnfast_event_group_read_events(raw as *mut rn_fast_event_c::lnfast_event_group_c, mask) }
    }
}

#[cfg(all(target_os = "none", not(any(feature = "esp32", feature = "external_std"))))]
mod backend {
    use core::cell::UnsafeCell;
    use core::ffi::c_void;
    use core::sync::atomic::{AtomicBool, Ordering};
    use crate::c_freertos as freertos;

    const EVENT_INDEX: freertos::UBaseType_t = 1;
    const LN_FAST_EVENT_GROUP_SLOTS: usize = 4;

    #[cfg(target_arch = "arm")]
    mod arch {
        const SCB_ICSR: *mut u32 = 0xE000_ED04 as *mut u32;
        const PENDSVSET_BIT: u32 = 1 << 28;

        #[inline(always)]
        pub fn in_isr() -> bool {
            let ipsr: u32;
            // SAFETY: Reading IPSR has no side effects.
            unsafe { core::arch::asm!("mrs {}, ipsr", out(reg) ipsr, options(nomem, nostack, preserves_flags)) };
            ipsr != 0
        }

        #[inline(always)]
        pub fn enter_critical_from_isr() -> u32 {
            let old_primask: u32;
            // SAFETY: Saves PRIMASK and disables interrupts on Cortex-M.
            unsafe {
                core::arch::asm!(
                    "mrs {old}, primask",
                    "cpsid i",
                    "dsb",
                    "isb",
                    old = out(reg) old_primask,
                    options(nomem, nostack, preserves_flags)
                );
            }
            old_primask
        }

        #[inline(always)]
        pub fn exit_critical_from_isr(old_state: u32) {
            // SAFETY: Restores PRIMASK saved by `enter_critical_from_isr`.
            unsafe {
                core::arch::asm!(
                    "msr primask, {old}",
                    old = in(reg) old_state,
                    options(nomem, nostack, preserves_flags)
                );
            }
        }

        #[inline(always)]
        pub fn yield_from_isr() {
            // SAFETY: `SCB_ICSR` is the Cortex-M Interrupt Control and State Register.
            unsafe {
                core::ptr::write_volatile(SCB_ICSR, PENDSVSET_BIT);
                core::arch::asm!("dsb", "isb", options(nomem, nostack, preserves_flags));
            }
        }
    }

    #[cfg(target_arch = "riscv32")]
    mod arch {
        const PFIC_GISR: *const u32 = 0xE000_E04C as *const u32;

        unsafe extern "C" {
            fn NVIC_trigger_sw_irq();
        }

        #[inline(always)]
        pub fn in_isr() -> bool {
            // SAFETY: `PFIC_GISR` bit 8 (`NESTSTA`) indicates whether an interrupt is active on CH32V3x.
            let gisr = unsafe { core::ptr::read_volatile(PFIC_GISR) };
            ((gisr >> 8) & 1) != 0
        }

        #[inline(always)]
        pub fn enter_critical_from_isr() -> u32 {
            let old_mstatus: u32;
            // SAFETY: Disables `mstatus.MIE` and returns previous `mstatus`.
            unsafe {
                core::arch::asm!(
                    "csrrci {old}, mstatus, 8",
                    old = out(reg) old_mstatus,
                    options(nomem, nostack, preserves_flags)
                );
            }
            old_mstatus
        }

        #[inline(always)]
        pub fn exit_critical_from_isr(old_state: u32) {
            // SAFETY: Restores `mstatus` saved by `enter_critical_from_isr`.
            unsafe {
                core::arch::asm!(
                    "csrw mstatus, {old}",
                    old = in(reg) old_state,
                    options(nomem, nostack, preserves_flags)
                );
            }
        }

        #[inline(always)]
        pub fn yield_from_isr() {
            // SAFETY: Provided by `rs_ch32v3x::irq`.
            unsafe { NVIC_trigger_sw_irq() };
        }
    }

    struct FastEventGroup {
        value: UnsafeCell<u32>,
        mask: UnsafeCell<u32>,
        waiting_task: UnsafeCell<freertos::TaskHandle_t>,
        in_use: AtomicBool,
    }

    // SAFETY: Access to `value`/`mask`/`waiting_task` is guarded by FreeRTOS critical sections.
    unsafe impl Sync for FastEventGroup {}

    impl FastEventGroup {
        const fn new() -> Self {
            Self {
                value: UnsafeCell::new(0),
                mask: UnsafeCell::new(0),
                waiting_task: UnsafeCell::new(core::ptr::null_mut()),
                in_use: AtomicBool::new(false),
            }
        }

        fn reset(&self) {
            // SAFETY: Called only when a pool slot is exclusively claimed.
            unsafe {
                *self.value.get() = 0;
                *self.mask.get() = 0;
                *self.waiting_task.get() = core::ptr::null_mut();
            }
        }
    }

    static EVG_POOL: [FastEventGroup; LN_FAST_EVENT_GROUP_SLOTS] = [
        FastEventGroup::new(),
        FastEventGroup::new(),
        FastEventGroup::new(),
        FastEventGroup::new(),
    ];

    pub fn create() -> *mut c_void {
        let claimed = critical_section::with(|_| {
            for slot in &EVG_POOL {
                if !slot.in_use.load(Ordering::Relaxed) {
                    slot.in_use.store(true, Ordering::Relaxed);
                    return Some(slot as *const FastEventGroup as *mut FastEventGroup);
                }
            }
            None
        });
        if let Some(slot) = claimed {
            // SAFETY: `slot` was just claimed exclusively for this caller.
            unsafe { (*slot).reset() };
            return slot as *mut c_void;
        }
        let boxed = alloc::boxed::Box::new(FastEventGroup::new());
        alloc::boxed::Box::into_raw(boxed) as *mut c_void
    }

    pub unsafe fn delete(f: *mut c_void) {
        if f.is_null() {
            return;
        }
        let f = f as *mut FastEventGroup;
        for slot in &EVG_POOL {
            if core::ptr::eq(slot as *const FastEventGroup, f) {
                slot.in_use.store(false, Ordering::Release);
                return;
            }
        }
        // SAFETY: Not in `EVG_POOL`, so it was allocated via `Box::into_raw` above.
        unsafe { drop(alloc::boxed::Box::from_raw(f)) };
    }

    pub unsafe fn take_ownership(inst: *mut c_void) {
        let evg = unsafe { &*(inst as *const FastEventGroup) };
        unsafe { *evg.waiting_task.get() = freertos::xTaskGetCurrentTaskHandle() };
    }

    pub unsafe fn set_events(inst: *mut c_void, events: u32) {
        let evg = unsafe { &*(inst as *const FastEventGroup) };
        if arch::in_isr() {
            let saved = arch::enter_critical_from_isr();
            let (wake, task) = unsafe {
                let v = *evg.value.get() | events;
                *evg.value.get() = v;
                ((v & *evg.mask.get()) != 0, *evg.waiting_task.get())
            };
            arch::exit_critical_from_isr(saved);
            if !wake || task.is_null() {
                return;
            }
            let mut awake: freertos::BaseType_t = 0;
            unsafe { freertos::vTaskGenericNotifyGiveFromISR(task, EVENT_INDEX, &mut awake) };
            if awake != 0 {
                arch::yield_from_isr();
            }
            return;
        }

        unsafe { freertos::vPortEnterCritical() };
        let (wake, task) = unsafe {
            let v = *evg.value.get() | events;
            *evg.value.get() = v;
            ((v & *evg.mask.get()) != 0, *evg.waiting_task.get())
        };
        unsafe { freertos::vPortExitCritical() };
        if !wake || task.is_null() {
            return;
        }
        unsafe {
            freertos::xTaskGenericNotify(
                task,
                EVENT_INDEX,
                0,
                freertos::eNotifyAction_eIncrement,
                core::ptr::null_mut(),
            );
        }
    }

    pub unsafe fn wait_events(inst: *mut c_void, maskint: u32, timeout: i32) -> u32 {
        let evg = unsafe { &*(inst as *const FastEventGroup) };
        let task = unsafe { *evg.waiting_task.get() };
        assert!(!task.is_null() && !arch::in_isr(), "EventGroup::wait_events invalid state");
        let ticks: u32 = if timeout == -1 {
            0x7fff_ffff
        } else {
            timeout as u32
        };
        loop {
            unsafe { freertos::vPortEnterCritical() };
            let set = unsafe { maskint & *evg.value.get() };
            if set != 0 {
                unsafe {
                    *evg.value.get() &= !set;
                    *evg.mask.get() = 0;
                    freertos::vPortExitCritical();
                    freertos::xTaskGenericNotifyStateClear(
                        freertos::xTaskGetCurrentTaskHandle(),
                        EVENT_INDEX,
                    );
                }
                return set;
            }
            unsafe {
                *evg.mask.get() = maskint;
                freertos::vPortExitCritical();
            }
            if unsafe { freertos::ulTaskGenericNotifyTake(EVENT_INDEX, 1, ticks) } == 0 {
                return 0;
            }
            unsafe { freertos::vPortEnterCritical() };
            let set = unsafe { maskint & *evg.value.get() };
            unsafe {
                *evg.value.get() &= !set;
                *evg.mask.get() = 0;
                freertos::vPortExitCritical();
            }
            return set;
        }
    }

    pub unsafe fn read_events(inst: *mut c_void, maskint: u32) -> u32 {
        let evg = unsafe { &*(inst as *const FastEventGroup) };
        unsafe { freertos::vPortEnterCritical() };
        let v = unsafe { *evg.value.get() & maskint };
        unsafe {
            *evg.value.get() &= !maskint;
            freertos::vPortExitCritical();
        }
        v
    }
}

// ── Internal ref-counted handle ──────────────────────────────────────

struct EventGroupInner {
    raw: *mut c_void,
}

impl EventGroupInner {
    fn new() -> Self {
        Self { raw: backend::create() }
    }

    fn raw(&self) -> *mut c_void {
        self.raw
    }
}

impl Drop for EventGroupInner {
    fn drop(&mut self) {
        unsafe { backend::delete(self.raw) };
    }
}

// SAFETY: FreeRTOS event groups are usable from any task.
unsafe impl Send for EventGroupInner {}
unsafe impl Sync for EventGroupInner {}

// ── Public type ──────────────────────────────────────────────────────

/// A lightweight FreeRTOS‑compatible event group backed by a fast‑path
/// implementation (no dynamic alloc inside the FreeRTOS scheduler).
///
/// `EventGroup` is `Clone` — all clones share the same underlying kernel
/// object. The object is freed when the last clone is dropped.
///
/// # Example
///
/// ```ignore
/// use rust_esprit::EventGroup;
///
/// let eg = EventGroup::new();
/// let eg2 = eg.clone(); // same kernel object
/// eg.set_events(1);
/// assert!(eg2.wait_events(1, 0) != 0);
/// ```
/// How the kernel object behind an [`EventGroup`] is owned.
enum EventGroupRef {
    /// `EventGroup::new()`: the `Arc` keeps the object alive, the last clone
    /// frees it.
    Shared(Arc<EventGroupInner>),
    /// `EventGroup::uninit()` + [`EventGroup::init_static`]: an object from the
    /// C side's `.bss` pool, never freed (`null` until `init_static` runs).
    Static(*mut c_void),
}

pub struct EventGroup {
    inner: EventGroupRef,
}

impl EventGroup {
    /// Create a new event group (one heap block for the `Arc`).
    pub fn new() -> Self {
        Self {
            inner: EventGroupRef::Shared(Arc::new(EventGroupInner::new())),
        }
    }

    /// An event group with no kernel object yet, constructible in a `static`.
    ///
    /// Call [`EventGroup::init_static`] once before any other method.
    pub const fn uninit() -> Self {
        Self {
            inner: EventGroupRef::Static(core::ptr::null_mut()),
        }
    }

    /// Create the kernel object of an [`EventGroup::uninit`] group.
    ///
    /// Allocates nothing: `lnfast_event_group_create` hands out a slot of its
    /// `.bss` pool. Unlike [`EventGroup::new`], the object is **not** released
    /// when the group goes away — this path is for singletons that live as long
    /// as the firmware. Calling it again is a no-op.
    pub fn init_static(&mut self) {
        if let EventGroupRef::Static(raw) = &mut self.inner {
            if !raw.is_null() {
                return;
            }
            let created = backend::create();
            *raw = created;
        }
    }

    /// The raw C object, for both ownership models.  Advanced use only.
    #[inline]
    pub fn raw(&self) -> *mut c_void {
        match &self.inner {
            EventGroupRef::Shared(inner) => inner.raw(),
            EventGroupRef::Static(raw) => {
                assert!(!raw.is_null(), "EventGroup used before init_static()");
                *raw
            }
        }
    }

    /// Adopt an event group created elsewhere.
    ///
    /// Used when the owner is not Rust — e.g. the hosted build, where the C++
    /// GDB server thread owns a Qt-backed `lnFastEventGroup` and the Rust
    /// `SocketRunner` has to wait on that very object.
    ///
    /// # Safety
    /// `raw` must be a live event group for as long as the result is used, and
    /// no other `EventGroup` may be built from it.
    pub unsafe fn from_raw(raw: *mut c_void) -> Self {
        Self {
            inner: EventGroupRef::Static(raw),
        }
    }

    /// Take ownership of the underlying event group
    /// It means the calling thread will be the one receiving the events
    pub fn take_ownership(&mut self) {
        unsafe { backend::take_ownership(self.raw()) };
    }

    /// Set (signal) one or more event bits.
    ///
    /// Safe to call from ISR context (uses `vTaskGenericNotifyGiveFromISR`
    /// under the hood).
    pub fn set_events(&self, bits: u32) {
        unsafe { backend::set_events(self.raw(), bits) };
    }

    /// Wait for the specified `bits` to be set.
    ///
    /// Returns the actual event bits observed.
    ///
    /// - `timeout_ms < 0`: infinite wait.
    /// - `timeout_ms = 0`: non-blocking poll.
    /// - `timeout_ms > 0`: timeout in milliseconds.
    pub fn wait_events(&self, bits: u32, timeout_ms: i32) -> u32 {
        unsafe { backend::wait_events(self.raw(), bits, timeout_ms) }
    }

    /// Read the current event bits matching `mask` without waiting
    /// (non‑blocking, non‑mutating).
    pub fn read_events(&self, mask: u32) -> u32 {
        unsafe { backend::read_events(self.raw(), mask) }
    }
}

impl Default for EventGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for EventGroup {
    fn clone(&self) -> Self {
        let inner = match &self.inner {
            EventGroupRef::Shared(arc) => EventGroupRef::Shared(arc.clone()),
            // A static group's object is never freed, so a clone is just
            // another view of it: there is no reference count to bump.
            EventGroupRef::Static(raw) => EventGroupRef::Static(*raw),
        };
        Self { inner }
    }
}

// SAFETY: FreeRTOS event groups are usable from any task.
unsafe impl Send for EventGroup {}
unsafe impl Sync for EventGroup {}

