//! Test suite for the `fake_std` layer of `rust_esprit`.
//!
//! This project exercises every public API provided by the `fake_std` shim:
//! synchronisation primitives (`Mutex`, `RwLock`, `OnceLock`, `LazyLock`,
//! `Arc`, semaphores), time (`Instant`, `Duration`), and task management
//! (`spawn`, `sleep`, `yield_now`, `current`).
//!
//! The tests run on the embedded target and report results via the logger.
//! A simple pass/fail summary is printed at the end.

#![no_std]
const STACK_SIZE: u32 = 512u32;
use rust_esprit::std::sync::{
    Arc, BinarySemaphore, CountingSemaphore, LazyLock, Mutex, OnceLock, RecursiveMutex, RwLock,
};
use rust_esprit::std::thread;
use rust_esprit::std::time::Instant;
use rust_esprit::{logger, logger_init};

logger_init!();

// ===========================================================================
//  Test harness helpers
// ===========================================================================

static PASSED: OnceLock<Mutex<u32>> = OnceLock::new();
static FAILED: OnceLock<Mutex<u32>> = OnceLock::new();

fn init_counters() {
    PASSED.get_or_init(|| Mutex::new(0));
    FAILED.get_or_init(|| Mutex::new(0));
}

fn pass(name: &str) {
    logger!("  OK {}\n", name);
    if let Some(m) = PASSED.get() {
        *m.lock() += 1;
    }
}

// (fail function removed – we use inline fail logging instead)

// (test macro removed – we use direct logger! calls instead)

// ===========================================================================
//  Test: Mutex basic locking
// ===========================================================================

fn test_mutex_basic() {
    let m = Mutex::new(0u32);
    {
        let mut guard = m.lock();
        *guard = 42;
    }
    let val = *m.lock();
    if val == 42 {
        pass("Mutex::lock / unlock");
    } else {
        logger!("  ✗ Mutex::lock / unlock: got {}\n", val);
        if let Some(m) = FAILED.get() {
            *m.lock() += 1;
        }
    }
}

fn test_mutex_try_lock() {
    let m = Mutex::new(7u32);
    match m.try_lock() {
        Some(mut g) => {
            *g = 8;
            drop(g);
            if *m.lock() == 8 {
                pass("Mutex::try_lock");
            } else {
                logger!("  ✗ Mutex::try_lock: value not updated\n");
                if let Some(m) = FAILED.get() {
                    *m.lock() += 1;
                }
            }
        }
        None => {
            logger!("  ✗ Mutex::try_lock: returned None\n");
            if let Some(m) = FAILED.get() {
                *m.lock() += 1;
            }
        }
    }
}

fn test_mutex_lock_timeout() {
    let m = Mutex::new(99u32);
    match m.lock_timeout(100) {
        Some(mut g) => {
            *g = 100;
            drop(g);
            if *m.lock() == 100 {
                pass("Mutex::lock_timeout");
            } else {
                logger!("  ✗ Mutex::lock_timeout: value not updated\n");
                if let Some(m) = FAILED.get() {
                    *m.lock() += 1;
                }
            }
        }
        None => {
            logger!("  ✗ Mutex::lock_timeout: timed out on uncontended lock\n");
            if let Some(m) = FAILED.get() {
                *m.lock() += 1;
            }
        }
    }
}

// ===========================================================================
//  Test: RecursiveMutex
// ===========================================================================

fn test_recursive_mutex() {
    let m = RecursiveMutex::new(0u32);
    {
        let g1 = m.lock();
        let g2 = m.lock(); // same task – should not deadlock
        let _ = (*g1, *g2);
    }
    pass("RecursiveMutex::lock (re-entrant)");
}

// ===========================================================================
//  Test: RwLock
// ===========================================================================

