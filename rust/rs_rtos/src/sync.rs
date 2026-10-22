#![allow(dead_code)]

//! Synchronisation primitives wrapping FreeRTOS mutexes and semaphores.
//!
//! Provided types:
//! * [`Mutex<T>`] – a standard (non‑recursive) mutex with RAII guard.
//! * [`RecursiveMutex<T>`] – a mutex that the **same** task can lock repeatedly.
//! * [`RwLock<T>`] – a reader‑writer lock modelled after `std::sync::RwLock`.
//! * [`OnceLock<T>`] – a one‑time initialiser modelled after `std::sync::OnceLock`.
//! * [`LazyLock<T, F>`] – lazy initialisation modelled after `std::sync::LazyLock`.
//! * [`BinarySemaphore`] – a binary (0/1) semaphore.
//! * [`CountingSemaphore`] – a semaphore with a configurable max count.
//!
//! Every wrapper allocates its kernel object with the FreeRTOS *Static() API:
//! the 80 B queue control block lives inside the wrapper (`.bss` as soon as the
//! wrapper is a `static`), so a mutex or a semaphore costs **no heap**.
//!
//! # Lifetime rule
//!
//! A statically allocated kernel object lives in the buffer it was given and its
//! internal lists point back into that buffer, so it must stay where it was
//! created.  Rather than forcing every wrapper to be pinned, the object is
//! created on **first use**: moving a wrapper around before that is harmless,
//! which is what lets `Mutex::new()` stay a `const fn` and lets wrappers live in
//! `Option`s and `LazyLock`s that move them once.  After the first use the rule
//! is the one the C++ wrappers already follow (`lnMutex` and friends keep a
//! `StaticSemaphore_t` in the object): do not move it, do not put it in a
//! growing `Vec`, do not `memcpy` it.
//!
//! `Drop` still calls `vQueueDelete()`, which for a static object only tears the
//! kernel state down — FreeRTOS never frees the caller's buffer.

use crate::prelude::*;
use crate::c_freertos as rn_freertos_c;
use core::ffi::c_void;
use core::ptr::NonNull;
use core::convert::From;

// ---------------------------------------------------------------------------
//  Helper – convert ms to FreeRTOS ticks (or 0 / portMAX_DELAY)
//  Single source of truth: `crate::task::ms_to_ticks` (uses the real
//  `configTICK_RATE_HZ` from FreeRTOSConfig.h).
// ---------------------------------------------------------------------------

// ===========================================================================
//  Static kernel objects
//
//  `STATIC_QUEUE_BYTES` is `sizeof(StaticQueue_t)` of the kernel we are built
//  against, so it lives in `crate::kernel` (one file per kernel) together with
//  the other kernel constants.  It is not a property of the CPU: the in-tree
//  kernel's control block is 80 bytes, ESP-IDF's is 84.  Each of those files
//  documents how the number is measured and how the build checks it.
//
//  Why every wrapper below shares that one storage type: the semaphore and
//  mutex creators are *macros* in `semphr.h` (`xSemaphoreCreateMutexStatic`,
//  `xSemaphoreCreateRecursiveMutexStatic`, `xSemaphoreCreateCountingStatic`,
//  `xSemaphoreCreateBinaryStatic`), and bindgen cannot bind a macro.  What is
//  callable is the queue function behind each of them
//  (`xQueueCreateMutexStatic`, `xQueueCreateCountingSemaphoreStatic`), and its
//  buffer is a `StaticQueue_t` - which is also the type of a
//  `StaticSemaphore_t` (`FreeRTOS.h`: `typedef StaticQueue_t
//  StaticSemaphore_t;`): a mutex or a semaphore *is* a `Queue_t`, with another
//  `ucQueueType` tag and a zero-length item.  So there is exactly one object
//  type to size, and `KernelObject` only picks which creator to call.  The
//  constant is named after the C type it measures, not after its users: the
//  real queue (`crate::queue::Queue`) does not use it, it takes its control
//  block and item buffer from the heap.
// ===========================================================================

/// `sizeof(StaticQueue_t)` (also the size of `StaticSemaphore_t`, which is the
/// same control block under another name).
const STATIC_QUEUE_BYTES: usize = crate::kernel::STATIC_QUEUE_BYTES;

/// Words needed for that many bytes.  A 32-bit alignment is all the control
/// block needs: it only holds pointers, `TickType_t` and `UBaseType_t`.
const STATIC_QUEUE_WORDS: usize = STATIC_QUEUE_BYTES / mem::size_of::<u32>();

/// A trailing partial word would mean the buffer is smaller than the object the
/// kernel writes into it, so refuse to compile.
const _: () = assert!(
    STATIC_QUEUE_BYTES.is_multiple_of(mem::size_of::<u32>()),
    "STATIC_QUEUE_BYTES is not a whole number of u32 words"
);

/// Storage for one statically created queue/semaphore (`.bss` when the wrapper
/// is a `static`, part of the wrapper object otherwise).
///
/// One type for all of them, mutexes included: see "Static kernel objects"
/// above.
#[repr(C)]
struct QueueStorage {
    words: [u32; STATIC_QUEUE_WORDS],
}

impl QueueStorage {
    /// Zeroed storage: the *Static() API expects uninitialised memory and sets
    /// up everything it needs.
    const fn new() -> Self {
        Self {
            words: [0; STATIC_QUEUE_WORDS],
        }
    }
}

/// `ucQueueType` of a plain mutex (FreeRTOS `queue.h`: `queueQUEUE_TYPE_MUTEX`).
const QUEUE_TYPE_MUTEX: u8 = 1;
/// `ucQueueType` of a recursive mutex (`queue.h`: `queueQUEUE_TYPE_RECURSIVE_MUTEX`).
const QUEUE_TYPE_RECURSIVE_MUTEX: u8 = 4;

