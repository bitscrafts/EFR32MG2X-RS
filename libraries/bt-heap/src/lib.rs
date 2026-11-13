//! A first-fit heap with coalescing, small enough to read in one sitting.
//!
//! The Bluetooth stack calls `sl_malloc` and `sl_free`; this answers them. `no_std`, no dependencies, and it
//! lives in `libraries/` because it is logic with no hardware in it — so it is tested on the host rather than
//! discovered broken on the board.
//!
//! # The bug this design exists to prevent
//!
//! The first version had no header on **allocated** blocks (to save eight bytes per allocation) and recovered a
//! block's size on free by looking at the next free block's address. That is wrong the moment an allocated block
//! sits between the block being freed and the next free one: the size comes out too large, and coalescing
//! swallows the neighbour. The example caught it — `alloc(a); alloc(b); alloc(c); free(b); alloc(...)` returned
//! a block covering `c`, whose contents were gone.
//!
//! So every block carries an eight-byte header with its size. Eight bytes per allocation, and the arithmetic
//! becomes local and true: `free(p)` reads the size at `p - HEADER`, and nothing has to be inferred from
//! neighbours.
//!
//! # Design
//!
//! - **First fit over an address-ordered free list.** The stack's allocations are few; first fit reuses a freed
//!   block instead of growing the arena.
//! - **Coalescing on free**, which is not optional: without it an allocate/free cycle with two sizes shatters
//!   the arena into pieces too small to reuse, and the failure shows up later as a null return from an unrelated
//!   call.
//! - **The free list lives in the payload area** of free blocks, so a 4 KiB arena is mostly available to the
//!   stack.
//! - **`ALIGN` is 8**: the stack's structures hold 64-bit counters, and an unaligned `u64` faults on this core.

#![no_std]

/// Every block starts on this boundary and every size is a multiple of it.
pub const ALIGN: usize = 8;

/// Per-block header: the size in bytes, including the header itself. Present on allocated *and* free blocks —
/// see the module note for the bug that made this necessary.
const HEADER: usize = 8;

/// Bytes a free block needs beyond its size word: the link to the next free block, as an offset from the arena
/// base. Written and read through raw pointers, so the type exists only to size it correctly.
const LINK: usize = core::mem::size_of::<Option<usize>>();

/// A bounded heap over a borrowed buffer.
pub struct Heap<'a> {
    buf: &'a mut [u8],
    /// Offset of the first free block, or `None` when nothing is free.
    free: Option<usize>,
    in_use: usize,
    failures: u32,
}

impl<'a> Heap<'a> {
    /// Wrap a buffer. The whole buffer becomes one free block.
    pub fn new(buf: &'a mut [u8]) -> Self {
        let len = buf.len() - (buf.len() % ALIGN);
        let mut heap = Self {
            buf,
            free: None,
            in_use: 0,
            failures: 0,
        };
        if len >= HEADER + LINK + ALIGN {
            heap.write_size(0, len);
            heap.write_next(0, None);
            heap.free = Some(0);
        }
        heap
    }

    fn write_size(&mut self, at: usize, size: usize) {
        // SAFETY: `at` is 8-aligned inside the buffer and has at least HEADER bytes.
        unsafe {
            core::ptr::write_unaligned(self.buf.as_mut_ptr().add(at) as *mut usize, size);
        }
    }

    fn read_size(&self, at: usize) -> usize {
        // SAFETY: as `write_size`; only called for offsets the free list says are blocks, or for a pointer this
        // heap returned.
        unsafe { core::ptr::read_unaligned(self.buf.as_ptr().add(at) as *const usize) }
    }

    fn write_next(&mut self, at: usize, next: Option<usize>) {
        // SAFETY: the link sits after the size word inside a block that is at least HEADER + LINK long.
        unsafe {
            core::ptr::write_unaligned(
                self.buf.as_mut_ptr().add(at + HEADER) as *mut Option<usize>,
                next,
            );
        }
    }