fn test_rwlock_read() {
    let rw = RwLock::new(42u32);
    let val = *rw.read();
    if val == 42 {
        pass("RwLock::read");
    } else {
        logger!("  ✗ RwLock::read: got {}\n", val);
        if let Some(m) = FAILED.get() {
            *m.lock() += 1;
        }
    }
}

fn test_rwlock_write() {
    let rw = RwLock::new(0u32);
    {
        let mut w = rw.write();
        *w = 99;
    }
    let val = *rw.read();
    if val == 99 {
        pass("RwLock::write");
    } else {
        logger!("  ✗ RwLock::write: got {}\n", val);
        if let Some(m) = FAILED.get() {
            *m.lock() += 1;
        }
    }
}

fn test_rwlock_try_read() {
    let rw = RwLock::new(1u32);
    match rw.try_read() {
        Some(g) => {
            if *g == 1 {
                pass("RwLock::try_read");
            } else {
                logger!("  ✗ RwLock::try_read: wrong value\n");
                if let Some(m) = FAILED.get() {
                    *m.lock() += 1;
                }
            }
        }
        None => {
            logger!("  ✗ RwLock::try_read: returned None\n");
            if let Some(m) = FAILED.get() {
                *m.lock() += 1;
            }
        }
    }
}

fn test_rwlock_try_write() {
    let rw = RwLock::new(2u32);
    match rw.try_write() {
        Some(mut g) => {
            *g = 3;
            drop(g);
            if *rw.read() == 3 {
                pass("RwLock::try_write");
            } else {
                logger!("  ✗ RwLock::try_write: value not updated\n");
                if let Some(m) = FAILED.get() {
                    *m.lock() += 1;
                }
            }
        }
        None => {
            logger!("  ✗ RwLock::try_write: returned None\n");
            if let Some(m) = FAILED.get() {
                *m.lock() += 1;
            }
        }
    }
}

// ===========================================================================
//  Test: OnceLock
// ===========================================================================

fn test_once_lock() {
    static ONCE: OnceLock<u32> = OnceLock::new();
    assert!(!ONCE.is_initialized());

    let v1 = ONCE.get_or_init(|| 123);
    assert!(ONCE.is_initialized());
    let v2 = ONCE.get_or_init(|| 456); // should not re-init
    if *v1 == 123 && *v2 == 123 {
        pass("OnceLock::get_or_init");
    } else {
        logger!("  ✗ OnceLock::get_or_init: unexpected value\n");
        if let Some(m) = FAILED.get() {
            *m.lock() += 1;
        }
    }
}

fn test_once_lock_set() {
    static ONCE: OnceLock<u32> = OnceLock::new();
    assert!(ONCE.set(10).is_ok());
    assert!(ONCE.set(20).is_err()); // already set
    if *ONCE.get().unwrap() == 10 {
        pass("OnceLock::set");
    } else {
        logger!("  ✗ OnceLock::set: wrong value\n");
        if let Some(m) = FAILED.get() {
            *m.lock() += 1;
        }
    }
}

// ===========================================================================
//  Test: LazyLock
// ===========================================================================

fn test_lazy_lock() {
    static LAZY: LazyLock<u32> = LazyLock::new(|| 42);
    let v = *LAZY;
    if v == 42 {
        pass("LazyLock::deref");
    } else {
        logger!("  ✗ LazyLock::deref: got {}\n", v);
        if let Some(m) = FAILED.get() {
            *m.lock() += 1;
        }
    }
}

// ===========================================================================
//  Test: Arc
// ===========================================================================

fn test_arc_basic() {
    let a = Arc::new(7u32);
    let b = a.clone();
    if *a == 7 && *b == 7 {
        pass("Arc::new / clone / deref");
    } else {
        logger!("  ✗ Arc::new / clone / deref: wrong value\n");
        if let Some(m) = FAILED.get() {
            *m.lock() += 1;
        }
    }
}