/// Which kernel object a wrapper needs, in terms of the FreeRTOS *Static() API.
///
/// The `ucQueueType` tag is only kept by the kernel for tracing and
/// `pcQueueGetName()`, it does not change the behaviour: a mutex is set up by
/// `prvInitialiseMutex()` whichever tag is passed.
#[derive(Clone, Copy)]
enum KernelObject {
    /// Mutex created by `xQueueCreateMutexStatic()`, used as the gate of
    /// [`OnceLock`] (semaphore-style take/give, no owner tracking).
    Mutex,
    /// Recursive mutex, what [`Mutex`] and [`RecursiveMutex`] use.
    RecursiveMutex,
    /// Counting semaphore of length 1, the static form of
    /// `xQueueCreateCountingSemaphore(1, given)`.
    BinarySemaphore { given: bool },
    /// Counting semaphore with an explicit ceiling and start value.
    CountingSemaphore { max: u32, initial: u32 },
}

impl KernelObject {
    /// Create the object inside `storage` and return its handle.
    ///
    /// The *Static() creators only fail when the buffer is NULL, which cannot
    /// happen here (the storage belongs to the wrapper), so a NULL return is a
    /// bug — hence the assert in [`handle_or_create`], not an error path.
    fn create(
        self,
        storage: *mut rn_freertos_c::StaticQueue_t,
    ) -> rn_freertos_c::SemaphoreHandle_t {
        unsafe {
            match self {
                KernelObject::Mutex => {
                    rn_freertos_c::xQueueCreateMutexStatic(QUEUE_TYPE_MUTEX, storage)
                }
                KernelObject::RecursiveMutex => {
                    rn_freertos_c::xQueueCreateMutexStatic(QUEUE_TYPE_RECURSIVE_MUTEX, storage)
                }
                KernelObject::BinarySemaphore { given } => {
                    rn_freertos_c::xQueueCreateCountingSemaphoreStatic(1, given as _, storage)
                }
                KernelObject::CountingSemaphore { max, initial } => {
                    rn_freertos_c::xQueueCreateCountingSemaphoreStatic(
                        max as _,
                        initial as _,
                        storage,
                    )
                }
            }
        }
    }
}

/// `taskSCHEDULER_RUNNING` from FreeRTOS `task.h`.
const TASK_SCHEDULER_RUNNING: rn_freertos_c::BaseType_t = 1;

/// Handle of the kernel object held in `storage`, created on first use.
///
/// See the "Lifetime rule" in the module documentation: the object is created
/// here, on first use, so the wrapper may still be moved before that.
///
/// Consequence for ISR callers (`give_from_isr`, ...): the wrapper must have
/// been used once from task context, which is where such a semaphore is created
/// anyway.
fn handle_or_create(
    handle: &UnsafeCell<rn_freertos_c::SemaphoreHandle_t>,
    storage: &UnsafeCell<QueueStorage>,
    object: KernelObject,
) -> rn_freertos_c::SemaphoreHandle_t {
    let mut created = unsafe { *handle.get() };
    if !created.is_null() {
        return created;
    }

    // First use.  Only one task may build the object in its storage, so the
    // check-and-create runs in a critical section -- but only while the
    // scheduler runs: `vPortEnterCritical()` must not be used before that
    // (the ports start with `uxCriticalNesting = 0xaaaaaaaa`, so
    // `vPortExitCritical()` would never unmask interrupts again), and before the
    // scheduler there is no other task to race with anyway.  The *Static()
    // creators neither block nor allocate, so running them here is safe.
    let guarded =
        unsafe { rn_freertos_c::xTaskGetSchedulerState() == TASK_SCHEDULER_RUNNING };
    if guarded {
        unsafe { rn_freertos_c::vPortEnterCritical() };
    }
    created = unsafe { *handle.get() };
    if created.is_null() {
        created = object.create(storage.get().cast());
        assert!(
            !created.is_null(),
            "static FreeRTOS object creation returned NULL"
        );
        unsafe { *handle.get() = created };
    }
    if guarded {
        unsafe { rn_freertos_c::vPortExitCritical() };
    }
    created
}

/// Delete a kernel object if it was ever created (the wrappers' `Drop`).
///
/// `vQueueDelete()` on a static object only resets the kernel state: the
/// storage belongs to the wrapper, so nothing is freed here.
fn delete_if_created(handle: &UnsafeCell<rn_freertos_c::SemaphoreHandle_t>) {
    let created = unsafe { *handle.get() };
    if !created.is_null() {
        unsafe { rn_freertos_c::vQueueDelete(created) }
    }
}
// ===========================================================================
//  Mutex<T>
// ===========================================================================

/// A FreeRTOS-backed mutual exclusion primitive.
///
/// The underlying FreeRTOS object is a **recursive** mutex, so the owning
/// task may lock it multiple times without deadlocking.  Each `lock()` call
/// must be paired with a corresponding `drop()` of the guard.
///
/// # Example
/// ```ignore
/// static DATA: Mutex<u32> = Mutex::new(0);
/// *DATA.lock() += 1;
/// ```
pub struct Mutex<T: ?Sized> {
    /// Kernel handle, null until the object is created on first use.
    handle: UnsafeCell<rn_freertos_c::SemaphoreHandle_t>, // actually a QueueHandle_t
    /// Storage the kernel control block lives in (sized from
    /// `crate::kernel::STATIC_QUEUE_BYTES`, `.bss` when the wrapper is a
    /// `static` -- no heap either way).
    storage: UnsafeCell<QueueStorage>,
    data: UnsafeCell<T>,
    // FreeRTOS mutexes are usable from any task, so we can implement Send + Sync.
}

// SAFETY: FreeRTOS mutexes are safe to send between tasks.
unsafe impl<T: Send> Send for Mutex<T> {}
// SAFETY: FreeRTOS mutexes provide mutual exclusion across tasks.
unsafe impl<T: Send> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    /// Create a new mutex wrapping `value`.
    ///
    /// Nothing is allocated and no kernel call is made: the FreeRTOS object is
    /// created in the mutex' own storage the first time it is used (see the
    /// module documentation for the move rule that makes that necessary).
    pub const fn new(value: T) -> Self {
        Self {
            handle: UnsafeCell::new(core::ptr::null_mut()),
            storage: UnsafeCell::new(QueueStorage::new()),
            data: UnsafeCell::new(value),
        }
    }
}

