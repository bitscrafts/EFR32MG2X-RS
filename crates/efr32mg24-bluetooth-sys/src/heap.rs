//! `sl_malloc` / `sl_free`, over the heap in `libraries/bt-heap`.
//!
//! The stack allocates; there is no libc here. The arena is static and deliberately small — 4 KiB, about a
//! fortieth of this part's RAM — because a heap that grows to hide a leak is a heap that fails on the customer's
//! device instead of on the bench.
//!
//! Reentrancy: supported, by serialising the free list behind a short critical section. It was *not* supported when
//! this module was written, and the reasoning -- "the stack allocates only from the main loop" -- held until pairing,
//! which allocates from the stack's own context. The failure surfaced as a pairing refused with SMP reason 8 and as
//! corrupt-list violations, not as a heap error.

use bt_heap::Heap;
use core::ffi::c_void;

/// Run `f` with interrupts masked, restoring the previous mask afterwards.
///
/// The project is PAC-only, so this is the two instructions the core offers rather than a dependency: `mrs` reads
/// PRIMASK, `cpsid i` sets it, and the old value is written back so a critical section nested inside another one does
/// not unmask on the way out.
#[inline]
fn critical<R>(f: impl FnOnce() -> R) -> R {
    let primask: u32;
    // SAFETY: reading and writing PRIMASK is architecturally defined on this core; interrupts are restored to their
    // previous state before returning, so the caller's context is unchanged.
    unsafe {
        core::arch::asm!("mrs {}, PRIMASK", out(reg) primask, options(nomem, nostack));
        core::arch::asm!("cpsid i", options(nomem, nostack));
    }
    let result = f();
    // SAFETY: as above; only the bit this function set is cleared.
    unsafe {
        core::arch::asm!("msr PRIMASK, {}", in(reg) primask, options(nomem, nostack));
    }
    result
}

/// The arena.
///
/// Sized by measurement, twice. 16 KiB was enough for a stack that never worked: the host asks for
/// `SL_BT_CONFIG_BUFFER_SIZE` (8192) and the controller's link layer asks for `SL_BT_CONTROLLER_BUFFER_MEMORY`
/// (8192), and with the controller's initialisation missing, only the first of those was ever requested. Once the
/// controller initialisation was restored, `sl_btctrl_init_mem(8192)` came first and left the host without room, so
/// the link layer's first command buffer allocation returned NULL and `hci_command_init_shared` faulted writing to
/// address 5 -- a fault that says "null buffer" only if you already know that is what it is.
// Sized against a pairing attempt, not against advertising. The security manager allocates its own buffers, and
// `smp_state_slave_idle` answers a pairing request whose allocation failed with SMP reason 8 -- "unspecified" -- which
// on the air is indistinguishable from any other refusal. The device has 252 KB of RAM; 32 KB was a guess made when
// the only consumer was the advertiser.
const ARENA_BYTES: usize = 65536;

static mut ARENA: [u8; ARENA_BYTES] = [0; ARENA_BYTES];
static mut HEAP: Option<Heap<'static>> = None;

// The contention counter and the `BUSY` flag that fed it are gone with the refusal they described: serialisation is
// now a critical section, so "a call arrived during another" is no longer a state that can exist.

fn heap() -> &'static mut Heap<'static> {
    // SAFETY: the entry points run inside `critical()`, so no other context touches this while it is borrowed,
    // and the arena lives for the program.
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
    // **Serialised, not refused.** The previous version returned NULL when a second call arrived while one was in
    // progress, and counted it as "contention". Pairing made that fatal: the security manager allocates from the
    // stack's own context while the main loop is draining events, and a refused 352-byte allocation is answered on
    // the air with SMP reason 8 -- "unspecified". The free list also showed corrupt-list violations, which is what
    // two writers to one list look like from the allocator's side.
    //
    // The critical section is a few microseconds: the list is small and the search is first-fit.
    critical(|| heap().alloc(size) as *mut c_void)
}

/// Release a block.
///
/// # Safety
///
/// `ptr` must come from `sl_malloc` and not have been freed already, or be null.
#[no_mangle]
pub unsafe extern "C" fn sl_free(ptr: *mut c_void) {
    // Same critical section as `sl_malloc`: a free that interleaved with an allocation in the other context would
    // splice the list under the allocator's feet.
    critical(|| heap().dealloc(ptr as *mut u8))
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

/// The last size an allocation asked for and could not get.
pub fn last_failed_size() -> usize {
    unsafe {
        let slot = &*core::ptr::addr_of!(HEAP);
        match slot.as_ref() {
            Some(h) => h.last_failed_size(),
            None => 0,
        }
    }
}

/// `(usable_size, offset)` of the last block some caller wrote past.
pub fn last_damaged() -> (usize, usize) {
    unsafe {
        let slot = &*core::ptr::addr_of!(HEAP);
        match slot.as_ref() {
            Some(h) => h.last_damaged(),
            None => (0, 0),
        }
    }
}

/// `(count, first_offset)` of frees of a block that was already free -- a use-after-free signature.
pub fn double_frees() -> (u32, usize) {
    unsafe {
        let slot = &*core::ptr::addr_of!(HEAP);
        match slot.as_ref() {
            Some(h) => h.double_frees(),
            None => (0, 0),
        }
    }
}

/// The last few allocations, newest first, as `(requested_size, offset)`.
pub fn recent() -> [(usize, usize); bt_heap::RECENT] {
    unsafe {
        let slot = &*core::ptr::addr_of!(HEAP);
        match slot.as_ref() {
            Some(h) => h.recent(),
            None => [(0, 0); bt_heap::RECENT],
        }
    }
}

/// `(damaged_blocks, total_blocks, first_damaged_size, first_damaged_offset)` for the whole arena.
pub fn audit() -> (u32, u32, usize, usize) {
    unsafe {
        let slot = &*core::ptr::addr_of!(HEAP);
        match slot.as_ref() {
            Some(h) => h.audit(),
            None => (0, 0, 0, 0),
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