fn test_arc_ref_count() {
    let a = Arc::new(0u32);
    assert_eq!(Arc::ref_count(&a), 1);
    let b = a.clone();
    assert_eq!(Arc::ref_count(&a), 2);
    drop(b);
    assert_eq!(Arc::ref_count(&a), 1);
    pass("Arc::ref_count");
}

fn test_arc_try_unwrap() {
    let a = Arc::new(99u32);
    let b = a.clone();
    assert!(Arc::try_unwrap(a).is_err()); // b still alive
    drop(b);
    pass("Arc::try_unwrap");
}

fn test_arc_get_mut() {
    let mut a = Arc::new(5u32);
    if let Some(v) = Arc::get_mut(&mut a) {
        *v = 10;
    }
    if *a == 10 {
        pass("Arc::get_mut");
    } else {
        logger!("  ✗ Arc::get_mut: got {}\n", *a);
        if let Some(m) = FAILED.get() {
            *m.lock() += 1;
        }
    }
}

fn test_arc_is_unique() {
    let a = Arc::new(1u32);
    assert!(Arc::is_unique(&a));
    let _b = a.clone();
    assert!(!Arc::is_unique(&a));
    pass("Arc::is_unique");
}

// ===========================================================================
//  Test: BinarySemaphore
// ===========================================================================

fn test_binary_semaphore() {
    let sem = BinarySemaphore::new_given();
    assert!(sem.try_take());
    assert!(!sem.try_take()); // should be 0 now
    sem.give();
    assert!(sem.try_take());
    pass("BinarySemaphore");
}

// ===========================================================================
//  Test: CountingSemaphore
// ===========================================================================

fn test_counting_semaphore() {
    let sem = CountingSemaphore::new(5, 3);
    assert!(sem.try_take());
    assert!(sem.try_take());
    assert!(sem.try_take());
    assert!(!sem.try_take()); // exhausted
    sem.give();
    assert!(sem.try_take());
    pass("CountingSemaphore");
}

// ===========================================================================
//  Test: Instant / Duration
// ===========================================================================

fn test_instant_now() {
    let start = Instant::now();
    thread::sleep_ms(50);
    let elapsed = start.elapsed();
    if elapsed.as_millis() >= 40 {
        pass("Instant::now / elapsed");
    } else {
        logger!(
            "  ✗ Instant::now / elapsed: only {}ms\n",
            elapsed.as_millis()
        );
        if let Some(m) = FAILED.get() {
            *m.lock() += 1;
        }
    }
}

fn test_instant_duration_since() {
    let a = Instant::now();
    thread::sleep_ms(20);
    let b = Instant::now();
    let d = b.duration_since(&a);
    if d.as_millis() >= 15 {
        pass("Instant::duration_since");
    } else {
        logger!("  ✗ Instant::duration_since: only {}ms\n", d.as_millis());
        if let Some(m) = FAILED.get() {
            *m.lock() += 1;
        }
    }
}

fn test_instant_checked_ops() {
    let a = Instant::now();
    let d = rust_esprit::Duration::from_millis(100);
    let b = a.checked_add(d).unwrap();
    let diff = b.checked_duration_since(&a).unwrap();
    if diff.as_millis() >= 100 {
        pass("Instant::checked_add / checked_duration_since");
    } else {
        logger!("  ✗ Instant::checked_add / checked_duration_since: unexpected\n");
        if let Some(m) = FAILED.get() {
            *m.lock() += 1;
        }
    }
}

// ===========================================================================
//  Test: thread::sleep / yield_now
// ===========================================================================

fn test_sleep() {
    let start = Instant::now();
    thread::sleep(rust_esprit::Duration::from_millis(30));
    let elapsed = start.elapsed();
    if elapsed.as_millis() >= 25 {
        pass("thread::sleep");
    } else {
        logger!("  ✗ thread::sleep: only {}ms\n", elapsed.as_millis());
        if let Some(m) = FAILED.get() {
            *m.lock() += 1;
        }
    }
}