impl<T: ?Sized> Mutex<T> {
    /// Handle of the underlying recursive mutex, created on first use.
    fn kernel_handle(&self) -> rn_freertos_c::SemaphoreHandle_t {
        handle_or_create(&self.handle, &self.storage, KernelObject::RecursiveMutex)
    }

    /// Lock the mutex, blocking until it is acquired.
    ///
    /// Returns a [`MutexGuard`] that releases the lock when dropped.
    pub fn lock(&self) -> MutexGuard<'_, T> {
        let ret = unsafe { rn_freertos_c::xQueueTakeMutexRecursive(self.kernel_handle(), u32::MAX) };
        assert!(ret != 0, "Mutex::lock failed (timed out on infinite wait)");
        MutexGuard {
            mutex: self,
            _not_send: PhantomData,
        }
    }

    /// Try to lock without blocking.
    ///
    /// Returns `Some(guard)` if the lock was acquired, `None` otherwise.
    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        let ret = unsafe { rn_freertos_c::xQueueTakeMutexRecursive(self.kernel_handle(), 0) };
        if ret != 0 {
            Some(MutexGuard {
                mutex: self,
                _not_send: PhantomData,
            })
        } else {
            None
        }
    }

    /// Lock with a timeout in milliseconds.
    ///
    /// Returns `Some(guard)` if acquired within the timeout, `None` on timeout.
    pub fn lock_timeout(&self, timeout_ms: u32) -> Option<MutexGuard<'_, T>> {
        let ret = unsafe { rn_freertos_c::xQueueTakeMutexRecursive(self.kernel_handle(), crate::task::ms_to_ticks(timeout_ms)) };
        if ret != 0 {
            Some(MutexGuard {
                mutex: self,
                _not_send: PhantomData,
            })
        } else {
            None
        }
    }

    /// Access the inner value without locking.
    ///
    /// # Safety
    /// The caller must ensure that no other task holds the lock.
    pub unsafe fn unsafe_get(&self) -> &T {
        &*self.data.get()
    }

    /// Mutably access the inner value without locking.
    ///
    /// # Safety
    /// The caller must ensure that no other task holds the lock.
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn unsafe_get_mut(&self) -> &mut T {
        &mut *self.data.get()
    }
}

impl<T: ?Sized> Drop for Mutex<T> {
    fn drop(&mut self) {
        delete_if_created(&self.handle)
    }
}

// ===========================================================================
//  MutexGuard
// ===========================================================================

/// RAII guard returned by [`Mutex::lock`] and friends.
///
/// The lock is released when this value is dropped.
pub struct MutexGuard<'a, T: ?Sized> {
    mutex: &'a Mutex<T>,
    _not_send: PhantomData<*mut ()>, // !Send
}

impl<T: ?Sized> Deref for MutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T: ?Sized> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<T: ?Sized> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        unsafe { rn_freertos_c::xQueueGiveMutexRecursive(self.mutex.kernel_handle()); }
    }
}

// ===========================================================================
//  RecursiveMutex<T>
// ===========================================================================

/// A FreeRTOS-backed **recursive** mutex.
///
/// The same task can lock this mutex multiple times without deadlocking.
/// Each `lock()` call must be paired with a corresponding `drop()` of the guard.
///
/// # Example
/// ```ignore
/// static M: RecursiveMutex<i32> = RecursiveMutex::new(0);
/// {
///     let _g1 = M.lock();
///     let _g2 = M.lock(); // OK – same task
/// }
/// ```
pub struct RecursiveMutex<T: ?Sized> {
    /// Kernel handle, null until the object is created on first use.
    handle: UnsafeCell<rn_freertos_c::SemaphoreHandle_t>,
    /// Storage the kernel control block lives in (sized from
    /// `crate::kernel::STATIC_QUEUE_BYTES`, `.bss` when the wrapper is a
    /// `static` -- no heap either way).
    storage: UnsafeCell<QueueStorage>,
    data: UnsafeCell<T>,
}

// SAFETY: same reasoning as Mutex
unsafe impl<T: Send> Send for RecursiveMutex<T> {}
unsafe impl<T: Send> Sync for RecursiveMutex<T> {}

impl<T> RecursiveMutex<T> {
    /// Create a new recursive mutex.
    ///
    /// The kernel object is created in the mutex' own storage on first use.
    pub const fn new(value: T) -> Self {
        Self {
            handle: UnsafeCell::new(core::ptr::null_mut()),
            storage: UnsafeCell::new(QueueStorage::new()),
            data: UnsafeCell::new(value),
        }
    }
}

impl<T: ?Sized> RecursiveMutex<T> {
    /// Handle of the underlying recursive mutex, created on first use.
    fn kernel_handle(&self) -> rn_freertos_c::SemaphoreHandle_t {
        handle_or_create(&self.handle, &self.storage, KernelObject::RecursiveMutex)
    }

    /// Lock the recursive mutex (blocking).
    pub fn lock(&self) -> RecursiveMutexGuard<'_, T> {
        let ret = unsafe { rn_freertos_c::xQueueTakeMutexRecursive(self.kernel_handle(), u32::MAX) };
        assert!(ret != 0, "RecursiveMutex::lock failed");
        RecursiveMutexGuard {
            mutex: self,
            _not_send: PhantomData,
        }
    }

    /// Try to lock without blocking.
    pub fn try_lock(&self) -> Option<RecursiveMutexGuard<'_, T>> {
        let ret = unsafe { rn_freertos_c::xQueueTakeMutexRecursive(self.kernel_handle(), 0) };
        if ret != 0 {
            Some(RecursiveMutexGuard {
                mutex: self,
                _not_send: PhantomData,
            })
        } else {
            None
        }
    }

    /// Lock with a timeout.
    pub fn lock_timeout(&self, timeout_ms: u32) -> Option<RecursiveMutexGuard<'_, T>> {
        let ret =
            unsafe { rn_freertos_c::xQueueTakeMutexRecursive(self.kernel_handle(), crate::task::ms_to_ticks(timeout_ms)) };
        if ret != 0 {
            Some(RecursiveMutexGuard {
                mutex: self,
                _not_send: PhantomData,
            })
        } else {
            None
        }
    }
}

