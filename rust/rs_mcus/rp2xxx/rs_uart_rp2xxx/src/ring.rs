//! One single-producer / single-consumer byte ring per UART instance.
//!
//! The producer is the receive interrupt (it drains the PL011 FIFO), the
//! consumer is the task calling `read` — exactly the two contexts the ring is
//! built for. `head` is written by the producer only, `tail` by the consumer
//! only, so no lock (and no critical section) is needed: each side publishes its
//! own index with `Release` and reads the other's with `Acquire`, which is what
//! makes the bytes visible across the two.
//!
//! `N` must be a power of two.

use core::cell::UnsafeCell;
use portable_atomic::{AtomicU16, Ordering};

/// A byte ring of `N` bytes (`N` a power of two).
pub struct Ring<const N: usize> {
    data: UnsafeCell<[u8; N]>,
    /// Next byte to write; producer only.
    head: AtomicU16,
    /// Next byte to read; consumer only.
    tail: AtomicU16,
}

// SAFETY: the two indices are the only shared state, and each is written by one
// side only (see the module doc). The payload is accessed through the indices:
// `[tail, head)` belongs to the consumer, `[head, tail)` to the producer.
unsafe impl<const N: usize> Sync for Ring<N> {}

impl<const N: usize> Ring<N> {
    const MASK: u16 = (N - 1) as u16;

    /// An empty ring, usable in a `static`.
    pub const fn new() -> Self {
        assert!(N.is_power_of_two(), "Ring<N>: N must be a power of two");
        Self {
            data: UnsafeCell::new([0; N]),
            head: AtomicU16::new(0),
            tail: AtomicU16::new(0),
        }
    }

    /// Bytes held right now.
    pub fn used(&self) -> usize {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        head.wrapping_sub(tail) as usize
    }

    /// No byte waiting.
    pub fn is_empty(&self) -> bool {
        self.used() == 0
    }

    /// Append one byte (producer). `false` when the ring is full.
    pub fn push(&self, byte: u8) -> bool {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);
        if head.wrapping_sub(tail) as usize >= N {
            return false;
        }
        let slot = (head & Self::MASK) as usize;
        // SAFETY: the slot is in the producer's free range (`head` is not
        // reachable by the consumer until `head` is published below).
        unsafe { (*self.data.get())[slot] = byte };
        self.head.store(head.wrapping_add(1), Ordering::Release);
        true
    }

    /// Move up to `out.len()` bytes out of the ring (consumer).
    pub fn pop_into(&self, out: &mut [u8]) -> usize {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);
        let available = core::cmp::min(head.wrapping_sub(tail) as usize, out.len());
        for (offset, slot) in out.iter_mut().take(available).enumerate() {
            let index = ((tail.wrapping_add(offset as u16)) & Self::MASK) as usize;
            // SAFETY: the byte is in the consumer's filled range.
            *slot = unsafe { (*self.data.get())[index] };
        }
        self.tail
            .store(tail.wrapping_add(available as u16), Ordering::Release);
        available
    }

    /// Drop everything (consumer).
    pub fn clear(&self) {
        let head = self.head.load(Ordering::Acquire);
        self.tail.store(head, Ordering::Release);
    }
}

impl<const N: usize> Default for Ring<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_and_pop_keep_the_order() {
        let ring: Ring<8> = Ring::new();
        assert!(ring.is_empty());
        for byte in 0..4u8 {
            assert!(ring.push(byte));
        }
        assert_eq!(ring.used(), 4);
        let mut out = [0u8; 8];
        assert_eq!(ring.pop_into(&mut out), 4);
        assert_eq!(&out[..4], &[0, 1, 2, 3]);
        assert!(ring.is_empty());
    }

    #[test]
    fn full_ring_refuses_the_extra_byte() {
        let ring: Ring<4> = Ring::new();
        for byte in 0..4u8 {
            assert!(ring.push(byte));
        }
        assert!(!ring.push(9));
        assert_eq!(ring.used(), 4);
    }

    #[test]
    fn indices_wrap_around() {
        let ring: Ring<4> = Ring::new();
        let mut out = [0u8; 4];
        // 3 pushes + 3 pops, six times: the counters wrap the `u16` range many
        // times over, so the mask and `wrapping_sub` are what is under test.
        for round in 0..6000u16 {
            for i in 0..3u8 {
                assert!(ring.push((round as u8).wrapping_add(i)));
            }
            assert_eq!(ring.pop_into(&mut out), 3);
            assert_eq!(out[0], round as u8);
        }
        assert!(ring.is_empty());
    }

    #[test]
    fn pop_stops_at_the_consumer_length() {
        let ring: Ring<8> = Ring::new();
        for byte in 0..6u8 {
            assert!(ring.push(byte));
        }
        let mut out = [0u8; 2];
        assert_eq!(ring.pop_into(&mut out), 2);
        assert_eq!(out, [0, 1]);
        assert_eq!(ring.used(), 4);
        ring.clear();
        assert!(ring.is_empty());
    }
}

// EOF