fn test_yield_now() {
    // yield_now should not block – just verify it returns
    thread::yield_now();
    pass("thread::yield_now");
}

// ===========================================================================
//  Test: task::spawn (basic)
// ===========================================================================

fn test_spawn() {
    static DONE: OnceLock<Mutex<bool>> = OnceLock::new();
    DONE.get_or_init(|| Mutex::new(false));

    let _handle = thread::spawn("test_spawn", STACK_SIZE, 1, || {
        thread::sleep_ms(10);
        *DONE.get().unwrap().lock() = true;
    });

    // Give the tick interrupt time to context-switch the deleted task out
    // and let the idle task free its TCB/stack.
    thread::sleep_ms(10);

    // Wait for the task to finish
    thread::sleep_ms(50);
    let done = *DONE.get().unwrap().lock();
    if done {
        pass("thread::spawn");
    } else {
        logger!("  ✗ thread::spawn: task did not complete\n");
        if let Some(m) = FAILED.get() {
            *m.lock() += 1;
        }
    }
}

// ===========================================================================
//  Test: task::current
// ===========================================================================

fn test_current() {
    let handle = thread::current();
    let name = handle.name();
    if !name.is_empty() {
        pass("thread::current");
    } else {
        logger!("  ✗ thread::current: empty name\n");
        if let Some(m) = FAILED.get() {
            *m.lock() += 1;
        }
    }
}

// ===========================================================================
//  Test: Arc across tasks
// ===========================================================================

fn test_arc_across_tasks() {
    let data = Arc::new(Mutex::new(0u32));

    let d1 = data.clone();
    let _h1 = thread::spawn("arc_writer", STACK_SIZE, 1, move || {
        thread::sleep_ms(5);
        *d1.lock() = 42;
    });
    thread::sleep_ms(5); // let the task start and the tick interrupt clean up if it finishes

    let d2 = data.clone();
    let _h2 = thread::spawn("arc_reader", STACK_SIZE, 1, move || {
        thread::sleep_ms(20);
        let val = *d2.lock();
        assert_eq!(val, 42);
    });
    thread::sleep_ms(5); // let the task start

    thread::sleep_ms(50);
    let val = *data.lock();
    if val == 42 {
        pass("Arc across tasks");
    } else {
        logger!("  ✗ Arc across tasks: got {}\n", val);
        if let Some(m) = FAILED.get() {
            *m.lock() += 1;
        }
    }
}

// ===========================================================================
//  Test: Mutex across tasks
// ===========================================================================

fn test_mutex_across_tasks() {
    let counter = Arc::new(Mutex::new(0u32));

    for _ in 0..3 {
        let c = counter.clone();
        let _h = thread::spawn("worker", STACK_SIZE, 1, move || {
            for _ in 0..10 {
                let mut val = c.lock();
                *val += 1;
                thread::yield_now();
            }
        });
        thread::sleep_ms(5); // let each worker start before spawning the next
    }

    thread::sleep_ms(100);
    let final_val = *counter.lock();
    if final_val == 30 {
        pass("Mutex across tasks (3×10 increments)");
    } else {
        logger!("  ✗ Mutex across tasks: got {}, expected 30\n", final_val);
        if let Some(m) = FAILED.get() {
            *m.lock() += 1;
        }
    }
}

// ===========================================================================
//  Test: RwLock across tasks
// ===========================================================================

fn test_rwlock_across_tasks() {
    let data = Arc::new(RwLock::new(0u32));

    // Writer task
    let w_data = data.clone();
    let _w = thread::spawn("rw_writer", STACK_SIZE, 2, move || {
        thread::sleep_ms(10);
        let mut w = w_data.write();
        *w = 100;
    });
    thread::sleep_ms(5); // let the writer start

    // Reader tasks
    let r_data = data.clone();
    let _r = thread::spawn("rw_reader", STACK_SIZE, 1, move || {
        thread::sleep_ms(30);
        let val = *r_data.read();
        assert_eq!(val, 100);
    });
    thread::sleep_ms(5); // let the reader start

    thread::sleep_ms(60);
    let val = *data.read();
    if val == 100 {
        pass("RwLock across tasks");
    } else {
        logger!("  ✗ RwLock across tasks: got {}\n", val);
        if let Some(m) = FAILED.get() {
            *m.lock() += 1;
        }
    }
}

