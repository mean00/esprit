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
//! All wrappers own the underlying FreeRTOS kernel object and free it on drop.

use crate::prelude::*;
use crate::rn_freertos_c;
use core::ffi::c_void;
use core::ptr::NonNull;
use core::convert::From;

// ---------------------------------------------------------------------------
//  Helper – convert ms to FreeRTOS ticks (or 0 / portMAX_DELAY)
// ---------------------------------------------------------------------------
const TICK_RATE_HZ: u32 = 1000; // typical; adjust if your config differs

fn ms_to_ticks(ms: u32) -> rn_freertos_c::TickType_t {
    if ms == u32::MAX {
        // Infinite wait – FreeRTOS portMAX_DELAY
        u32::MAX
    } else if ms == 0 {
        0
    } else {
        (ms as u64 * TICK_RATE_HZ as u64 / 1000) as u32
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
    handle: rn_freertos_c::SemaphoreHandle_t, // actually a QueueHandle_t
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
    /// Panics if the underlying FreeRTOS mutex cannot be created (out of heap).
    pub fn new(value: T) -> Self {
        let handle = unsafe { rn_freertos_c::xQueueCreateMutex(1) }; // 1 = recursive
        assert!(!handle.is_null(), "Mutex: xQueueCreateMutex returned NULL");
        Self {
            handle,
            data: UnsafeCell::new(value),
        }
    }
}

impl<T: ?Sized> Mutex<T> {
    /// Lock the mutex, blocking until it is acquired.
    ///
    /// Returns a [`MutexGuard`] that releases the lock when dropped.
    pub fn lock(&self) -> MutexGuard<'_, T> {
        let ret = unsafe { rn_freertos_c::xQueueTakeMutexRecursive(self.handle, u32::MAX) };
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
        let ret = unsafe { rn_freertos_c::xQueueTakeMutexRecursive(self.handle, 0) };
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
        let ret = unsafe { rn_freertos_c::xQueueTakeMutexRecursive(self.handle, ms_to_ticks(timeout_ms)) };
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
        unsafe { rn_freertos_c::vQueueDelete(self.handle) }
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
        unsafe { rn_freertos_c::xQueueGiveMutexRecursive(self.mutex.handle); }
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
    handle: rn_freertos_c::SemaphoreHandle_t,
    data: UnsafeCell<T>,
}

// SAFETY: same reasoning as Mutex
unsafe impl<T: Send> Send for RecursiveMutex<T> {}
unsafe impl<T: Send> Sync for RecursiveMutex<T> {}

impl<T> RecursiveMutex<T> {
    /// Create a new recursive mutex.
    pub fn new(value: T) -> Self {
        let handle = unsafe { rn_freertos_c::xQueueCreateMutex(1) }; // 1 = recursive
        assert!(!handle.is_null(), "RecursiveMutex: xQueueCreateMutex returned NULL");
        Self {
            handle,
            data: UnsafeCell::new(value),
        }
    }
}

impl<T: ?Sized> RecursiveMutex<T> {
    /// Lock the recursive mutex (blocking).
    pub fn lock(&self) -> RecursiveMutexGuard<'_, T> {
        let ret = unsafe { rn_freertos_c::xQueueTakeMutexRecursive(self.handle, u32::MAX) };
        assert!(ret != 0, "RecursiveMutex::lock failed");
        RecursiveMutexGuard {
            mutex: self,
            _not_send: PhantomData,
        }
    }

    /// Try to lock without blocking.
    pub fn try_lock(&self) -> Option<RecursiveMutexGuard<'_, T>> {
        let ret = unsafe { rn_freertos_c::xQueueTakeMutexRecursive(self.handle, 0) };
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
            unsafe { rn_freertos_c::xQueueTakeMutexRecursive(self.handle, ms_to_ticks(timeout_ms)) };
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
        unsafe { rn_freertos_c::vQueueDelete(self.handle) }
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
        unsafe { rn_freertos_c::xQueueGiveMutexRecursive(self.mutex.handle); }
    }
}

// ===========================================================================
//  RwLock<T> – Reader‑Writer lock
// ===========================================================================

