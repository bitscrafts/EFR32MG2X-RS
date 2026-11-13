//! What the heap does, and what it reports when it runs out.
//!
//! Runs on the host: `cargo run --target x86_64-unknown-linux-gnu -p bt-heap --example heap_report`.
//!
//! The numbers matter as much as the algorithm. On the board, `sl_malloc` failing is a Bluetooth feature that
//! silently does not start, so the counters this prints (`in_use`, `failures`, `largest_free`) are what a
//! bring-up example should print when something else breaks.

use bt_heap::Heap;

fn main() {
    const ARENA_BYTES: usize = 512;
    let mut arena = [0u8; ARENA_BYTES];
    let mut heap = Heap::new(&mut arena);

    println!(
        "arena {ARENA_BYTES} bytes, largest free at start: {}",
        heap.largest_free()
    );

    // Four blocks, written through, so a corruption would show as mismatched bytes.
    let a = heap.alloc(64);
    let b = heap.alloc(32);
    let c = heap.alloc(16);
    unsafe {
        core::ptr::write_bytes(a, 0xAA, 64);
        core::ptr::write_bytes(b, 0xBB, 32);
        core::ptr::write_bytes(c, 0xCC, 16);
    }
    println!(
        "three blocks: in_use={} largest_free={}",
        heap.in_use(),
        heap.largest_free()
    );

    // Free the middle one and take a block that only fits because its neighbours were not disturbed.
    heap.dealloc(b);
    let d = heap.alloc(32);
    println!("after free+alloc: in_use={}", heap.in_use());
    unsafe {
        assert_eq!(*a, 0xAA, "the first block must be untouched");
        assert_eq!(*c, 0xCC, "the third block must be untouched");
        // A reused block holds neither zeros nor its old payload: while it was free, its first bytes were the
        // free-list link. Both assumptions were wrong here before, which is the point of asserting about
        // neighbours instead of about recycled memory.
        core::ptr::write_bytes(d, 0xDD, 32);
        assert_eq!(*d, 0xDD);
    }

    // Exhaust it, then report the failure instead of pretending the allocation worked.
    let mut held = 0;
    loop {
        let p = heap.alloc(64);
        if p.is_null() {
            break;
        }
        held += 1;
        if held > 100 {
            panic!("the allocator must fail before 100 blocks of 64 bytes in 512");
        }
    }
    let (in_use, failures) = (heap.in_use(), heap.failures());
    println!("exhausted after {held} more blocks: in_use={in_use} failures={failures}");
    assert!(failures > 0, "the failure must be counted, not swallowed");

    // Everything back, and the arena is usable again as one block.
    heap.dealloc(a);
    heap.dealloc(c);
    heap.dealloc(d);
    println!(
        "after returning the first three: in_use={} largest_free={}",
        heap.in_use(),
        heap.largest_free()
    );
}