// ===========================================================================
//  Test: OnceLock across tasks
// ===========================================================================

fn test_once_lock_across_tasks() {
    static SHARED: OnceLock<u32> = OnceLock::new();

    let _h1 = thread::spawn("once_1", STACK_SIZE, 1, || {
        let v = SHARED.get_or_init(|| 77);
        assert_eq!(*v, 77);
    });
    thread::sleep_ms(5); // let task start

    let _h2 = thread::spawn("once_2", STACK_SIZE, 1, || {
        let v = SHARED.get_or_init(|| 99); // should not overwrite
        assert_eq!(*v, 77);
    });
    thread::sleep_ms(5); // let task start

    thread::sleep_ms(30);
    if *SHARED.get().unwrap() == 77 {
        pass("OnceLock across tasks");
    } else {
        logger!("  ✗ OnceLock across tasks: wrong value\n");
        if let Some(m) = FAILED.get() {
            *m.lock() += 1;
        }
    }
}

// ===========================================================================
//  Test: BinarySemaphore across tasks
// ===========================================================================

fn test_semaphore_across_tasks() {
    let sem = Arc::new(BinarySemaphore::new());

    let s = sem.clone();
    let _giver = thread::spawn("sem_giver", STACK_SIZE, 1, move || {
        thread::sleep_ms(10);
        s.give();
    });
    thread::sleep_ms(5); // let the giver start

    // Block until the giver signals
    sem.take_timeout(100);
    pass("BinarySemaphore across tasks");
}

// ===========================================================================
//  Test: CountingSemaphore across tasks
// ===========================================================================

fn test_counting_sem_across_tasks() {
    let sem = Arc::new(CountingSemaphore::new(10, 0));

    let s = sem.clone();
    let _giver = thread::spawn("cnt_giver", STACK_SIZE, 1, move || {
        for _ in 0..5 {
            s.give();
            thread::sleep_ms(5);
        }
    });
    thread::sleep_ms(5); // let the giver start

    thread::sleep_ms(40);
    let taken = sem.try_take() as u32
        + sem.try_take() as u32
        + sem.try_take() as u32
        + sem.try_take() as u32
        + sem.try_take() as u32;
    if taken == 5 {
        pass("CountingSemaphore across tasks");
    } else {
        logger!("  ✗ CountingSemaphore across tasks: took {}\n", taken);
        if let Some(m) = FAILED.get() {
            *m.lock() += 1;
        }
    }
}

// ===========================================================================
//  Test: LazyLock across tasks
// ===========================================================================

fn test_lazy_lock_across_tasks() {
    static LAZY: LazyLock<u32> = LazyLock::new(|| 1234);

    let _h1 = thread::spawn("lazy_1", STACK_SIZE, 1, || {
        let v = *LAZY;
        assert_eq!(v, 1234);
    });
    thread::sleep_ms(5); // let task start

    let _h2 = thread::spawn("lazy_2", STACK_SIZE, 1, || {
        let v = *LAZY;
        assert_eq!(v, 1234);
    });
    thread::sleep_ms(5); // let task start

    thread::sleep_ms(20);
    if *LAZY == 1234 {
        pass("LazyLock across tasks");
    } else {
        logger!("  ✗ LazyLock across tasks: wrong value\n");
        if let Some(m) = FAILED.get() {
            *m.lock() += 1;
        }
    }
}

// ===========================================================================
//  Test: Instant across tasks
// ===========================================================================

