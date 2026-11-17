//! The storage side of bonding, provided by us instead of by the stack.
//!
//! The host library ships two implementations of the bonding database: `bondingdb.c`, which keeps records in NVM3
//! and — measured on this board — writes the record and not the long-term key, so a peer that reconnects is answered
//! `0x1006 PIN or Key Missing`; and `bondingdb-stubs.c`, a bridge that implements the same internal API by delegating
//! to four callbacks that the application provides.
//!
//! This module is those callbacks. With the `external-bonding` feature on, the build derives the host archive without
//! the internal implementation and links the bridge in its place, so these functions are the ones that run.
//!
//! OPTION, NOT YET TAKEN: the bridge is the PSA-key-backed database, so a bond -- long-term key included -- is stored
//! as a PSA key by `psa.rs`. That table is RAM only, which is why a bond cannot survive a reset with this feature on.
//! Persisting it (one NVM3 object per key, id range restored at start-up) is the smaller change than implementing the
//! remaining database API, and it is written up where the table lives, in `psa.rs`.
//!
//! The signatures are in no header the vendor ships; they were read from the call sites. `store_data` was read from
//! the internal implementation, which does the same work: validate the handle, build the storage key as
//! `(handle << 8) | type`, and write `len` bytes from `data`. The same key formula is used here, moved into a key
//! range of our own so it cannot collide with anything the stack keeps in NVM3.

use core::sync::atomic::{AtomicU32, Ordering};

/// Keys of our bonding records live above anything the stack uses: 20-bit keys, and the top nibble of the range is
/// ours. `(handle << 8) | type` then fits inside one handle's worth of keys with no overlap between handles.
const KEY_BASE: u32 = 0xE_0000;

/// The record pieces the stack writes, as seen in the internal implementation, keep their own type byte; a delete
/// walks the same small set rather than the whole 256-value space.
const MAX_PIECE: u8 = 16;

/// How many bonds are kept, matching the store configuration the stack is given (8).
const MAX_SLOTS: u8 = 8;

extern "C" {
    static mut nvm3_defaultHandle: *mut u8;
    fn nvm3_writeData(handle: *mut u8, key: u32, data: *const u8, length: usize) -> u32;
    fn nvm3_readData(handle: *mut u8, key: u32, data: *mut u8, length: usize) -> u32;
    fn nvm3_deleteObject(handle: *mut u8, key: u32) -> u32;
    fn nvm3_getObjectInfo(handle: *mut u8, key: u32, obj_type: *mut u32, length: *mut usize)
        -> u32;
}

#[inline]
fn storage_key(bonding: u8, piece: u8) -> u32 {
    KEY_BASE | ((bonding as u32) << 8) | piece as u32
}

#[inline]
fn handle() -> *mut u8 {
    unsafe { nvm3_defaultHandle }
}

/// Counters, so a test can tell the callbacks apart from the stack never asking. `get_database_type` is asked once at
/// start-up; the others move during a pairing.
static CALLS_IS_BONDED: AtomicU32 = AtomicU32::new(0);
static CALLS_STORE: AtomicU32 = AtomicU32::new(0);
static CALLS_DB_TYPE: AtomicU32 = AtomicU32::new(0);
static CALLS_DELETE: AtomicU32 = AtomicU32::new(0);
static CALLS_RESOLVE: AtomicU32 = AtomicU32::new(0);
static STORE_ERRORS: AtomicU32 = AtomicU32::new(0);
static CALLS_READ: AtomicU32 = AtomicU32::new(0);
static READ_ERRORS: AtomicU32 = AtomicU32::new(0);

/// `(is_bonded, store, db_type, delete, resolve)` and the number of stores NVM3 refused.
pub fn stats() -> (u32, u32, u32, u32, u32, u32) {
    (
        CALLS_IS_BONDED.load(Ordering::Relaxed),
        CALLS_STORE.load(Ordering::Relaxed),
        CALLS_DB_TYPE.load(Ordering::Relaxed),
        CALLS_DELETE.load(Ordering::Relaxed),
        CALLS_READ.load(Ordering::Relaxed),
        STORE_ERRORS.load(Ordering::Relaxed) + READ_ERRORS.load(Ordering::Relaxed),
    )
}

/// Which kind of database this is: the stack asks once, at start-up, and routes its storage through the answer.
#[no_mangle]
pub extern "C" fn sli_bt_external_bonding_get_database_type() -> u32 {
    CALLS_DB_TYPE.fetch_add(1, Ordering::Relaxed);
    1
}

/// Which slot a record should occupy, or -1 when the store is full.
///
/// The internal implementation walks its handles, asks whether each one is already bonded and answers with the first
/// free one -- read from its disassembly (`movs r7, #0` before the loop, a read of the record type 8 per handle, a
/// compare against the configured maximum). The vendor's external-database default instead answers with a byte from
/// the record itself, which is why copying that default left the pairing failing: the stack was told to reuse a slot
/// it had not been given.
///
/// # Safety
///
/// `record` is the stack's record structure; this implementation does not read it, but the argument is part of the
/// contract and is accepted unchanged.
#[no_mangle]
pub unsafe extern "C" fn sli_bt_external_bonding_alloc(record: *const u8) -> i8 {
    let _ = record;
    for bonding in 0..MAX_SLOTS {
        let mut obj_type = 0u32;
        let mut length = 0usize;
        let taken = unsafe {
            nvm3_getObjectInfo(
                handle(),
                storage_key(bonding, 8),
                &mut obj_type,
                &mut length,
            )
        } == 0;
        if !taken {
            return bonding as i8;
        }
    }
    -1
}

