#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clashing_extern_declarations)]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(unused_imports)]

#[cfg(not(feature = "external_std"))]
extern crate alloc;

pub mod prelude {
    pub use core::cell::UnsafeCell;
    pub use core::convert::Infallible;
    pub use core::marker::PhantomData;
    pub use core::mem;
    pub use core::ops::{Deref, DerefMut};
    pub use core::ptr;
    pub use core::slice;
    pub use core::str;
    pub use core::time::Duration;

    #[cfg(not(feature = "external_std"))]
    pub use core::alloc::{GlobalAlloc, Layout};
    #[cfg(not(feature = "external_std"))]
    pub use core::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering};
    #[cfg(not(feature = "external_std"))]
    pub use alloc::boxed::Box;
    #[cfg(not(feature = "external_std"))]
    pub use alloc::string::String;
    #[cfg(not(feature = "external_std"))]
    pub use alloc::vec::Vec;

    #[cfg(feature = "external_std")]
    pub use std::alloc::{GlobalAlloc, Layout};
    #[cfg(feature = "external_std")]
    pub use std::boxed::Box;
    #[cfg(feature = "external_std")]
    pub use std::string::String;
    #[cfg(feature = "external_std")]
    pub use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering};
    #[cfg(feature = "external_std")]
    pub use std::vec::Vec;
}

pub mod c_freertos;
pub mod c_fast_event;

pub mod sync;
pub mod queue;
pub mod event;
pub mod task;

pub use sync::{
    Arc, BinarySemaphore, CountingSemaphore, LazyLock, Mutex, MutexGuard, OnceLock,
    RecursiveMutex, RecursiveMutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
    SemaphoreGuard,
};
pub use queue::Queue;
pub use event::EventGroup;
pub use task::{
    current, delay_ms, delay_us, sleep, sleep_ms, spawn, spawn_raw, tick_count, time_ms, time_us,
    time_us64, yield_now, Duration, Instant, TaskEntry, TaskHandle, ms_to_ticks,
};