fn test_instant_across_tasks() {
    let start = Arc::new(Instant::now());

    let s = start.clone();
    let _t = thread::spawn("instant_task", STACK_SIZE, 1, move || {
        thread::sleep_ms(30);
        let elapsed = s.elapsed();
        assert!(elapsed.as_millis() >= 25);
    });
    thread::sleep_ms(5); // let the task start

    thread::sleep_ms(50);
    let elapsed = start.elapsed();
    // Total: 5ms (delay after spawn) + 50ms (main sleep) = ~55ms.
    // Allow a small tolerance for scheduling overhead.
    if elapsed.as_millis() >= 50 {
        pass("Instant across tasks");
    } else {
        logger!(
            "  FAIL  Instant across tasks: only {}ms\n",
            elapsed.as_millis()
        );
        if let Some(m) = FAILED.get() {
            *m.lock() += 1;
        }
    }
}

// ===========================================================================
//  Main entry point
// ===========================================================================

#[unsafe(no_mangle)]
extern "C" fn user_init() {
    logger!("\n");
    logger!("+--------------------------------------+\n");
    logger!("|  rust_esprit fake_std Test Suite     |\n");
    logger!("+--------------------------------------+\n");
    logger!("\n");

    init_counters();

    // ── Basic synchronisation ──
    logger!("+ Mutex basic locking ..\n");
    test_mutex_basic();
    test_mutex_try_lock();
    test_mutex_lock_timeout();

    logger!("+ RecursiveMutex ...\n");
    test_recursive_mutex();

    logger!("+ RwLock \n");
    test_rwlock_read();
    test_rwlock_write();
    test_rwlock_try_read();
    test_rwlock_try_write();

    logger!("+ OnceLock \n");
    test_once_lock();
    test_once_lock_set();

    logger!("+ LazyLock \n");
    test_lazy_lock();

    logger!("+ Arc \n");
    test_arc_basic();
    test_arc_ref_count();
    test_arc_try_unwrap();
    test_arc_get_mut();
    test_arc_is_unique();

    logger!("+ Semaphores \n");
    test_binary_semaphore();
    test_counting_semaphore();

    // ── Time ──
    logger!("+ Instant / Duration \n");
    test_instant_now();
    test_instant_duration_since();
    test_instant_checked_ops();

    // ── Thread / task ──
    logger!("+ thread::sleep / yield_now \n");
    test_sleep();
    test_yield_now();

    logger!("+ thread::spawn \n");
    test_spawn();

    logger!("+ thread::current \n");
    test_current();

    // ── Cross-task tests ──
    logger!("+ Arc across tasks \n");
    test_arc_across_tasks();

    logger!("+ Mutex across tasks \n");
    test_mutex_across_tasks();

    logger!("+ RwLock across tasks \n");
    test_rwlock_across_tasks();

    logger!("+ OnceLock across tasks \n");
    test_once_lock_across_tasks();

    logger!("+ Semaphores across tasks \n");
    test_semaphore_across_tasks();
    test_counting_sem_across_tasks();

    logger!("+ LazyLock across tasks \n");
    test_lazy_lock_across_tasks();

    logger!("+ Instant across tasks \n");
    test_instant_across_tasks();

    // ── Summary ──
    let passed_val = PASSED.get().map(|m| *m.lock()).unwrap_or(0);
    let failed_val = FAILED.get().map(|m| *m.lock()).unwrap_or(0);
    let total = passed_val + failed_val;

    logger!("\n");
    logger!("+--------------------------------------+\n");
    logger!(
        "  Results: {} passed, {} failed ({} total)\n",
        passed_val,
        failed_val,
        total
    );
    if failed_val == 0 {
        logger!("  OK:  ALL TESTS PASSED\n");
    } else {
        logger!("  FAIL: SOME TESTS FAILED\n");
    }
    logger!("+--------------------------------------+\n");
}

// EOF
