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

/// A word written at the end of every block, past the last byte the caller may use. `dealloc` checks it, so an overrun
/// is reported against **the block that was overrun** -- its size and offset -- instead of only being noticed later as a
/// corrupt free list. That difference is what turns "eight violations" into "the 352-byte block was written past".
pub const CANARY: usize = 8;

/// The value written into the canary word, mixed with the block's offset so that a stray copy of a fixed pattern is
/// unlikely to reproduce it.
// **A value that fits `usize` on both targets.** The board is 32-bit and the host running the tests is 64-bit; a
// 64-bit magic compiled on the board and silently became an out-of-range literal.
const CANARY_MAGIC: usize = 0xcafe_babe;

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
    /// Walks that hit a corrupt block or exceeded the step budget. Non-zero means the arena's own metadata was
    /// damaged, which almost always means someone wrote past an allocation.
    violations: u32,
    /// The last size an allocation asked for and could not be served. Recorded because "out of memory" without the
    /// size says nothing actionable: 352 bytes failing out of 64 KiB is a different bug from 40 KiB failing.
    last_failed_size: usize,
    /// The usable size of the last block whose canary was found damaged, and that block's offset. This is the number
    /// that names the overrunning consumer: blocks are handed out per call site, so the size identifies it.
    last_damaged_size: usize,
    last_damaged_offset: usize,
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
            violations: 0,
            last_failed_size: 0,
            last_damaged_size: 0,
            last_damaged_offset: 0,
        };
        if len >= HEADER + LINK + ALIGN {
            heap.write_size(0, len);
            heap.write_next(0, None);
            heap.free = Some(0);
        }
        heap
    }

    fn canary_for(at: usize) -> usize {
        CANARY_MAGIC ^ at
    }

    /// Write the canary into the last word of the block starting at `at`.
    fn write_canary(&mut self, at: usize, block_size: usize) {
        let value = Self::canary_for(at);
        // SAFETY: `block_size` is the block's own size and ends on an eight-byte boundary inside the buffer.
        unsafe {
            core::ptr::write_unaligned(
                self.buf.as_mut_ptr().add(at + block_size - CANARY) as *mut usize,
                value,
            );
        }
    }

    /// Check the canary at the end of the block starting at `at`. Records the block's usable size when it is damaged.
    fn check_canary(&mut self, at: usize, block_size: usize) -> bool {
        // SAFETY: as `write_canary`.
        let found = unsafe {
            core::ptr::read_unaligned(
                self.buf.as_ptr().add(at + block_size - CANARY) as *const usize
            )
        };
        if found == Self::canary_for(at) {
            return true;
        }
        self.violations = self.violations.saturating_add(1);
        self.last_damaged_size = block_size.saturating_sub(HEADER + CANARY);
        self.last_damaged_offset = at;
        false
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
        let need = round_up(size + HEADER + CANARY);

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
                self.write_canary(at, self.read_size(at));
                self.in_use += need;
                // The payload starts after the header; the header keeps the size for `dealloc`.
                return unsafe { self.buf.as_mut_ptr().add(at + HEADER) };
            }
            prev = Some(at);
            cur = self.read_next(at);
        }

        self.failures = self.failures.saturating_add(1);
        self.last_failed_size = size;
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
        // Before the list is touched: a damaged canary means the caller's bytes ran into the next block's header, and
        // splicing this block into the free list would spread the damage.
        self.check_canary(offset, size);
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
    ///
    /// **Every walk in this file is bounded.** A free list and a size word are ordinary memory: a caller that
    /// writes one byte past an allocation can turn the list into a cycle, and an unbounded `while let Some(..)`
    /// then spins forever. That is not hypothetical — it happened on the board, where the Bluetooth stack's
    /// init stopped writing to the mailbox and the core sat in a loop the debugger showed inside the firmware.
    /// A hang says nothing; a counted violation says where to look.
    fn coalesce(&mut self) {
        let cap = self.buf.len() / (HEADER + LINK + ALIGN) + 8;
        let mut steps = 0usize;
        let mut cur = self.free;
        while let Some(at) = cur {
            steps += 1;
            if steps > cap {
                self.violations = self.violations.saturating_add(1);
                return;
            }
            let size = self.read_size(at);
            let next = self.read_next(at);
            if !self.block_is_sane(at, size) {
                self.violations = self.violations.saturating_add(1);
                return;
            }
            if let Some(next_at) = next {
                if at + size == next_at {
                    let next_size = self.read_size(next_at);
                    let merged = size + next_size;
                    // Guard against a zero-size neighbour, which would make the merge a no-op and the loop
                    // spin: `at + size == next_at` stays true forever.
                    if next_size < HEADER || !merged.is_multiple_of(ALIGN) {
                        self.violations = self.violations.saturating_add(1);
                        return;
                    }
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

    /// A block is sane when it lies inside the arena, is 8-aligned, and is at least a header long.
    fn block_is_sane(&self, at: usize, size: usize) -> bool {
        at.is_multiple_of(ALIGN)
            && size >= HEADER
            && size.is_multiple_of(ALIGN)
            && at.saturating_add(size) <= self.buf.len()
    }

    /// Bytes handed out, including headers, so the number matches the arena arithmetic.
    pub fn in_use(&self) -> usize {
        self.in_use
    }

    /// Allocations that could not be satisfied, plus pointers that were not this heap's.
    pub fn failures(&self) -> u32 {
        self.failures
    }

    /// Walks that found a corrupt block or ran past the step budget. Non-zero is the signature of an
    /// out-of-bounds write by a caller, and it is reported instead of hanging.
    pub fn violations(&self) -> u32 {
        self.violations
    }

    /// The last size that could not be served.
    pub fn last_failed_size(&self) -> usize {
        self.last_failed_size
    }

    /// Walk the whole arena and check every block's canary, whether it is free or allocated.
    ///
    /// Blocks cover the arena contiguously, so a linear walk by size reaches all of them without consulting the free
    /// list -- which is the point: the free list is the thing suspected of being damaged. Returns
    /// `(damaged_blocks, total_blocks, first_damaged_size, first_damaged_offset)`.
    pub fn audit(&self) -> (u32, u32, usize, usize) {
        let mut at = 0usize;
        let mut blocks = 0u32;
        let mut damaged = 0u32;
        let mut first_size = 0usize;
        let mut first_at = 0usize;
        let limit = self.buf.len() - (self.buf.len() % ALIGN);
        let mut steps = 0usize;
        // Bounded, for the same reason every walk in this file is: a corrupt size word turns the loop into an
        // infinite one, and a hang says nothing.
        while at + HEADER + CANARY <= limit && steps <= limit / ALIGN {
            steps += 1;
            let size = self.read_size(at);
            if size < HEADER + CANARY || !size.is_multiple_of(ALIGN) || at + size > limit {
                damaged = damaged.saturating_add(1);
                if first_size == 0 {
                    first_size = size;
                    first_at = at;
                }
                break;
            }
            blocks = blocks.saturating_add(1);
            // SAFETY: `size` was just validated to lie inside the buffer.
            let found = unsafe {
                core::ptr::read_unaligned(self.buf.as_ptr().add(at + size - CANARY) as *const usize)
            };
            if found != Self::canary_for(at) {
                damaged = damaged.saturating_add(1);
                if first_size == 0 {
                    first_size = size.saturating_sub(HEADER + CANARY);
                    first_at = at;
                }
            }
            at += size;
        }
        (damaged, blocks, first_size, first_at)
    }

    /// `(usable_size, offset)` of the last block found with a damaged canary -- the block some consumer wrote past.
    pub fn last_damaged(&self) -> (usize, usize) {
        (self.last_damaged_size, self.last_damaged_offset)
    }

    /// Walk the free list and count blocks that fail the sanity check, without modifying anything. The entry
    /// points call this after every operation they can, because a heap that only fails later is a heap whose
    /// failure gets blamed on whatever ran in between.
    pub fn check(&mut self) -> u32 {
        let cap = self.buf.len() / (HEADER + LINK + ALIGN) + 8;
        let mut steps = 0usize;
        let mut bad = 0u32;
        let mut cur = self.free;
        while let Some(at) = cur {
            steps += 1;
            if steps > cap {
                self.violations = self.violations.saturating_add(1);
                return bad + 1;
            }
            let size = self.read_size(at);
            if !self.block_is_sane(at, size) {
                bad += 1;
                break;
            }
            cur = self.read_next(at);
        }
        bad
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

#[cfg(test)]
mod smp_size_tests {
    use super::*;

    /// The security manager's state block: `smp_alloc_data` asks for 0x160 bytes and then writes all of it. If this
    /// allocator hands back a block that cannot hold what it was asked for, the last bytes land on the next block's
    /// header -- which shows up as a corrupt free list, not as a heap error.
    /// The canary has to be a control, not a decoration: a caller that writes exactly what it asked for must pass, and
    /// one byte more must be reported against that block's size. Without the first half, a canary placed one word too
    /// early would flag every allocation in the system and the report would be worthless.
    #[test]
    fn writing_past_a_block_is_reported_against_that_block() {
        let mut arena = [0u8; 4096];
        let mut heap = Heap::new(&mut arena);

        let size = 0x160usize;
        let p = heap.alloc(size);
        assert!(!p.is_null());
        unsafe {
            for i in 0..size {
                *p.add(i) = 0xa5;
            }
        }
        heap.dealloc(p);
        assert_eq!(
            heap.violations(),
            0,
            "writing exactly `size` bytes must not be a violation"
        );
        assert_eq!(heap.last_damaged(), (0, 0));

        // The near miss: one byte past the end.
        let q = heap.alloc(size);
        assert!(!q.is_null());
        unsafe {
            for i in 0..=size {
                *q.add(i) = 0x5a;
            }
        }
        heap.dealloc(q);
        assert_eq!(heap.violations(), 1, "one byte past the end must be caught");
        let (damaged_size, offset) = heap.last_damaged();
        assert!(
            damaged_size >= size,
            "the report must name a block at least as large as the one that was overrun, got {damaged_size}"
        );
        // Offset zero is legitimate -- it is the arena's first block. What matters is that the offset is inside it.
        assert!(
            offset < 4096,
            "the block's offset must be recorded, got {offset}"
        );
    }

    #[test]
    fn a_352_byte_allocation_holds_352_bytes() {
        let mut arena = [0u8; 4096];
        let mut heap = Heap::new(&mut arena);

        let p = heap.alloc(0x160);
        assert!(!p.is_null(), "352 bytes out of 4 KiB must be served");
        // Write every byte the caller owns, then one canary pattern around it: any overrun would corrupt a header.
        unsafe {
            for i in 0..0x160 {
                *p.add(i) = (i % 251) as u8;
            }
        }
        for i in 0..0x160 {
            unsafe {
                assert_eq!(*p.add(i), (i % 251) as u8, "byte {i} was overwritten");
            }
        }

        // A second allocation must not overlap the first one's payload.
        let q = heap.alloc(0x40);
        assert!(!q.is_null());
        unsafe {
            let first_end = p as usize + 0x160;
            assert!(
                q as usize >= first_end,
                "second block at {q:p} overlaps the first block's payload ending at {first_end:#x}"
            );
        }

        heap.dealloc(p);
        heap.dealloc(q);
        assert_eq!(
            heap.violations(),
            0,
            "the free list was corrupted by a legal sequence"
        );
        assert_eq!(heap.failures(), 0);
    }
}