impl<T: ?Sized> Drop for RecursiveMutex<T> {
    fn drop(&mut self) {
        delete_if_created(&self.handle)
    }
}

// ===========================================================================
//  RecursiveMutexGuard
// ===========================================================================

/// RAII guard for [`RecursiveMutex`].
pub struct RecursiveMutexGuard<'a, T: ?Sized> {
    mutex: &'a RecursiveMutex<T>,
    _not_send: PhantomData<*mut ()>,
}

impl<T: ?Sized> Deref for RecursiveMutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T: ?Sized> DerefMut for RecursiveMutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<T: ?Sized> Drop for RecursiveMutexGuard<'_, T> {
    fn drop(&mut self) {
        unsafe { rn_freertos_c::xQueueGiveMutexRecursive(self.mutex.kernel_handle()); }
    }
}

// ===========================================================================
//  RwLock<T> – Reader‑Writer lock
// ===========================================================================

/// A FreeRTOS‑backed reader‑writer lock.
///
/// Allows any number of readers or a single writer.  Modelled after
/// `std::sync::RwLock`.
///
/// # Implementation
///
/// Uses a recursive mutex to protect the reader count and data, plus a
/// binary semaphore for writers to wait until all readers have finished.
pub struct RwLock<T: ?Sized> {
    /// Recursive mutex protecting `reader_count` and the data (null until the
    /// object is created on first use).
    gate: UnsafeCell<rn_freertos_c::SemaphoreHandle_t>,
    /// Storage the gate's kernel control block lives in (80 B, `.bss`).
    gate_storage: UnsafeCell<QueueStorage>,
    /// Binary semaphore – writers wait on this until readers drain (null until
    /// the object is created on first use).
    drain: UnsafeCell<rn_freertos_c::SemaphoreHandle_t>,
    /// Storage the drain's kernel control block lives in (80 B, `.bss`).
    drain_storage: UnsafeCell<QueueStorage>,
    /// Number of active readers.
    reader_count: UnsafeCell<u32>,
    /// Protected data.
    data: UnsafeCell<T>,
}

// SAFETY: RwLock provides mutual exclusion for writers and shared access for readers.
unsafe impl<T: Send> Send for RwLock<T> {}
unsafe impl<T: Send + Sync> Sync for RwLock<T> {}

impl<T> RwLock<T> {
    /// Create a new `RwLock` wrapping `value`.
    ///
    /// The gate mutex and the drain semaphore are created in the lock's own
    /// storage on first use, so this neither allocates nor touches the kernel.
    pub const fn new(value: T) -> Self {
        Self {
            gate: UnsafeCell::new(core::ptr::null_mut()),
            gate_storage: UnsafeCell::new(QueueStorage::new()),
            drain: UnsafeCell::new(core::ptr::null_mut()),
            drain_storage: UnsafeCell::new(QueueStorage::new()),
            reader_count: UnsafeCell::new(0),
            data: UnsafeCell::new(value),
        }
    }
}

impl<T: ?Sized> RwLock<T> {
    /// Handle of the gate mutex, created on first use.
    fn gate_handle(&self) -> rn_freertos_c::SemaphoreHandle_t {
        handle_or_create(&self.gate, &self.gate_storage, KernelObject::RecursiveMutex)
    }

    /// Handle of the drain semaphore (max 1, starts given), created on first use.
    fn drain_handle(&self) -> rn_freertos_c::SemaphoreHandle_t {
        handle_or_create(
            &self.drain,
            &self.drain_storage,
            KernelObject::CountingSemaphore { max: 1, initial: 1 },
        )
    }

    /// Lock for reading (shared access).
    ///
    /// Blocks if a writer holds the lock.  Multiple readers can hold the
    /// lock simultaneously.
    pub fn read(&self) -> RwLockReadGuard<'_, T> {
        unsafe { rn_freertos_c::xQueueTakeMutexRecursive(self.gate_handle(), u32::MAX) };
        // SAFETY: we hold the gate mutex, so no other task can access reader_count
        let count = unsafe { &mut *self.reader_count.get() };
        if *count == 0 {
            // First reader – take the drain semaphore so writers will block
            unsafe { rn_freertos_c::xQueueSemaphoreTake(self.drain_handle(), u32::MAX) };
        }
        *count += 1;
        unsafe { rn_freertos_c::xQueueGiveMutexRecursive(self.gate_handle()) };
        RwLockReadGuard { lock: self }
    }

    /// Try to lock for reading without blocking.
    pub fn try_read(&self) -> Option<RwLockReadGuard<'_, T>> {
        let wret = unsafe { rn_freertos_c::xQueueTakeMutexRecursive(self.gate_handle(), 0) };
        if wret == 0 {
            return None;
        }
        // SAFETY: we hold the gate mutex, so no other task can access reader_count
        let count = unsafe { &mut *self.reader_count.get() };
        if *count == 0 {
            let dret = unsafe { rn_freertos_c::xQueueSemaphoreTake(self.drain_handle(), 0) };
            if dret == 0 {
                // Writer is waiting – bail
                unsafe { rn_freertos_c::xQueueGiveMutexRecursive(self.gate_handle()) };
                return None;
            }
        }
        *count += 1;
        unsafe { rn_freertos_c::xQueueGiveMutexRecursive(self.gate_handle()) };
        Some(RwLockReadGuard { lock: self })
    }

    /// Lock for writing (exclusive access).
    ///
    /// Blocks until all readers and other writers have released.
    pub fn write(&self) -> RwLockWriteGuard<'_, T> {
        // Take the drain semaphore – this blocks until all current readers finish
        unsafe { rn_freertos_c::xQueueSemaphoreTake(self.drain_handle(), u32::MAX) };
        // Now take the gate to protect data access
        unsafe { rn_freertos_c::xQueueTakeMutexRecursive(self.gate_handle(), u32::MAX) };
        RwLockWriteGuard { lock: self }
    }

    /// Try to lock for writing without blocking.
    pub fn try_write(&self) -> Option<RwLockWriteGuard<'_, T>> {
        let dret = unsafe { rn_freertos_c::xQueueSemaphoreTake(self.drain_handle(), 0) };
        if dret == 0 {
            return None; // readers active
        }
        let gret = unsafe { rn_freertos_c::xQueueTakeMutexRecursive(self.gate_handle(), 0) };
        if gret == 0 {
            // Shouldn't happen since we hold drain, but release and bail
            unsafe { rn_freertos_c::xQueueGenericSend(self.drain_handle(), core::ptr::null::<c_void>(), 0, 0) };
            return None;
        }
        Some(RwLockWriteGuard { lock: self })
    }
}

