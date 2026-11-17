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
    fn nvm3_enumObjects(handle: *mut u8, key_list: *mut u32, key_list_size: usize) -> usize;
}

#[inline]
fn storage_key(bonding: u8, piece: u8) -> u32 {
    KEY_BASE | ((bonding as u32) << 8) | piece as u32
}

/// The stack's NVM3 handle. Named with the module in it because the helpers below take a `handle: u8` argument, and
/// a local of that name shadows a function of that name (found by the compiler, 2026-11-17).
#[inline]
fn nvm3_handle() -> *mut u8 {
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
/// The stack's request for bonding data and the answers given to it. With an external database this conversation is
/// what gates the security manager: `sm_init_bonding_data` asks instead of setting its "bonding data ready" bit, and
/// the pairing exchange does not start until the application answers. Counted here so a test can tell "asked and
/// answered" from "never asked".
static CALLS_DATA_REQ: AtomicU32 = AtomicU32::new(0);
static CALLS_DATA_ANSWERED: AtomicU32 = AtomicU32::new(0);
static LAST_DATA_STATUS: AtomicU32 = AtomicU32::new(0xFFFF_FFFF);
/// Which piece of bonding data was asked for, and on which connection. The vendor's types are small integers; the
/// exact meaning is in `sl_bt_api.h` under `sl_bt_evt_external_bondingdb_data_request`.
static LAST_DATA_TYPE: AtomicU32 = AtomicU32::new(0xFFFF_FFFF);
static LAST_DATA_CONN: AtomicU32 = AtomicU32::new(0xFFFF_FFFF);
/// The stack's "I have all the bonding data I asked for" signal. Its absence is the difference between "the stack is
/// still waiting for us" and "the stack moved on and failed elsewhere".
static DATA_READY_EVENTS: AtomicU32 = AtomicU32::new(0);

/// Called when the stack asks for the bonding data of a connection.
pub fn note_data_request(connection: u8, data_type: u8) {
    CALLS_DATA_REQ.fetch_add(1, Ordering::Relaxed);
    trace(1, data_type as u32, connection as u32);
    LAST_DATA_CONN.store(connection as u32, Ordering::Relaxed);
    LAST_DATA_TYPE.store(data_type as u32, Ordering::Relaxed);
}

/// The peer address of each open connection, as the stack reported it on `connection_opened`.
///
/// With an external database this bookkeeping is the application's job, and the vendor's own contract says so
/// (`sl_bt_api.h:11580`): "Manage a local RAM-based bookkeeping to keep track of open connections. At a minimum, the
/// remote Bluetooth address and address type must be stored in the bookkeeping when a
/// `sl_bt_evt_connection_opened` event is received." It exists here for one reason: the stack asks for bonding data
/// by **connection**, and the bond is identified by the **peer address** stored with it, so without the table there
/// is no way to tell which bond a request is about. Measured 2026-11-17: answering every request with zero length
/// (the previous behaviour) means encryption can never be established from stored data.
static CONNECTION_PEERS: [AtomicU32; 8] = [
    AtomicU32::new(0xFFFF_FFFF),
    AtomicU32::new(0xFFFF_FFFF),
    AtomicU32::new(0xFFFF_FFFF),
    AtomicU32::new(0xFFFF_FFFF),
    AtomicU32::new(0xFFFF_FFFF),
    AtomicU32::new(0xFFFF_FFFF),
    AtomicU32::new(0xFFFF_FFFF),
    AtomicU32::new(0xFFFF_FFFF),
];

/// Pack a 6-byte address and its type into one word: type in the top byte, the first five address bytes below it.
fn pack_peer(address: [u8; 6], address_type: u8) -> u32 {
    (address_type as u32) << 24
        | (address[0] as u32) << 16
        | (address[1] as u32) << 8
        | address[2] as u32
}

/// Remember the peer of a connection, from `connection_opened`.
pub fn note_connection(connection: u8, address: [u8; 6], address_type: u8) {
    if let Some(slot) = CONNECTION_PEERS.get(connection as usize) {
        slot.store(pack_peer(address, address_type), Ordering::Relaxed);
    }
}

/// Forget a connection, from `connection_closed`.
pub fn forget_connection(connection: u8) {
    if let Some(slot) = CONNECTION_PEERS.get(connection as usize) {
        slot.store(0xFFFF_FFFF, Ordering::Relaxed);
    }
}

/// How many bytes the stack stored for `(handle, piece)`, or `None` when nothing is stored.
///
/// # Safety
/// Reads the NVM3 object for that pair; the handle must be the stack's NVM3 handle.
unsafe fn stored_len(handle: u8, piece: u8) -> Option<usize> {
    let mut obj_type = 0u32;
    let mut len = 0usize;
    let rc = nvm3_getObjectInfo(
        nvm3_handle(),
        storage_key(handle, piece),
        &mut obj_type,
        &mut len,
    );
    if rc != 0 || len == 0 {
        None
    } else {
        Some(len)
    }
}

/// Copy the piece the stack stored for `(handle, piece)` into `out`, returning how many bytes were copied.
///
/// # Safety
/// As for `stored_len`.
unsafe fn copy_stored(handle: u8, piece: u8, out: &mut [u8]) -> Option<usize> {
    let len = stored_len(handle, piece)?;
    if len > out.len() {
        return None;
    }
    let rc = nvm3_readData(
        nvm3_handle(),
        storage_key(handle, piece),
        out.as_mut_ptr(),
        len,
    );
    if rc != 0 {
        None
    } else {
        Some(len)
    }
}

/// Delete every bond this database holds, and say how many objects were removed.
///
/// With an external database the stack cannot delete bonds for the application: `sl_bt_sm_delete_bonding` and the whole
/// family answer `SL_STATUS_NOT_AVAILABLE` (`sl_bt_api.h:11605-11610`), and "The application also needs to manage the
/// bondings in the storage, e.g., delete a bonding when requested by the business logic." Measured 2026-11-17: a test
/// phase that asked the stack to clear the device cleared nothing on this path, the device kept answering the next
/// pairing with the previous long-term key, and the host cancelled.
pub fn clear_all() -> u32 {
    let mut removed = 0;
    // SAFETY: single-core; NVM3 is opened by the stack and this only enumerates and deletes within our own key range.
    unsafe {
        let mut keys = [0u32; 48];
        let found = nvm3_enumObjects(nvm3_handle(), keys.as_mut_ptr(), keys.len());
        for key in keys.iter().take(found.min(keys.len())) {
            if *key / 0x1_0000 != KEY_BASE / 0x1_0000 {
                continue;
            }
            if nvm3_deleteObject(nvm3_handle(), *key) == 0 {
                removed += 1;
            }
        }
    }
    removed
}

/// The bond handle whose stored peer address matches the peer of `connection`.
///
/// The address the stack reports on the connection is what the bond was stored under: `bondingdb` keeps the remote
/// address as piece 0. Comparing the two is how a request for "the long-term key of connection 1" becomes "the
/// long-term key of bond N".
///
/// # Safety
/// Reads NVM3 objects for handles `0..MAX_SLOTS`.
unsafe fn bond_for_connection(connection: u8) -> Option<u8> {
    let packed = CONNECTION_PEERS
        .get(connection as usize)?
        .load(Ordering::Relaxed);
    if packed == 0xFFFF_FFFF {
        return None;
    }
    let mut stored = [0u8; 8];
    for slot in 0..MAX_SLOTS {
        let Some(len) = copy_stored(slot, 0, &mut stored) else {
            continue;
        };
        // Piece 0 is the remote address: six bytes, little-endian as the stack reports it.
        if len < 6 {
            continue;
        }
        let address = [
            stored[0], stored[1], stored[2], stored[3], stored[4], stored[5],
        ];
        if pack_peer(address, 0) >> 8 == packed >> 8 {
            return Some(slot);
        }
    }
    None
}

/// The bonding data of `piece` for the bond of `connection`, copied into `out`.
///
/// `None` means "this stack has no such data", which the caller answers with a zero-length
/// `sl_bt_external_bondingdb_set_data` -- the documented way to say "not available" (`sl_bt_api.h:11887`).
pub fn piece_for(connection: u8, piece: u8, out: &mut [u8]) -> Option<usize> {
    // SAFETY: single-core, NVM3 is opened by the stack and this only reads.
    unsafe {
        let handle = bond_for_connection(connection)?;
        copy_stored(handle, piece, out)
    }
}

/// A compact trace of the bonding conversation: eight entries of `(kind << 16) | (a << 8) | b`.
///
/// kinds: 1 = the stack asked for data (a=type, b=connection), 2 = we answered (a=type, b=status),
/// 3 = the stack stored a piece (a=type, b=length), 4 = the stack read a piece (a=type, b=1 if found),
/// 5 = the stack announced data_ready.
static TRACE: [AtomicU32; 8] = [
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
];
static TRACE_NEXT: AtomicU32 = AtomicU32::new(0);

fn trace(kind: u32, a: u32, b: u32) {
    let index = (TRACE_NEXT.fetch_add(1, Ordering::Relaxed) % 8) as usize;
    TRACE[index].store(
        (kind << 16) | ((a & 0xFF) << 8) | (b & 0xFF),
        Ordering::Relaxed,
    );
}

/// The eight most recent entries, oldest first, as `kind/a/b` numbers packed in one word each.
pub fn trace_entries() -> [u32; 8] {
    let mut out = [0u32; 8];
    let written = TRACE_NEXT.load(Ordering::Relaxed) as usize;
    if written == 0 {
        return out;
    }
    // Oldest first: with fewer than eight entries the ring has not wrapped, so the oldest is index 0 and there is
    // nothing after `written`. Getting this wrong made the whole trace print as empty (measured 2026-11-17).
    let count = written.min(8);
    let start = if written > 8 { written % 8 } else { 0 };
    for i in 0..count {
        out[i] = TRACE[(start + i) % 8].load(Ordering::Relaxed);
    }
    out
}

/// A bonding-data request waiting to be answered from a safe context.
static QUEUED_DATA: AtomicU32 = AtomicU32::new(0xFFFF_FFFF);

/// Record a request to be answered on the next poll, outside any BGAPI call.
pub fn queue_data_answer(connection: u8, data_type: u8) {
    QUEUED_DATA.store(
        ((connection as u32) << 8) | data_type as u32,
        Ordering::Relaxed,
    );
}

/// Take the queued request, if any.
pub fn take_queued_data_answer() -> Option<(u8, u8)> {
    let packed = QUEUED_DATA.swap(0xFFFF_FFFF, Ordering::Relaxed);
    if packed == 0xFFFF_FFFF {
        None
    } else {
        Some(((packed >> 8) as u8, (packed & 0xFF) as u8))
    }
}

/// Called when the stack announces that it has all the bonding data it asked for.
pub fn note_data_ready() {
    DATA_READY_EVENTS.fetch_add(1, Ordering::Relaxed);
    trace(5, 0, 0);
}

/// Called after answering that request, with the length answered and the status the stack returned.
pub fn note_data_answer(length: usize, status: u32) {
    trace(2, length as u32, status);
    CALLS_DATA_ANSWERED.fetch_add(1, Ordering::Relaxed);
    trace(2, 0, status);
    LAST_DATA_STATUS.store(status, Ordering::Relaxed);
}

/// `(requests, answered, last status, last type, last connection)` of the bonding-data conversation.
pub fn data_request_stats() -> (u32, u32, u32, u32, u32, u32) {
    (
        CALLS_DATA_REQ.load(Ordering::Relaxed),
        CALLS_DATA_ANSWERED.load(Ordering::Relaxed),
        LAST_DATA_STATUS.load(Ordering::Relaxed),
        LAST_DATA_TYPE.load(Ordering::Relaxed),
        LAST_DATA_CONN.load(Ordering::Relaxed),
        DATA_READY_EVENTS.load(Ordering::Relaxed),
    )
}

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
    // The vendor's own defaults answer 1 from the builtin database and 2 from the external one. Answering 1 here --
    // which is what this did until it was measured -- tells the stack that the builtin database is running while the
    // bridge is in place, and the pairing then fails authentication with nothing in the record to say why.
    2
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
                nvm3_handle(),
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
    // The record is there when NVM3 holds it. The vendor's external default answers `(handle + 1) != 0` -- true for
    // every handle but 0xFF -- and copying that was a real defect: the stack asked "is slot 0 bonded?", was told yes,
    // concluded a bond already existed and never ran the pairing exchange (measured: ec=0,0,0 and no crypto at all).
    let mut obj_type = 0u32;
    let mut length = 0usize;
    unsafe {
        nvm3_getObjectInfo(
            nvm3_handle(),
            storage_key(bonding, 8),
            &mut obj_type,
            &mut length,
        ) == 0
    }
}