    fn read_next(&self, at: usize) -> Option<usize> {
        // SAFETY: as `write_next`.
        unsafe {
            core::ptr::read_unaligned(self.buf.as_ptr().add(at + HEADER) as *const Option<usize>)
        }
    }

    /// Allocate `size` bytes, 8-byte aligned, or return null.
    pub fn alloc(&mut self, size: usize) -> *mut u8 {
        if size == 0 {
            return core::ptr::null_mut();
        }
        let need = round_up(size + HEADER);

        let mut prev: Option<usize> = None;
        let mut cur = self.free;
        while let Some(at) = cur {
            let block_size = self.read_size(at);
            if block_size >= need {
                let next = self.read_next(at);
                let leftover = block_size - need;
                // A remainder only becomes a block if it can hold a header, a link and one aligned payload.
                if leftover >= HEADER + LINK + ALIGN {
                    let tail = at + need;
                    self.write_size(tail, leftover);
                    self.write_next(tail, next);
                    self.write_size(at, need);
                    match prev {
                        Some(p) => self.write_next(p, Some(tail)),
                        None => self.free = Some(tail),
                    }
                } else {
                    // Hand the whole block over: up to 15 bytes wasted, and no unsplittable fragment left.
                    match prev {
                        Some(p) => self.write_next(p, next),
                        None => self.free = next,
                    }
                    self.write_size(at, block_size);
                }
                self.in_use += need;
                // The payload starts after the header; the header keeps the size for `dealloc`.
                return unsafe { self.buf.as_mut_ptr().add(at + HEADER) };
            }
            prev = Some(at);
            cur = self.read_next(at);
        }

        self.failures = self.failures.saturating_add(1);
        core::ptr::null_mut()
    }

    /// Return a block. A null pointer is accepted and ignored, because `free(NULL)` is legal everywhere else.
    pub fn dealloc(&mut self, ptr: *mut u8) {
        if ptr.is_null() {
            return;
        }
        let base = self.buf.as_ptr() as usize;
        let at = ptr as usize;
        if at < base + HEADER || at >= base + self.buf.len() {
            self.failures = self.failures.saturating_add(1);
            return;
        }
        let offset = at - base - HEADER;
        if !offset.is_multiple_of(ALIGN) || offset + HEADER > self.buf.len() {
            self.failures = self.failures.saturating_add(1);
            return;
        }
        let size = self.read_size(offset);
        if size < HEADER || !size.is_multiple_of(ALIGN) || offset + size > self.buf.len() {
            // Not one of ours, or the header was overwritten. Counting is better than corrupting the list.
            self.failures = self.failures.saturating_add(1);
            return;
        }

        // Insert in address order, then merge with both neighbours. Address order is what makes coalescing
        // possible; a LIFO free list would be simpler and would fragment exactly as described above.
        let mut prev: Option<usize> = None;
        let mut cur = self.free;
        while let Some(c) = cur {
            if c > offset {
                break;
            }
            prev = Some(c);
            cur = self.read_next(c);
        }
        self.write_next(offset, cur);
        match prev {
            Some(p) => self.write_next(p, Some(offset)),
            None => self.free = Some(offset),
        }
        self.in_use = self.in_use.saturating_sub(size);
        self.coalesce();
    }

    /// Merge every adjacent pair of free blocks.
    fn coalesce(&mut self) {
        let mut cur = self.free;
        while let Some(at) = cur {
            let size = self.read_size(at);
            let next = self.read_next(at);
            if let Some(next_at) = next {
                if at + size == next_at {
                    let merged = size + self.read_size(next_at);
                    let after = self.read_next(next_at);
                    self.write_size(at, merged);
                    self.write_next(at, after);
                    // Stay: the merged block may now touch the one after it too.
                    continue;
                }
            }
            cur = next;
        }
    }

    /// Bytes handed out, including headers, so the number matches the arena arithmetic.
    pub fn in_use(&self) -> usize {
        self.in_use
    }

    /// Allocations that could not be satisfied, plus pointers that were not this heap's.
    pub fn failures(&self) -> u32 {
        self.failures
    }

