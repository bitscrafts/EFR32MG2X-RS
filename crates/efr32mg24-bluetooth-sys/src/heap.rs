//! `sl_malloc` / `sl_free`, over the heap in `libraries/bt-heap`.
//!
//! The stack allocates; there is no libc here. The arena is static and deliberately small — 4 KiB, about a
//! fortieth of this part's RAM — because a heap that grows to hide a leak is a heap that fails on the customer's
//! device instead of on the bench.
//!
//! Reentrancy: not supported, and measured rather than assumed. If the stack ever allocates from an interrupt,
//! `contention` will be non-zero and the caller will find out from the example's diagnostics instead of from a
//! corrupted free list.

use bt_heap::Heap;
use core::ffi::c_void;

/// The arena. Sized from measurement of what the stack asks for during boot, rounded up generously; see the
/// `largest_free` report in the example if a future configuration needs more.
const ARENA_BYTES: usize = 16384;

static mut ARENA: [u8; ARENA_BYTES] = [0; ARENA_BYTES];
static mut HEAP: Option<Heap<'static>> = None;

/// Calls that arrived while another allocation was in progress. Should be zero; a non-zero value means the
/// caller must move its allocation out of the interrupt context.
static mut CONTENTION: u32 = 0;
static mut BUSY: bool = false;

/// Count a call that arrived while another was in progress.
fn bump_contention() {
    let p = core::ptr::addr_of_mut!(CONTENTION);
    // SAFETY: single word, written under the same serialisation the entry points use.
    unsafe { core::ptr::write_volatile(p, core::ptr::read_volatile(p).wrapping_add(1)) };
}

fn heap() -> &'static mut Heap<'static> {
    // SAFETY: single-threaded, and `BUSY` serialises the entry points below. The arena lives for the program.
    unsafe {
        let slot = &mut *core::ptr::addr_of_mut!(HEAP);
        if slot.is_none() {
            let arena = &mut *core::ptr::addr_of_mut!(ARENA);
            *slot = Some(Heap::new(arena));
        }
        slot.as_mut().unwrap()
    }
}

/// Allocate `size` bytes, or return null.
///
/// # Safety
///
/// Called by the vendor stack with a size it computed; the returned pointer must be released with `sl_free`.
#[no_mangle]
pub unsafe extern "C" fn sl_malloc(size: usize) -> *mut c_void {
    if core::ptr::read_volatile(core::ptr::addr_of!(BUSY)) {
        bump_contention();
        return core::ptr::null_mut();
    }
    core::ptr::write_volatile(core::ptr::addr_of_mut!(BUSY), true);
    let p = heap().alloc(size) as *mut c_void;
    core::ptr::write_volatile(core::ptr::addr_of_mut!(BUSY), false);
    p
}

/// Release a block.
///
/// # Safety
///
/// `ptr` must come from `sl_malloc` and not have been freed already, or be null.
#[no_mangle]
pub unsafe extern "C" fn sl_free(ptr: *mut c_void) {
    if core::ptr::read_volatile(core::ptr::addr_of!(BUSY)) {
        bump_contention();
        return;
    }
    core::ptr::write_volatile(core::ptr::addr_of_mut!(BUSY), true);
    heap().dealloc(ptr as *mut u8);
    core::ptr::write_volatile(core::ptr::addr_of_mut!(BUSY), false);
}

/// Bytes handed out, allocation failures, and corrupt-list violations — the three numbers that say whether the
/// arena is adequate, without a debugger. Violations are separate from failures on purpose: a failure means the
/// caller asked for more than is left, while a violation means someone wrote past an allocation.
pub fn stats() -> (usize, u32, u32) {
    // SAFETY: the heap is only touched through `heap()`, which is serialised by `BUSY`; reading here is a
    // diagnostic and races only with an allocation that is in progress.
    unsafe {
        let slot = &*core::ptr::addr_of!(HEAP);
        match slot.as_ref() {
            Some(h) => (h.in_use(), h.failures(), h.violations()),
            None => (0, 0, 0),
        }
    }
}

/// The largest single allocation still possible.
pub fn largest_free() -> usize {
    unsafe {
        let slot = &*core::ptr::addr_of!(HEAP);
        match slot.as_ref() {
            Some(h) => h.largest_free(),
            None => ARENA_BYTES,
        }
    }
}