impl<T: ?Sized> Drop for RwLock<T> {
    fn drop(&mut self) {
        delete_if_created(&self.gate);
        delete_if_created(&self.drain);
    }
}

/// RAII guard returned by [`RwLock::read`].
pub struct RwLockReadGuard<'a, T: ?Sized> {
    lock: &'a RwLock<T>,
}

impl<T: ?Sized> Deref for RwLockReadGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T: ?Sized> Drop for RwLockReadGuard<'_, T> {
    fn drop(&mut self) {
        unsafe { rn_freertos_c::xQueueTakeMutexRecursive(self.lock.gate_handle(), u32::MAX) };
        // SAFETY: we hold the gate mutex, so no other task can access reader_count
        let count = unsafe { &mut *self.lock.reader_count.get() };
        *count -= 1;
        if *count == 0 {
            // Last reader – release the drain semaphore so writers can proceed
            unsafe { rn_freertos_c::xQueueGenericSend(self.lock.drain_handle(), core::ptr::null::<c_void>(), 0, 0) };
        }
        unsafe { rn_freertos_c::xQueueGiveMutexRecursive(self.lock.gate_handle()) };
    }
}

/// RAII guard returned by [`RwLock::write`].
pub struct RwLockWriteGuard<'a, T: ?Sized> {
    lock: &'a RwLock<T>,
}

impl<T: ?Sized> Deref for RwLockWriteGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T: ?Sized> DerefMut for RwLockWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T: ?Sized> Drop for RwLockWriteGuard<'_, T> {
    fn drop(&mut self) {
        unsafe {
            // Release the gate
            rn_freertos_c::xQueueGiveMutexRecursive(self.lock.gate_handle());
            // Release the drain semaphore so new readers/writers can proceed
            rn_freertos_c::xQueueGenericSend(self.lock.drain_handle(), core::ptr::null::<c_void>(), 0, 0);
        }
    }
}

// ===========================================================================
//  OnceLock<T> – One‑time initialiser
// ===========================================================================

/// A synchronisation primitive that can be written to **once**.
///
/// Modelled after `std::sync::OnceLock`.  Useful for lazy global
/// singletons.
///
/// # Example
/// ```ignore
/// static CONFIG: OnceLock<Config> = OnceLock::new();
/// let cfg = CONFIG.get_or_init(|| Config::load());
/// ```
pub struct OnceLock<T> {
    data: UnsafeCell<Option<T>>,
    /// Binary semaphore: guards the initialisation critical section.
    /// Null until the object is created on first use.
    gate: UnsafeCell<rn_freertos_c::SemaphoreHandle_t>,
    /// Storage the gate's kernel control block lives in (80 B, `.bss`).
    storage: UnsafeCell<QueueStorage>,
    inited: AtomicBool,
}

// SAFETY: OnceLock synchronises access via the semaphore + atomic flag.
unsafe impl<T: Send + Sync> Sync for OnceLock<T> {}
unsafe impl<T: Send> Send for OnceLock<T> {}

impl<T> OnceLock<T> {
    /// Create a new, empty `OnceLock`.
    pub const fn new() -> Self {
        Self {
            data: UnsafeCell::new(None),
            gate: UnsafeCell::new(core::ptr::null_mut()),
            storage: UnsafeCell::new(QueueStorage::new()),
            inited: AtomicBool::new(false),
        }
    }

    /// Handle of the gate semaphore, created on first use.
    ///
    /// Creating the object needs no heap (the storage is ours) and the
    /// check-and-create is done inside a FreeRTOS critical section, so two tasks
    /// racing to initialise cannot both create a gate (which would corrupt the
    /// storage and leak one mutex).
    fn gate_handle(&self) -> rn_freertos_c::SemaphoreHandle_t {
        handle_or_create(&self.gate, &self.storage, KernelObject::Mutex)
    }

    /// Get a reference to the contained value, or initialise it with `f`.
    ///
    /// Blocks if another task is currently initialising.
    pub fn get_or_init<F>(&self, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        if !self.inited.load(Ordering::Acquire) {
            // Try to acquire the gate – only one task gets through
            unsafe { rn_freertos_c::xQueueSemaphoreTake(self.gate_handle(), u32::MAX) };
            if !self.inited.load(Ordering::Acquire) {
                // Double-check: we are the initialiser
                unsafe {
                    *self.data.get() = Some(f());
                }
                self.inited.store(true, Ordering::Release);
            }
            // Release the gate
            unsafe {
                rn_freertos_c::xQueueGenericSend(
                    self.gate_handle(),
                    core::ptr::null::<c_void>(),
                    0,
                    0,
                );
            }
        }
        unsafe { (*self.data.get()).as_ref().unwrap_unchecked() }
    }

    /// Get a reference to the contained value, or return `None`.
    pub fn get(&self) -> Option<&T> {
        if self.inited.load(Ordering::Acquire) {
            unsafe { (*self.data.get()).as_ref() }
        } else {
            None
        }
    }

    /// Set the value.  Returns `Ok(())` on success, or `Err(value)` if
    /// already initialised.
    pub fn set(&self, value: T) -> Result<(), T> {
        if self.inited.load(Ordering::Acquire) {
            return Err(value);
        }
        unsafe { rn_freertos_c::xQueueSemaphoreTake(self.gate_handle(), u32::MAX) };
        if self.inited.load(Ordering::Acquire) {
            unsafe {
                rn_freertos_c::xQueueGenericSend(
                    self.gate_handle(),
                    core::ptr::null::<c_void>(),
                    0,
                    0,
                );
            }
            return Err(value);
        }
        unsafe {
            *self.data.get() = Some(value);
        }
        self.inited.store(true, Ordering::Release);
        unsafe {
            rn_freertos_c::xQueueGenericSend(
                self.gate_handle(),
                core::ptr::null::<c_void>(),
                0,
                0,
            );
        }
        Ok(())
    }

