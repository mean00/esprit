//! Handler registration, one slot per peripheral **instance**.
//!
//! Several instances of the same peripheral coexist (UART0..UART4, SPI0..SPI2,
//! ADC0..ADC1, one EXTI slot per pin), and each needs its own handler: a probe
//! bridge on UART0 and the logger on UART1 must not call the same `on_rx`.
//!
//! A family crate therefore owns one table and one instance count:
//!
//! ```ignore
//! static HANDLERS: HandlerTable<dyn UartHandler, UART_INSTANCES> = HandlerTable::new();
//! // in Uart::set_handler:  HANDLERS.set(self.index, handler);
//! // in the RX interrupt:   if let Some(h) = unsafe { HANDLERS.isr_get(self.index) } { h.on_rx(byte) }
//! ```
//!
//! The table is lock-free on purpose: it is read from interrupts, where a
//! critical section is either illegal or very expensive. The write side is a
//! single handler pointer, so [`HandlerCell::set`] is documented to be called
//! from task context.

use core::cell::UnsafeCell;

/// One `Option<&'static T>` that can live in a `static` and be read from an ISR.
///
/// `T` is `?Sized` because the slots hold `&'static dyn XHandler`.
pub struct HandlerCell<T: ?Sized + 'static> {
    slot: UnsafeCell<Option<&'static T>>,
}

// SAFETY: the only writer is `set`, which the contract below restricts to task
// context while the peripheral is idle; readers take a copy of a `&'static`
// reference and may run in any context.
unsafe impl<T: ?Sized + 'static> Sync for HandlerCell<T> {}

impl<T: ?Sized + 'static> HandlerCell<T> {
    /// An empty slot, usable in a `static`.
    pub const fn new() -> Self {
        Self {
            slot: UnsafeCell::new(None),
        }
    }

    /// Install (or clear) the handler.
    ///
    /// # Contract
    ///
    /// Call from task context, with the peripheral's interrupt disabled or the
    /// peripheral idle: the slot is a fat pointer (data + vtable), so it is
    /// written in two words and an interrupt taken in between would see a mix of
    /// the old vtable and the new data. A driver arms its handler *before*
    /// enabling the interrupt, which is exactly that contract.
    pub fn set(&self, handler: Option<&'static T>) {
        // SAFETY: unique writer by the contract above; no reference derived from
        // the slot is alive across this store because readers copy the value out.
        unsafe { *self.slot.get() = handler };
    }

    /// The installed handler, if any.
    pub fn get(&self) -> Option<&'static T> {
        // SAFETY: a `&'static T` is `Copy` and stays valid for the program, so
        // copying it out cannot dangle, even if another context clears the slot
        // right after.
        unsafe { *self.slot.get() }
    }

    /// [`get`](Self::get) for an interrupt handler.
    ///
    /// # Safety
    ///
    /// The caller must be an interrupt that cannot observe a partially written
    /// slot: the slot's owner wrote it before enabling this interrupt.
    pub unsafe fn isr_get(&self) -> Option<&'static T> {
        // SAFETY: the caller guarantees the write happened-before the interrupt.
        unsafe { *self.slot.get() }
    }
}

impl<T: ?Sized + 'static> Default for HandlerCell<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// `N` handler slots, one per peripheral instance (`N` is the family's instance
/// count, e.g. 5 UARTs or 30 GPIO interrupts).
pub struct HandlerTable<T: ?Sized + 'static, const N: usize> {
    slots: [HandlerCell<T>; N],
}

impl<T: ?Sized + 'static, const N: usize> HandlerTable<T, N> {
    /// An empty table, usable in a `static`.
    pub const fn new() -> Self {
        Self {
            slots: [const { HandlerCell::new() }; N],
        }
    }

    /// Number of instances this table covers.
    pub const fn len(&self) -> usize {
        N
    }

    /// Always `false` (a table has `N` slots); present so the usual lint pair is
    /// satisfied.
    pub const fn is_empty(&self) -> bool {
        N == 0
    }

    /// Install the handler of instance `index`; out of range is a no-op, so a
    /// driver cannot corrupt a neighbouring instance by arithmetic.
    pub fn set(&self, index: usize, handler: Option<&'static T>) {
        if let Some(slot) = self.slots.get(index) {
            slot.set(handler);
        }
    }

    /// The handler of instance `index`.
    pub fn get(&self, index: usize) -> Option<&'static T> {
        self.slots.get(index).and_then(HandlerCell::get)
    }

    /// [`get`](Self::get) for an interrupt handler, see
    /// [`HandlerCell::isr_get`].
    ///
    /// # Safety
    ///
    /// Same contract as [`HandlerCell::isr_get`]: called from the interrupt of
    /// that instance only.
    pub unsafe fn isr_get(&self, index: usize) -> Option<&'static T> {
        match self.slots.get(index) {
            // SAFETY: forwarded to the caller's guarantee.
            Some(slot) => unsafe { slot.isr_get() },
            None => None,
        }
    }
}

impl<T: ?Sized + 'static, const N: usize> Default for HandlerTable<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    trait Marker: Sync {
        fn id(&self) -> u8;
    }

    struct Zero;
    struct One;

    impl Marker for Zero {
        fn id(&self) -> u8 {
            0
        }
    }
    impl Marker for One {
        fn id(&self) -> u8 {
            1
        }
    }

    static A: Zero = Zero;
    static B: One = One;
    static TABLE: HandlerTable<dyn Marker, 2> = HandlerTable::new();

    /// Two instances keep two different handlers: that is the whole point of the
    /// table (UART0 for the bridge, UART1 for the logger).
    #[test]
    fn instances_have_independent_handlers() {
        TABLE.set(0, Some(&A));
        TABLE.set(1, Some(&B));

        assert_eq!(TABLE.get(0).map(|h| h.id()), Some(0));
        assert_eq!(TABLE.get(1).map(|h| h.id()), Some(1));
    }

    /// An out-of-range instance is ignored instead of aliasing slot 0.
    #[test]
    fn out_of_range_index_is_a_no_op() {
        TABLE.set(0, Some(&A));
        TABLE.set(2, Some(&B));
        TABLE.set(usize::MAX, None);

        assert_eq!(TABLE.get(0).map(|h| h.id()), Some(0));
        assert!(TABLE.get(2).is_none());
    }

    /// Clearing one slot leaves the other instance alone.
    #[test]
    fn clearing_one_slot_keeps_the_other() {
        TABLE.set(0, Some(&A));
        TABLE.set(1, Some(&B));
        TABLE.set(0, None);

        assert!(TABLE.get(0).is_none());
        assert_eq!(TABLE.get(1).map(|h| h.id()), Some(1));
    }

    /// The interrupt view sees what the task installed.
    #[test]
    fn isr_reads_what_the_task_installed() {
        TABLE.set(1, Some(&B));
        // SAFETY: in this test there is no interrupt at all.
        assert_eq!(unsafe { TABLE.isr_get(1) }.map(|h| h.id()), Some(1));
    }
}