/// Whether a bonding handle names a record this application holds.
///
/// The internal implementation answers this by reading the piece numbered 8 with a length of 8, so the same question
/// is asked of NVM3 here: does that object exist.
///
/// # Safety
///
/// Called by the stack with the C convention; `bonding` is a handle, not a pointer.
#[no_mangle]
pub extern "C" fn sli_bt_external_bonding_is_bonded(bonding: u8) -> bool {
    CALLS_IS_BONDED.fetch_add(1, Ordering::Relaxed);
    // 0xFF is the "no bonding" handle. The vendor's external default answers `(handle + 1) != 0`, and asking NVM3
    // instead would report a record as absent while the stack is still filling it in.
    let mut obj_type = 0u32;
    let mut length = 0usize;
    let exists = unsafe {
        nvm3_getObjectInfo(
            handle(),
            storage_key(bonding, 8),
            &mut obj_type,
            &mut length,
        )
    } == 0;
    bonding != 0xFF || exists
}

/// Store one piece of a bonding record. 0 is success; 0x406 is the status the internal implementation returns for a
/// handle it does not recognise.
///
/// # Safety
///
/// `data` must point to `length` readable bytes.
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
    let rc = nvm3_writeData(handle(), storage_key(bonding, piece), data, length);
    if rc != 0 {
        STORE_ERRORS.fetch_add(1, Ordering::Relaxed);
        // NVM3 refused: report it as a storage failure rather than pretending the record exists.
        return rc;
    }
    0
}

/// Read one piece of a bonding record back.
///
/// # Safety
///
/// `data` must point to `length` writable bytes.
#[no_mangle]
pub unsafe extern "C" fn sli_bt_external_bonding_read_data(
    bonding: u8,
    piece: u8,
    data: *mut u8,
    length: usize,
) -> u32 {
    if data.is_null() || length == 0 {
        return 0x406;
    }
    nvm3_readData(handle(), storage_key(bonding, piece), data, length)
}

/// Forget every piece of one bonding record.
///
/// # Safety
///
/// Called by the stack with the C convention; `bonding` is a handle.
#[no_mangle]
pub extern "C" fn sli_bt_external_bonding_delete(bonding: u8) -> u32 {
    CALLS_DELETE.fetch_add(1, Ordering::Relaxed);
    for piece in 0..MAX_PIECE {
        unsafe {
            nvm3_deleteObject(handle(), storage_key(bonding, piece));
        }
    }
    0
}

/// Whether an address belongs to a bond this application holds.
///
/// Reading the address out of a record means knowing which piece holds it, and that piece number is not in any
/// documented place; until it is measured, the honest answer is "not resolved by this layer" rather than a guess that
/// could bind the wrong peer to a record.
///
/// # Safety
///
/// `address` points to six bytes when non-null.
#[no_mangle]
pub unsafe extern "C" fn sli_bt_external_bonding_resolve_address(
    address: *const u8,
    address_type: u8,
) -> u32 {
    CALLS_RESOLVE.fetch_add(1, Ordering::Relaxed);
    let _ = (address, address_type);
    0
}

/// Read one piece of a bonding record.
///
/// The bridge ships this one as a stub -- `movs r0, #0; bx lr`, success without touching the buffer -- so with an
/// external database the stack stores a record and cannot read it back, and the pairing that follows fails
/// authentication with nothing in the counters to say why. The symbol is weak, so a strong definition here is the one
/// the linker keeps. Same shape as the store, and measured the same way: `(handle, type, data, length)`.
///
/// # Safety
///
/// `data` must point to `length` writable bytes.
#[no_mangle]
pub unsafe extern "C" fn sli_bt_bonding_get_data(
    bonding: u8,
    piece: u8,
    data: *mut u8,
    length: usize,
) -> u32 {
    CALLS_READ.fetch_add(1, Ordering::Relaxed);
    if data.is_null() || length == 0 {
        return 0x406;
    }
    let rc = nvm3_readData(handle(), storage_key(bonding, piece), data, length);
    if rc != 0 {
        READ_ERRORS.fetch_add(1, Ordering::Relaxed);
    }
    rc
}

/// How many bonds the store can hold.
///
/// The bridge ships this as a stub returning zero, and zero is not a harmless default: with no capacity the security
/// manager refuses to bond, and the peer is told the pairing failed authentication with nothing in the record to say
/// why. Measured 2026-11-17: the pairing failed with AuthenticationFailed while `is_bonded` and `store_data` were
/// both being called, because the stack had been told it could keep no bonds at all.
#[no_mangle]
pub extern "C" fn sli_bt_bonding_get_max_bondings() -> u8 {
    MAX_SLOTS
}

/// Whether a bonding handle names one of the slots this store provides.
///
/// Also a stub in the bridge, also answering zero -- which would make every handle invalid.
///
/// # Safety
///
/// Called by the stack with the C convention; `bonding` is a handle.
#[no_mangle]
pub extern "C" fn sli_bt_bonding_is_valid_handle(bonding: u8) -> bool {
    bonding < MAX_SLOTS
}