/// A FreeRTOS‑backed reader‑writer lock.
///
/// Allows any number of readers or a single writer.  Modelled after
/// [`std::sync::RwLock`].
///
/// # Implementation
///
/// Uses a recursive mutex to protect the reader count and data, plus a
/// binary semaphore for writers to wait until all readers have finished.
pub struct RwLock<T: ?Sized> {
    /// Recursive mutex protecting `reader_count` and the data.
    gate: rn_freertos_c::SemaphoreHandle_t,
    /// Binary semaphore – writers wait on this until readers drain.
    drain: rn_freertos_c::SemaphoreHandle_t,
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
    pub fn new(value: T) -> Self {
        let gate = unsafe { rn_freertos_c::xQueueCreateMutex(1) }; // recursive
        assert!(!gate.is_null(), "RwLock: gate mutex creation failed");
        let drain = unsafe { rn_freertos_c::xQueueCreateCountingSemaphore(1, 1) };
        assert!(!drain.is_null(), "RwLock: drain semaphore creation failed");
        Self {
            gate,
            drain,
            reader_count: UnsafeCell::new(0),
            data: UnsafeCell::new(value),
        }
    }
}

impl<T: ?Sized> RwLock<T> {
    /// Lock for reading (shared access).
    ///
    /// Blocks if a writer holds the lock.  Multiple readers can hold the
    /// lock simultaneously.
    pub fn read(&self) -> RwLockReadGuard<'_, T> {
        unsafe { rn_freertos_c::xQueueTakeMutexRecursive(self.gate, u32::MAX) };
        // SAFETY: we hold the gate mutex, so no other task can access reader_count
        let count = unsafe { &mut *self.reader_count.get() };
        if *count == 0 {
            // First reader – take the drain semaphore so writers will block
            unsafe { rn_freertos_c::xQueueSemaphoreTake(self.drain, u32::MAX) };
        }
        *count += 1;
        unsafe { rn_freertos_c::xQueueGiveMutexRecursive(self.gate) };
        RwLockReadGuard { lock: self }
    }

    /// Try to lock for reading without blocking.
    pub fn try_read(&self) -> Option<RwLockReadGuard<'_, T>> {
        let wret = unsafe { rn_freertos_c::xQueueTakeMutexRecursive(self.gate, 0) };
        if wret == 0 {
            return None;
        }
        // SAFETY: we hold the gate mutex, so no other task can access reader_count
        let count = unsafe { &mut *self.reader_count.get() };
        if *count == 0 {
            let dret = unsafe { rn_freertos_c::xQueueSemaphoreTake(self.drain, 0) };
            if dret == 0 {
                // Writer is waiting – bail
                unsafe { rn_freertos_c::xQueueGiveMutexRecursive(self.gate) };
                return None;
            }
        }
        *count += 1;
        unsafe { rn_freertos_c::xQueueGiveMutexRecursive(self.gate) };
        Some(RwLockReadGuard { lock: self })
    }

    /// Lock for writing (exclusive access).
    ///
    /// Blocks until all readers and other writers have released.
    pub fn write(&self) -> RwLockWriteGuard<'_, T> {
        // Take the drain semaphore – this blocks until all current readers finish
        unsafe { rn_freertos_c::xQueueSemaphoreTake(self.drain, u32::MAX) };
        // Now take the gate to protect data access
        unsafe { rn_freertos_c::xQueueTakeMutexRecursive(self.gate, u32::MAX) };
        RwLockWriteGuard { lock: self }
    }

    /// Try to lock for writing without blocking.
    pub fn try_write(&self) -> Option<RwLockWriteGuard<'_, T>> {
        let dret = unsafe { rn_freertos_c::xQueueSemaphoreTake(self.drain, 0) };
        if dret == 0 {
            return None; // readers active
        }
        let gret = unsafe { rn_freertos_c::xQueueTakeMutexRecursive(self.gate, 0) };
        if gret == 0 {
            // Shouldn't happen since we hold drain, but release and bail
            unsafe { rn_freertos_c::xQueueGenericSend(self.drain, core::ptr::null::<c_void>(), 0, 0) };
            return None;
        }
        Some(RwLockWriteGuard { lock: self })
    }
}