    /// The largest single payload allocation still possible.
    pub fn largest_free(&self) -> usize {
        let mut best = 0;
        let mut cur = self.free;
        while let Some(at) = cur {
            let size = self.read_size(at);
            if size > best && size >= HEADER + LINK + ALIGN {
                best = size;
            }
            cur = self.read_next(at);
        }
        if best == 0 {
            0
        } else {
            best - HEADER
        }
    }
}

fn round_up(n: usize) -> usize {
    (n + ALIGN - 1) & !(ALIGN - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_heap<T>(f: impl FnOnce(&mut Heap<'_>) -> T) -> T {
        let mut buf = [0u8; 2048];
        let mut heap = Heap::new(&mut buf);
        f(&mut heap)
    }

    #[test]
    fn allocates_aligned_non_overlapping_blocks() {
        with_heap(|heap| {
            let a = heap.alloc(1);
            let b = heap.alloc(9);
            assert!(!a.is_null() && !b.is_null());
            assert_eq!(a as usize % ALIGN, 0);
            assert_eq!(b as usize % ALIGN, 0);
            unsafe {
                core::ptr::write_bytes(a, 0x11, 1);
                core::ptr::write_bytes(b, 0x22, 9);
                assert_eq!(*a, 0x11);
                assert_eq!(*b, 0x22);
            }
        });
    }

    #[test]
    fn freed_block_is_reused() {
        with_heap(|heap| {
            let a = heap.alloc(64);
            heap.dealloc(a);
            let b = heap.alloc(64);
            assert_eq!(a, b, "the same block must come back");
        });
    }

    /// The regression the example found: a block freed between two allocated neighbours must not swallow them.
    #[test]
    fn freeing_the_middle_block_leaves_the_neighbours_intact() {
        with_heap(|heap| {
            let a = heap.alloc(64);
            let b = heap.alloc(32);
            let c = heap.alloc(16);
            unsafe {
                core::ptr::write_bytes(a, 0xAA, 64);
                core::ptr::write_bytes(b, 0xBB, 32);
                core::ptr::write_bytes(c, 0xCC, 16);
            }
            heap.dealloc(b);
            let d = heap.alloc(32);
            assert!(!d.is_null());
            unsafe {
                assert_eq!(*a, 0xAA, "the first neighbour must survive");
                assert_eq!(*c, 0xCC, "the second neighbour must survive");
            }
        });
    }

    #[test]
    fn adjacent_free_blocks_coalesce() {
        with_heap(|heap| {
            let a = heap.alloc(100);
            let b = heap.alloc(100);
            heap.dealloc(a);
            heap.dealloc(b);
            // Neither 100-byte hole fits 200; the merged pair must.
            let big = heap.alloc(200);
            assert!(!big.is_null(), "coalescing must make 200 bytes available");
            assert_eq!(big, a);
        });
    }

    #[test]
    fn out_of_memory_is_reported_not_fatal() {
        with_heap(|heap| {
            let mut n = 0;
            while n < 100 && !heap.alloc(256).is_null() {
                n += 1;
            }
            assert!(n > 0);
            assert!(heap.failures() > 0);
            assert_eq!(heap.alloc(256), core::ptr::null_mut());
        });
    }

    #[test]
    fn foreign_pointer_is_refused() {
        with_heap(|heap| {
            let mut elsewhere = [0u8; 16];
            let before = heap.failures();
            heap.dealloc(elsewhere.as_mut_ptr());
            assert_eq!(heap.failures(), before + 1);
            assert_eq!(heap.in_use(), 0);
        });
    }

    #[test]
    fn largest_free_shrinks_then_recovers() {
        with_heap(|heap| {
            let start = heap.largest_free();
            let a = heap.alloc(256);
            let mid = heap.largest_free();
            assert!(mid < start);
            heap.dealloc(a);
            assert!(
                heap.largest_free() >= mid,
                "returning a block must restore capacity"
            );
        });
    }
}