    /// Returns `true` if the `OnceLock` has been initialised.
    pub fn is_initialized(&self) -> bool {
        self.inited.load(Ordering::Acquire)
    }
}

impl<T> Drop for OnceLock<T> {
    fn drop(&mut self) {
        delete_if_created(&self.gate)
    }
}

impl<T> Default for OnceLock<T> {
    fn default() -> Self {
        Self::new()
    }
}

// ===========================================================================
//  LazyLock<T, F> – Lazy initialisation
// ===========================================================================

/// A value that is lazily initialised on first access, modelled after
/// `std::sync::LazyLock`.
///
/// Unlike [`OnceLock`], this type stores the initialiser closure and
/// runs it automatically on first dereference.
///
/// # Example
/// ```ignore
/// static DATA: LazyLock<u32> = LazyLock::new(|| compute_expensive());
/// let v = *DATA; // initialises on first access
/// ```
pub struct LazyLock<T, F = fn() -> T> {
    once: OnceLock<T>,
    init: UnsafeCell<Option<F>>,
}

unsafe impl<T: Send + Sync, F: Send> Sync for LazyLock<T, F> {}
unsafe impl<T: Send, F: Send> Send for LazyLock<T, F> {}

impl<T, F: FnOnce() -> T> LazyLock<T, F> {
    /// Create a new `LazyLock` that will call `f` on first access.
    pub const fn new(f: F) -> Self {
        Self {
            once: OnceLock::new(),
            init: UnsafeCell::new(Some(f)),
        }
    }

    /// Force initialisation (if not already done) and return a reference.
    pub fn force(this: &Self) -> &T {
        this.once.get_or_init(|| unsafe {
            let f = (*this.init.get()).take().unwrap_unchecked();
            f()
        })
    }

    /// Returns `true` if the value has been initialised.
    pub fn is_initialized(&self) -> bool {
        self.once.is_initialized()
    }
}

impl<T, F: FnOnce() -> T> Deref for LazyLock<T, F> {
    type Target = T;
    fn deref(&self) -> &T {
        Self::force(self)
    }
}

// ===========================================================================
//  Semaphore helpers
// ===========================================================================

/// An RAII handle returned when taking a semaphore.
///
/// The semaphore is given back when this handle is dropped.
pub struct SemaphoreGuard<'a> {
    handle: rn_freertos_c::SemaphoreHandle_t,
    semaphore: &'a dyn SemaphoreOps,
}

impl Drop for SemaphoreGuard<'_> {
    fn drop(&mut self) {
        self.semaphore.give();
    }
}

// Trait used internally to allow SemaphoreGuard to call give()
trait SemaphoreOps {
    fn give(&self);
    fn handle(&self) -> rn_freertos_c::SemaphoreHandle_t;
}

// ===========================================================================
//  BinarySemaphore
// ===========================================================================

/// A binary semaphore – can be in state 0 (taken) or 1 (given).
///
/// Unlike a mutex, a semaphore has no concept of "owner"; any task can give
/// and any task can take.
pub struct BinarySemaphore {
    /// Kernel handle, null until the object is created on first use.
    handle: UnsafeCell<rn_freertos_c::SemaphoreHandle_t>,
    /// Storage the kernel control block lives in (sized from
    /// `crate::kernel::STATIC_QUEUE_BYTES`, `.bss` when the wrapper is a
    /// `static` -- no heap either way).
    storage: UnsafeCell<QueueStorage>,
    /// State to start in, applied when the object is created.
    given: bool,
}

// SAFETY: binary semaphores are usable across tasks.
unsafe impl Send for BinarySemaphore {}
unsafe impl Sync for BinarySemaphore {}

impl BinarySemaphore {
    /// Create a new binary semaphore, initially **not** given (i.e. take will block).
    ///
    /// The kernel object is created in the semaphore's own storage on first use.
    pub const fn new() -> Self {
        Self::with_state(false)
    }

    /// Create a new binary semaphore that starts in the "given" state.
    pub const fn new_given() -> Self {
        Self::with_state(true)
    }

    /// Common constructor: `given` is applied by the kernel when the object is
    /// created on first use.
    const fn with_state(given: bool) -> Self {
        Self {
            handle: UnsafeCell::new(core::ptr::null_mut()),
            storage: UnsafeCell::new(QueueStorage::new()),
            given,
        }
    }

    /// Handle of the underlying counting semaphore, created on first use.
    fn kernel_handle(&self) -> rn_freertos_c::SemaphoreHandle_t {
        handle_or_create(
            &self.handle,
            &self.storage,
            KernelObject::BinarySemaphore { given: self.given },
        )
    }

    /// Give the semaphore (increment to 1).
    /// Returns `true` if the give was successful.
    pub fn give(&self) -> bool {
        let ret = unsafe {
            rn_freertos_c::xQueueGenericSend(self.kernel_handle(), core::ptr::null::<c_void>(), 0, 0)
        };
        ret != 0
    }

    /// Give the semaphore from an ISR.
    /// Returns `true` if the operation was successful.
    ///
    /// The semaphore must already have been used from task context (that is
    /// where its storage is initialised, see the module documentation).
    pub fn give_from_isr(&self) -> bool {
        let mut higher_priority_task_woken: rn_freertos_c::BaseType_t = 0;
        let ret = unsafe {
            rn_freertos_c::xQueueGiveFromISR(self.kernel_handle(), &mut higher_priority_task_woken)
        };
        ret != 0
    }

    /// Take the semaphore, blocking until it becomes available.
    pub fn take(&self) {
        let ret = unsafe { rn_freertos_c::xQueueSemaphoreTake(self.kernel_handle(), u32::MAX) };
        assert!(ret != 0, "BinarySemaphore::take failed");
    }