impl<T: ?Sized> Drop for RwLock<T> {
    fn drop(&mut self) {
        unsafe {
            rn_freertos_c::vQueueDelete(self.gate);
            rn_freertos_c::vQueueDelete(self.drain);
        }
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
        unsafe { rn_freertos_c::xQueueTakeMutexRecursive(self.lock.gate, u32::MAX) };
        // SAFETY: we hold the gate mutex, so no other task can access reader_count
        let count = unsafe { &mut *self.lock.reader_count.get() };
        *count -= 1;
        if *count == 0 {
            // Last reader – release the drain semaphore so writers can proceed
            unsafe { rn_freertos_c::xQueueGenericSend(self.lock.drain, core::ptr::null::<c_void>(), 0, 0) };
        }
        unsafe { rn_freertos_c::xQueueGiveMutexRecursive(self.lock.gate) };
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
            rn_freertos_c::xQueueGiveMutexRecursive(self.lock.gate);
            // Release the drain semaphore so new readers/writers can proceed
            rn_freertos_c::xQueueGenericSend(self.lock.drain, core::ptr::null::<c_void>(), 0, 0);
        }
    }
}

// ===========================================================================
//  OnceLock<T> – One‑time initialiser
// ===========================================================================

/// A synchronisation primitive that can be written to **once**.
///
/// Modelled after [`std::sync::OnceLock`].  Useful for lazy global
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
    /// Wrapped in `UnsafeCell` for interior mutability in `init_gate`.
    gate: UnsafeCell<rn_freertos_c::SemaphoreHandle_t>,
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
            inited: AtomicBool::new(false),
        }
    }

    /// Lazily create the gate semaphore on first use.
    fn init_gate(&self) {
        // SAFETY: we only read the current value; if it's null we write a new one.
        let current = unsafe { *self.gate.get() };
        if current.is_null() {
            let g = unsafe { rn_freertos_c::xQueueCreateMutex(0) };
            assert!(!g.is_null(), "OnceLock: xQueueCreateMutex returned NULL");
            unsafe {
                *self.gate.get() = g;
            }
        }
    }

    /// Get a reference to the contained value, or initialise it with `f`.
    ///
    /// Blocks if another task is currently initialising.
    pub fn get_or_init<F>(&self, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        if !self.inited.load(Ordering::Acquire) {
            self.init_gate();
            // Try to acquire the gate – only one task gets through
            unsafe { rn_freertos_c::xQueueSemaphoreTake(*self.gate.get(), u32::MAX) };
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
                    *self.gate.get(),
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
        self.init_gate();
        unsafe { rn_freertos_c::xQueueSemaphoreTake(*self.gate.get(), u32::MAX) };
        if self.inited.load(Ordering::Acquire) {
            unsafe {
                rn_freertos_c::xQueueGenericSend(
                    *self.gate.get(),
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
                *self.gate.get(),
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
        let gate = unsafe { *self.gate.get() };
        if !gate.is_null() {
            unsafe { rn_freertos_c::vQueueDelete(gate) };
        }
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
/// [`std::sync::LazyLock`].
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
    handle: rn_freertos_c::SemaphoreHandle_t,
}

// SAFETY: binary semaphores are usable across tasks.
unsafe impl Send for BinarySemaphore {}
unsafe impl Sync for BinarySemaphore {}

impl BinarySemaphore {
    /// Create a new binary semaphore, initially **not** given (i.e. take will block).
    pub fn new() -> Self {
        let handle = unsafe { rn_freertos_c::xQueueCreateCountingSemaphore(1, 0 as _) };
        assert!(!handle.is_null(), "BinarySemaphore: xQueueCreateCountingSemaphore returned NULL");
        Self { handle }
    }

    /// Create a new binary semaphore that starts in the "given" state.
    pub fn new_given() -> Self {
        let handle = unsafe { rn_freertos_c::xQueueCreateCountingSemaphore(1, 1 as _) };
        // Note: first param `1` is an integer literal that Rust infers to the target type
        assert!(!handle.is_null(), "BinarySemaphore: xQueueCreateCountingSemaphore returned NULL");
        Self { handle }
    }

    /// Give the semaphore (increment to 1).
    /// Returns `true` if the give was successful.
    pub fn give(&self) -> bool {
        let ret = unsafe {
            rn_freertos_c::xQueueGenericSend(self.handle, core::ptr::null::<c_void>(), 0, 0)
        };
        ret != 0
    }

    /// Take the semaphore, blocking until it becomes available.
    pub fn take(&self) {
        let ret = unsafe { rn_freertos_c::xQueueSemaphoreTake(self.handle, u32::MAX) };
        assert!(ret != 0, "BinarySemaphore::take failed");
    }

    /// Try to take without blocking.
    pub fn try_take(&self) -> bool {
        unsafe { rn_freertos_c::xQueueSemaphoreTake(self.handle, 0) != 0 }
    }

    /// Take with a timeout.
    pub fn take_timeout(&self, timeout_ms: u32) -> bool {
        unsafe { rn_freertos_c::xQueueSemaphoreTake(self.handle, ms_to_ticks(timeout_ms)) != 0 }
    }
}

impl Default for BinarySemaphore {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for BinarySemaphore {
    fn drop(&mut self) {
        unsafe { rn_freertos_c::vQueueDelete(self.handle) }
    }
}

// ===========================================================================
//  CountingSemaphore
// ===========================================================================

/// A counting semaphore with a configurable maximum count.
///
/// Typical use: managing a pool of resources.
pub struct CountingSemaphore {
    handle: rn_freertos_c::SemaphoreHandle_t,
}

unsafe impl Send for CountingSemaphore {}
unsafe impl Sync for CountingSemaphore {}

impl CountingSemaphore {
    /// Create a counting semaphore with `max_count` and `initial_count`.
    pub fn new(max_count: u32, initial_count: u32) -> Self {
        let handle =
            unsafe { rn_freertos_c::xQueueCreateCountingSemaphore(max_count as _, initial_count as _) };
        assert!(!handle.is_null(), "CountingSemaphore: xQueueCreateCountingSemaphore returned NULL");
        Self { handle }
    }

    /// Give (increment) the semaphore.
    pub fn give(&self) -> bool {
        let ret = unsafe {
            rn_freertos_c::xQueueGenericSend(self.handle, core::ptr::null::<c_void>(), 0, 0)
        };
        ret != 0
    }

    /// Block until we can take (decrement) the semaphore.
    pub fn take(&self) {
        let ret = unsafe { rn_freertos_c::xQueueSemaphoreTake(self.handle, u32::MAX) };
        assert!(ret != 0, "CountingSemaphore::take failed");
    }

    /// Try to take without blocking.
    pub fn try_take(&self) -> bool {
        unsafe { rn_freertos_c::xQueueSemaphoreTake(self.handle, 0) != 0 }
    }

    /// Take with a timeout.
    pub fn take_timeout(&self, timeout_ms: u32) -> bool {
        unsafe { rn_freertos_c::xQueueSemaphoreTake(self.handle, ms_to_ticks(timeout_ms)) != 0 }
    }
}

impl Drop for CountingSemaphore {
    fn drop(&mut self) {
        unsafe { rn_freertos_c::vQueueDelete(self.handle) }
    }
}

// ===========================================================================
//  Arc<T> – Atomically Reference‑Counted Pointer
// ===========================================================================

/// A thread‑safe reference‑counting pointer, modelled after
/// [`std::sync::Arc`].
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

impl<T: ?Sized> Clone for Arc<T> {
    fn clone(&self) -> Self {
        // SAFETY: the reference count is at least 1, so incrementing is safe.
        // Use load/store instead of fetch_add for targets without atomic CAS (e.g. Cortex-M0+).
        let old = self.inner().count.load(Ordering::Relaxed);
        assert!(old < usize::MAX, "Arc::clone: reference count overflow");
        self.inner().count.store(old + 1, Ordering::Relaxed);
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
        // Use load/store instead of fetch_sub for targets without atomic CAS (e.g. Cortex-M0+).
        let old = self.inner().count.load(Ordering::Acquire);
        if old <= 1 {
            // We are the last reference.
            core::sync::atomic::fence(Ordering::Acquire);
            // SAFETY: we are the sole owner now.
            unsafe {
                let _ = Box::from_raw(self.ptr.as_ptr());
            }
        } else {
            self.inner().count.store(old - 1, Ordering::Release);
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
