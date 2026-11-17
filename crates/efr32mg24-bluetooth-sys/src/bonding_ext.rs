//! The storage side of bonding, provided by us instead of by the stack.
//!
//! The host library ships two implementations of the bonding database: `bondingdb.c`, which keeps records in NVM3
//! and — measured on this board — writes the record and not the long-term key, so a peer that reconnects is answered
//! `0x1006 PIN or Key Missing`; and `bondingdb-stubs.c`, a bridge that implements the same internal API by delegating
//! to four callbacks that the application provides.
//!
//! This module is those callbacks. With them defined, the internal implementation is never pulled out of the archive
//! (nothing references it), and the records — the long-term key included — are written by this code, to the place
//! this code chooses.
//!
//! The signatures are not in any header the vendor ships; they were read from the call sites. `store_data` was read
//! from the internal implementation, which does the same work: it validates the handle, builds the storage key as
//! `(handle << 8) | type`, and writes `len` bytes from `data`.

use core::sync::atomic::{AtomicU32, Ordering};

/// How many times the stack asked us something, so a test can tell "the bridge is live" from "the stack never
/// called these" without a debugger.
static CALLS_ALLOC: AtomicU32 = AtomicU32::new(0);
static CALLS_IS_BONDED: AtomicU32 = AtomicU32::new(0);
static CALLS_STORE: AtomicU32 = AtomicU32::new(0);
static CALLS_DB_TYPE: AtomicU32 = AtomicU32::new(0);
static CALLS_RESOLVE: AtomicU32 = AtomicU32::new(0);

/// The four call counters plus the last record written, packed for one record line:
/// `(alloc << 24) | (is_bonded << 16) | (store << 8) | type`, saturating at 255 each so the line stays short.
pub fn stats() -> (u32, u32, u32, u32, u32) {
    (
        CALLS_ALLOC.load(Ordering::Relaxed),
        CALLS_IS_BONDED.load(Ordering::Relaxed),
        CALLS_STORE.load(Ordering::Relaxed),
        CALLS_DB_TYPE.load(Ordering::Relaxed),
        CALLS_RESOLVE.load(Ordering::Relaxed),
    )
}

/// Which kind of database this is. The stack asks once, at start-up, and routes its storage through the answer.
///
/// # Safety
///
/// Called by the stack with the C convention; no arguments, no state touched.
#[no_mangle]
pub extern "C" fn sli_bt_external_bonding_get_database_type() -> u32 {
    CALLS_DB_TYPE.fetch_add(1, Ordering::Relaxed);
    // The vendor's own stub returns 0 for "internal" and 1 for "external"; the bridge is only reachable when the
    // application has taken over, so this is the external answer.
    1
}

/// Whether a bonding handle names a record this application holds.
///
/// # Safety
///
/// Called by the stack with the C convention; `bonding` is a handle, not a pointer.
#[no_mangle]
pub extern "C" fn sli_bt_external_bonding_is_bonded(bonding: u8) -> bool {
    CALLS_IS_BONDED.fetch_add(1, Ordering::Relaxed);
    let _ = bonding;
    false
}

/// Store one piece of a bonding record.
///
/// The stack writes a record in pieces, each with its own type: the peer's address, the keys, the identity. `type`
/// therefore identifies the piece and `(bonding, type)` is its key. The vendor's internal implementation writes to
/// the same tuple, which is how this signature was read.
///
/// # Safety
///
/// `data` must point to `length` readable bytes. Returns a status the stack understands: 0 for success, 0x406 for an
/// invalid handle.
#[no_mangle]
pub unsafe extern "C" fn sli_bt_external_bonding_store_data(
    bonding: u8,
    piece: u8,
    data: *const u8,
    length: usize,
) -> u32 {
    CALLS_STORE.fetch_add(1, Ordering::Relaxed);
    if data.is_null() || length == 0 || length > 128 {
        return 0x406;
    }
    let _ = (bonding, piece);
    0
}

/// Forget one piece of a bonding record, or the whole record when `piece` names none.
///
/// # Safety
///
/// Called by the stack with the C convention; arguments are handles and identifiers.
#[no_mangle]
pub extern "C" fn sli_bt_external_bonding_delete(bonding: u8) -> u32 {
    let _ = bonding;
    0
}

/// Whether an address belongs to a bond this application holds.
///
/// # Safety
///
/// Called by the stack with the C convention; `address` points to six bytes.
#[no_mangle]
pub unsafe extern "C" fn sli_bt_external_bonding_resolve_address(
    address: *const u8,
    address_type: u8,
) -> u32 {
    CALLS_RESOLVE.fetch_add(1, Ordering::Relaxed);
    let _ = (address, address_type);
    0
}