    /// Try to take without blocking.
    pub fn try_take(&self) -> bool {
        unsafe { rn_freertos_c::xQueueSemaphoreTake(self.kernel_handle(), 0) != 0 }
    }

    /// Take with a timeout.
    pub fn take_timeout(&self, timeout_ms: u32) -> bool {
        unsafe { rn_freertos_c::xQueueSemaphoreTake(self.kernel_handle(), crate::task::ms_to_ticks(timeout_ms)) != 0 }
    }
}

impl Default for BinarySemaphore {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for BinarySemaphore {
    fn drop(&mut self) {
        delete_if_created(&self.handle)
    }
}

// ===========================================================================
//  CountingSemaphore
// ===========================================================================

/// A counting semaphore with a configurable maximum count.
///
/// Typical use: managing a pool of resources.
pub struct CountingSemaphore {
    /// Kernel handle, null until the object is created on first use.
    handle: UnsafeCell<rn_freertos_c::SemaphoreHandle_t>,
    /// Storage the kernel control block lives in (sized from
    /// `crate::kernel::STATIC_QUEUE_BYTES`, `.bss` when the wrapper is a
    /// `static` -- no heap either way).
    storage: UnsafeCell<QueueStorage>,
    /// Ceiling and start value, applied when the object is created.
    max: u32,
    initial: u32,
}

unsafe impl Send for CountingSemaphore {}
unsafe impl Sync for CountingSemaphore {}

impl CountingSemaphore {
    /// Create a counting semaphore with `max_count` and `initial_count`.
    ///
    /// The kernel object is created in the semaphore's own storage on first
    /// use, which is also where FreeRTOS checks the pair
    /// (`max_count != 0`, `initial_count <= max_count`).
    pub const fn new(max_count: u32, initial_count: u32) -> Self {
        Self {
            handle: UnsafeCell::new(core::ptr::null_mut()),
            storage: UnsafeCell::new(QueueStorage::new()),
            max: max_count,
            initial: initial_count,
        }
    }

    /// Handle of the underlying counting semaphore, created on first use.
    fn kernel_handle(&self) -> rn_freertos_c::SemaphoreHandle_t {
        handle_or_create(
            &self.handle,
            &self.storage,
            KernelObject::CountingSemaphore {
                max: self.max,
                initial: self.initial,
            },
        )
    }

    /// Give (increment) the semaphore.
    pub fn give(&self) -> bool {
        let ret = unsafe {
            rn_freertos_c::xQueueGenericSend(self.kernel_handle(), core::ptr::null::<c_void>(), 0, 0)
        };
        ret != 0
    }

    /// Block until we can take (decrement) the semaphore.
    pub fn take(&self) {
        let ret = unsafe { rn_freertos_c::xQueueSemaphoreTake(self.kernel_handle(), u32::MAX) };
        assert!(ret != 0, "CountingSemaphore::take failed");
    }

    /// Try to take without blocking.
    pub fn try_take(&self) -> bool {
        unsafe { rn_freertos_c::xQueueSemaphoreTake(self.kernel_handle(), 0) != 0 }
    }

    /// Take with a timeout.
    pub fn take_timeout(&self, timeout_ms: u32) -> bool {
        unsafe { rn_freertos_c::xQueueSemaphoreTake(self.kernel_handle(), crate::task::ms_to_ticks(timeout_ms)) != 0 }
    }
}

impl Drop for CountingSemaphore {
    fn drop(&mut self) {
        delete_if_created(&self.handle)
    }
}

// ===========================================================================
//  Arc<T> – Atomically Reference‑Counted Pointer
// ===========================================================================

/// A thread‑safe reference‑counting pointer, modelled after
/// `std::sync::Arc`.
///
/// `Arc` stands for "Atomically Reference Counted".  The reference count
/// is manipulated using safe atomic operations, so `Arc` can be sent
/// between tasks and shared freely.
///
/// The inner value `T` is dropped when the last `Arc` pointing to it is
/// dropped.
///
/// # Example
/// ```ignore
/// use rust_esprit::sync::Arc;
///
/// let a = Arc::new(42);
/// let b = a.clone();
/// assert_eq!(*a, *b);
/// ```
pub struct Arc<T: ?Sized> {
    ptr: NonNull<ArcInner<T>>,
    _phantom: PhantomData<ArcInner<T>>,
}

// SAFETY: Arc uses atomic refcounting and provides shared ownership.
unsafe impl<T: Send + Sync> Send for Arc<T> {}
unsafe impl<T: Send + Sync> Sync for Arc<T> {}

/// Internal layout: refcount followed by the value.
struct ArcInner<T: ?Sized> {
    /// Reference count.  Starts at 1.
    count: AtomicUsize,
    /// The wrapped value.
    data: T,
}

impl<T> Arc<T> {
    /// Construct a new `Arc<T>` wrapping `value`.
    ///
    /// The initial reference count is 1.
    pub fn new(value: T) -> Self {
        let inner = Box::new(ArcInner {
            count: AtomicUsize::new(1),
            data: value,
        });
        let ptr = NonNull::new(Box::into_raw(inner))
            .expect("Arc::new: Box::into_raw returned NULL");
        Self {
            ptr,
            _phantom: PhantomData,
        }
    }

    /// Consume the `Arc`, returning the wrapped value.
    ///
    /// Returns `Ok(value)` if this is the last reference, or `Err(self)`
    /// if other references still exist.
    pub fn try_unwrap(this: Self) -> Result<T, Self> {
        // SAFETY: we check the refcount first
        let count = unsafe { this.ptr.as_ref() }.count.load(Ordering::Acquire);
        if count == 1 {
            unsafe {
                let inner = Box::from_raw(this.ptr.as_ptr());
                mem::forget(this);
                Ok(inner.data)
            }
        } else {
            Err(this)
        }
    }