/// Store one piece of a bonding record. 0 is success; 0x406 is the status the internal implementation returns for a
/// handle it does not recognise.
///
/// # Safety
///
/// `data` must point to `length` readable bytes.
#[no_mangle]
// The argument order is the CALLER's, and it is not the one that reads naturally: the stack passes
// `(handle, type, length, data)`. Measured in the blob, at the site that fetches the identity address:
//
//     mov  r3, r4        ; data
//     movs r2, #6        ; length
//     movs r1, #1        ; type
//     mov  r0, r5        ; handle
//     bl   sli_bt_bonding_get_data
//
// Declaring `(handle, type, data, length)` instead -- which is how this file first had it -- makes every store read
// its length from the data pointer and every read write `data` bytes to the address `length`: garbage in, and a heap
// write to a wild address out.
pub unsafe extern "C" fn sli_bt_external_bonding_store_data(
    bonding: u8,
    piece: u8,
    length: usize,
    data: *const u8,
) -> u32 {
    CALLS_STORE.fetch_add(1, Ordering::Relaxed);
    trace(3, piece as u32, length as u32);
    if data.is_null() || length == 0 || length > 128 {
        return 0x406;
    }
    let rc = nvm3_writeData(nvm3_handle(), storage_key(bonding, piece), data, length);
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
    nvm3_readData(nvm3_handle(), storage_key(bonding, piece), data, length)
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
            nvm3_deleteObject(nvm3_handle(), storage_key(bonding, piece));
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
    length: usize,
    data: *mut u8,
) -> u32 {
    CALLS_READ.fetch_add(1, Ordering::Relaxed);
    trace(4, piece as u32, 0);
    if data.is_null() || length == 0 {
        return 0x406;
    }
    let rc = nvm3_readData(nvm3_handle(), storage_key(bonding, piece), data, length);
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