    /// Returns a mutable reference to the inner value, if there are no
    /// other `Arc` pointers to this allocation.
    pub fn get_mut(this: &mut Self) -> Option<&mut T> {
        // SAFETY: we check the refcount first
        let count = unsafe { this.ptr.as_ref() }.count.load(Ordering::Acquire);
        if count == 1 {
            Some(unsafe { &mut *core::ptr::addr_of_mut!((*this.ptr.as_ptr()).data) })
        } else {
            None
        }
    }

    /// Returns `true` if this is the only reference to the inner value.
    pub fn is_unique(this: &Self) -> bool {
        // SAFETY: we only read the refcount
        unsafe { this.ptr.as_ref() }.count.load(Ordering::Acquire) == 1
    }

    /// Return the current reference count (for debugging).
    pub fn ref_count(this: &Self) -> usize {
        // SAFETY: we only read the refcount
        unsafe { this.ptr.as_ref() }.count.load(Ordering::Relaxed)
    }
}

impl<T: ?Sized> Arc<T> {
    /// Get a raw pointer to the inner value.
    ///
    /// The caller must ensure that the `Arc` is kept alive while the
    /// pointer is in use.
    pub fn as_ptr(this: &Self) -> *const T {
        unsafe { core::ptr::addr_of!((*this.ptr.as_ptr()).data) }
    }

    fn inner(&self) -> &ArcInner<T> {
        unsafe { self.ptr.as_ref() }
    }
}

// ---------------------------------------------------------------------------
//  Arc refcount helpers
//
//  On targets with atomic read-modify-write instructions
//  (`target_has_atomic = "ptr"`, e.g. ARMv7-M/thumbv7m, RV32IMAC) we use the
//  lock-free `fetch_add`/`fetch_sub`.
//
//  On targets that only expose atomic loads/stores (Cortex-M0+ such as the
//  dual-core RP2040, and RV32IMC) a plain load/store update is NOT atomic, so
//  the refcount update is guarded with a FreeRTOS critical section.  This is
//  cross-core safe on SMP FreeRTOS ports (the RP2040 SMP port takes a spinlock
//  inside `vPortEnterCritical`) and prevents a lost refcount update when both
//  cores clone/drop the same `Arc` concurrently.
//
//  NOTE: like the rest of the sync API, these helpers must not be used from
//  ISR context (task-level FreeRTOS critical sections only).
// ---------------------------------------------------------------------------

/// Atomically `count += 1`, returning the previous value.
#[inline]
fn refcount_fetch_add(count: &AtomicUsize) -> usize {
    #[cfg(target_has_atomic = "ptr")]
    {
        count.fetch_add(1, Ordering::Relaxed)
    }

    #[cfg(not(target_has_atomic = "ptr"))]
    {
        unsafe {
            rn_freertos_c::vPortEnterCritical();
            let old = count.load(Ordering::Relaxed);
            count.store(old.wrapping_add(1), Ordering::Relaxed);
            rn_freertos_c::vPortExitCritical();
            old
        }
    }
}

/// Atomically `count -= 1`, returning the previous value.
#[inline]
fn refcount_fetch_sub(count: &AtomicUsize) -> usize {
    #[cfg(target_has_atomic = "ptr")]
    {
        count.fetch_sub(1, Ordering::Release)
    }

    #[cfg(not(target_has_atomic = "ptr"))]
    {
        unsafe {
            rn_freertos_c::vPortEnterCritical();
            let old = count.load(Ordering::Relaxed);
            count.store(old.wrapping_sub(1), Ordering::Relaxed);
            rn_freertos_c::vPortExitCritical();
            old
        }
    }
}

impl<T: ?Sized> Clone for Arc<T> {
    fn clone(&self) -> Self {
        // SAFETY: the reference count is at least 1, so incrementing is safe.
        let old = refcount_fetch_add(&self.inner().count);
        assert!(old < usize::MAX, "Arc::clone: reference count overflow");
        Self {
            ptr: self.ptr,
            _phantom: PhantomData,
        }
    }
}

impl<T: ?Sized> Deref for Arc<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.inner().data
    }
}

impl<T: ?Sized> Drop for Arc<T> {
    fn drop(&mut self) {
        // If we were the last reference, deallocate.
        if refcount_fetch_sub(&self.inner().count) == 1 {
            // We are the last reference.  Make sure every write made before
            // the last clone was released is visible before we free the
            // allocation (paired with the `Release` in refcount_fetch_sub).
            core::sync::atomic::fence(Ordering::Acquire);
            // SAFETY: we are the sole owner now.
            unsafe {
                let _ = Box::from_raw(self.ptr.as_ptr());
            }
        }
    }
}

// ---------------------------------------------------------------------------
//  Arc + Send / Sync helpers for unsized types
// ---------------------------------------------------------------------------

impl<T: ?Sized + core::fmt::Debug> core::fmt::Debug for Arc<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&**self, f)
    }
}

impl<T: ?Sized + core::fmt::Display> core::fmt::Display for Arc<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(&**self, f)
    }
}

impl<T: ?Sized + PartialEq> PartialEq for Arc<T> {
    fn eq(&self, other: &Self) -> bool {
        core::ptr::eq(self.ptr.as_ptr(), other.ptr.as_ptr()) || *(*self) == *(*other)
    }
}

impl<T: ?Sized + Eq> Eq for Arc<T> {}

impl<T: ?Sized + PartialOrd> PartialOrd for Arc<T> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        (**self).partial_cmp(&**other)
    }
}

impl<T: ?Sized + Ord> Ord for Arc<T> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        (**self).cmp(&**other)
    }
}

impl<T: ?Sized + core::hash::Hash> core::hash::Hash for Arc<T> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

/// `Arc` can be created from `Box` via `From`.
impl<T> From<T> for Arc<T> {
    fn from(t: T) -> Self {
        Arc::new(t)
    }
}

impl<T> From<Box<T>> for Arc<T> {
    fn from(boxed: Box<T>) -> Self {
        let inner = Box::new(ArcInner {
            count: AtomicUsize::new(1),
            data: *boxed,
        });
        let ptr = NonNull::new(Box::into_raw(inner))
            .expect("Arc::from(Box): Box::into_raw returned NULL");
        Self {
            ptr,
            _phantom: PhantomData,
        }
    }
}
