//! PSA Crypto, the part the Bluetooth host actually uses: key slots and CMAC.
//!
//! The host stack imports its AES keys through PSA and asks PSA for a CMAC. That is the whole surface it needs
//! from this file, and it is implemented for real: keys live in a slot table, and the MAC is AES-CMAC from
//! `libraries/aes-modes`, validated against RFC 4493.
//!
//! # What is deliberately not here
//!
//! **`psa_generate_random` returns `PSA_ERROR_NOT_SUPPORTED`.** This part's device headers declare no TRNG
//! (`grep TRNG` over `vendor445/device/*.h` finds nothing), and a counter, a timer or an LFSR is not entropy.
//! Returning bytes from one would let the stack seed keys with a value an observer can predict, while every test
//! in the tree stayed green — the definition of a false green. A refusal is a visible boundary; a fake RNG is a
//! silent vulnerability in a shipped product. If the boot path needs randomness, this is where it stops, and
//! that is information, not a regression.
//!
//! **Key attributes are not parsed.** `psa_key_attributes_t` is opaque and its layout depends on the mbedtls
//! configuration (`232` bytes as measured for this project's headers). Reading fields out of it would be a guess
//! dressed as code; the key size comes from `data_length` instead, and the key type is whatever
//! `psa_set_key_domain_parameters` recorded, defaulting to AES. The stack's own call site in the host blob does
//! `psa_set_key_domain_parameters(attrs, 0x2400 /* PSA_KEY_TYPE_AES */, NULL, 0)`, which is the case implemented.
//!
//! **Key ids are 32-bit.** `mbedtls_svc_key_id_t` is `psa_key_id_t` (a `u32`) unless
//! `MBEDTLS_PSA_CRYPTO_KEY_ID_ENCODES_OWNER` is defined; the blobs contain no owner-encoding helpers, so this
//! writes four bytes and treats every id it receives as a `u32`.
//!
//! # Operation state lives outside the caller's struct
//!
//! `psa_mac_operation_t` is opaque and this crate does not know its size. State is therefore kept in a table
//! keyed by the operation pointer the caller passes, which is stable for the lifetime of an operation and
//! requires no assumption about the struct's layout.

use core::ptr;
use core::sync::atomic::{AtomicU32, Ordering};

use aes_modes::cmac;

/// `PSA_SUCCESS`
const PSA_SUCCESS: i32 = 0;
/// `PSA_ERROR_NOT_SUPPORTED`
const PSA_ERROR_NOT_SUPPORTED: i32 = -134;
/// `PSA_ERROR_INVALID_ARGUMENT`
const PSA_ERROR_INVALID_ARGUMENT: i32 = -135;

/// `PSA_ERROR_HARDWARE_FAILURE`: the entropy source failed. The buffer is zeroed and nothing is invented.
const PSA_ERROR_HARDWARE_FAILURE: i32 = -147;
/// `PSA_ERROR_INVALID_HANDLE`
const PSA_ERROR_INVALID_HANDLE: i32 = -136;
/// `PSA_ERROR_BAD_STATE`
const PSA_ERROR_BAD_STATE: i32 = -137;
/// `PSA_ERROR_BUFFER_TOO_SMALL`
const PSA_ERROR_BUFFER_TOO_SMALL: i32 = -138;
/// `PSA_ERROR_DOES_NOT_EXIST`
const PSA_ERROR_DOES_NOT_EXIST: i32 = -140;

/// `PSA_KEY_TYPE_AES`, the only key type this firmware handles.
const PSA_KEY_TYPE_AES: u32 = 0x2400;

/// Slots. Sixteen AES-128 keys is more than a peripheral needs (one for the link layer, one per bonded peer).
const SLOTS: usize = 16;
/// Longest key this accepts, in bytes. AES-256 would be 32; the link layer uses 16. Raised to 65 for the unwrapped
/// secp256r1 public point (`0x04 || X || Y`), which the stack imports to validate a peer's key before any agreement.
const MAX_KEY: usize = 65;
/// `PSA_KEY_TYPE_ECC_PUBLIC_KEY(PSA_ECC_FAMILY_SECP_R1)` = 0x4112: what the stack passes when it imports the peer's
/// public key for validation.
const PSA_KEY_TYPE_ECC_PUBLIC_KEY_P256: u32 = 0x4112;
/// Largest message a single MAC operation may accumulate. A BLE CMAC covers a handful of bytes; 128 is generous
/// and bounded on purpose, because a MAC operation that can queue unbounded input is a memory bug waiting.
const MAX_MAC_INPUT: usize = 128;
/// Concurrent MAC operations. The stack runs one at a time; two allows for a retry in flight.
const MAC_OPS: usize = 2;

#[derive(Clone, Copy)]
pub(crate) struct Slot {
    pub(crate) used: bool,
    /// The handle this slot was handed out under. Stored, because handles are a monotonic counter and slot positions
    /// are reused: after one destroy, `handle n` is no longer `slot n - 1`, and the previous version of this file
    /// guessed the mapping from position.
    pub(crate) id: u32,
    pub(crate) key_len: usize,
    pub(crate) key: [u8; MAX_KEY],
    /// `psa_key_type_t` recorded at import. Read by nothing yet — this firmware only holds AES keys — but stored
    /// so that a future key type is a table change rather than a redesign.
    pub(crate) key_type: u32,
}

const EMPTY_SLOT: Slot = Slot {
    used: false,
    id: 0,
    key_len: 0,
    key: [0; MAX_KEY],
    key_type: PSA_KEY_TYPE_AES,
};

// THE TABLE IS IN RAM, AND THAT IS THE NEXT DEFECT TO CLOSE.
//
// The vendor's host library ships two bonding databases. The builtin one keeps records in NVM3 and, measured on this
// board, writes the record and not the long-term key, so a peer that reconnects after a reset is answered 0x1006 PIN
// or Key Missing (docs/04-reference/PAIRING-D2F-STATE.md). The other one -- the object linked when the feature
// `external-bonding` is on -- is the PSA-key-backed database: it keeps the bond, long-term key included, as a PSA
// key, which means it keeps it *here*.
//
// Which is why this table matters beyond this file: it is RAM only, so with the PSA database selected a bond has
// nowhere to survive a reset, and the pairing that writes through it does not complete.
//
// The option, written down so the next session does not have to rediscover it:
//
//   1. one NVM3 object per key, under a key range of our own (`0xD0000 | id`, clear of the stack's keys and of the
//      bonding records at `0xE0000`), holding `key_type`, `key_len` and the key bytes
//   2. `psa_import_key`, `psa_generate_key` and `psa_destroy_key` write and delete the object, so the flash always
//      agrees with the table
//   3. `slot_for` loads a key from flash on a miss, so a handle that survived a reset resolves again
//   4. `NEXT_ID` is restored at start-up by enumerating our range and taking the highest id, because ids are handed
//      out from a counter and a counter that restarts at 1 would overwrite a persisted key
//
// Then: turn `external-bonding` on, pair from the host, reset the board, reconnect without pairing. That last step is
// the proof the whole BLE requirement rests on, and it is why this is written here rather than in a note.
static mut SLOTS_TABLE: [Slot; SLOTS] = [EMPTY_SLOT; SLOTS];
/// Key id 0 is `PSA_KEY_ID_NULL` and must never be handed out; ids start at 1.
static mut NEXT_ID: u32 = 1;
/// Keys destroyed through `psa_destroy_key`. A bond deleted on the device must reach this counter, otherwise the key
/// survives under the identifier the stack names and the next pairing answers with it.
static DESTROYED: AtomicU32 = AtomicU32::new(0);

/// The identifiers destroyed, oldest first, four deep: which keys the stack removes at boot is the question that the
/// `del=` counter alone cannot answer.
static DESTROYED_IDS: [AtomicU32; 4] = [
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
];
static DESTROYED_NEXT: AtomicU32 = AtomicU32::new(0);

/// The last four destroyed identifiers, oldest first.
pub fn destroyed_ids() -> [u32; 4] {
    let written = DESTROYED_NEXT.load(Ordering::Relaxed) as usize;
    let mut out = [0u32; 4];
    if written == 0 {
        return out;
    }
    let count = written.min(4);
    let start = if written > 4 { written % 4 } else { 0 };
    for i in 0..count {
        out[i] = DESTROYED_IDS[(start + i) % 4].load(Ordering::Relaxed);
    }
    out
}

/// How many keys have been destroyed.
pub fn destroy_count() -> u32 {
    DESTROYED.load(Ordering::Relaxed)
}

/// The key lookups that found nothing, and what the last one asked for. A miss at reconnect is the whole of the
/// "PIN or Key Missing" symptom, seen from our side of the ABI.
static SLOT_MISSES: AtomicU32 = AtomicU32::new(0);
static LAST_MISS_ID: AtomicU32 = AtomicU32::new(0);
/// Keys restored from flash at start-up, and the last id restored.
/// How many restore attempts have been made this boot. The enumeration can legitimately return nothing if it runs
/// before the stack has populated the NVM3 pages (measured: `rcal=1/0/0/0` at boot against `5` objects once running),
/// so a lookup that misses is allowed to trigger one more attempt instead of trusting the first window.
/// Set once the key table has been restored from flash for this boot. Module scope on purpose: a function-local
/// `static mut` is a separate variable per function, and clearing it from a helper silently failed.
static mut RESTORE_DONE: bool = false;

static RESTORE_RUNS: AtomicU32 = AtomicU32::new(0);
/// The attempts allowed per boot. Small on purpose: this is a fallback for a bad moment, not a polling loop.
const RESTORE_MAX_RUNS: u32 = 4;

/// Forget the "already restored" mark so the next miss can try again. Safe to call from a lookup.
fn allow_another_restore() {
    if RESTORE_RUNS.load(Ordering::Relaxed) >= RESTORE_MAX_RUNS {
        return;
    }
    // SAFETY: single-core, and the flag only guards a table walk that is not re-entered.
    unsafe { *ptr::addr_of_mut!(RESTORE_DONE) = false }
}

/// Why the restore did or did not happen: how many times it was attempted, how many of those saw a null NVM3 handle,
/// how many objects the enumeration returned, and how many of them fell in our key range.
static RESTORE_CALLS: AtomicU32 = AtomicU32::new(0);
static RESTORE_NULL_HANDLE: AtomicU32 = AtomicU32::new(0);
static RESTORE_ENUMERATED: AtomicU32 = AtomicU32::new(0);
static RESTORE_IN_RANGE: AtomicU32 = AtomicU32::new(0);

/// `(calls, null handle, enumerated, in range)` of the restore.
pub fn restore_stats() -> (u32, u32, u32, u32) {
    (
        RESTORE_CALLS.load(Ordering::Relaxed),
        RESTORE_NULL_HANDLE.load(Ordering::Relaxed),
        RESTORE_ENUMERATED.load(Ordering::Relaxed),
        RESTORE_IN_RANGE.load(Ordering::Relaxed),
    )
}
static SLOTS_RESTORED: AtomicU32 = AtomicU32::new(0);
static LAST_RESTORED_ID: AtomicU32 = AtomicU32::new(0);

/// The used slots, as `(id, key_type)` pairs, for a test that needs to compare our identifiers with the ones the
/// stack asks for.
pub fn table_summary() -> [(u32, u32); 6] {
    let mut out = [(0u32, 0u32); 6];
    let mut n = 0;
    for slot in slots().iter() {
        if !slot.used || n >= out.len() {
            continue;
        }
        out[n] = (slot.id, slot.key_type);
        n += 1;
    }
    out
}

/// `(misses, last missed id, restored, last restored id)`.
pub fn lookup_stats() -> (u32, u32, u32, u32) {
    (
        SLOT_MISSES.load(Ordering::Relaxed),
        LAST_MISS_ID.load(Ordering::Relaxed),
        SLOTS_RESTORED.load(Ordering::Relaxed),
        LAST_RESTORED_ID.load(Ordering::Relaxed),
    )
}
/// Diagnostics: how many imports, and whether randomness was ever requested.
static mut IMPORTS: u32 = 0;
static mut RANDOM_REQUESTS: u32 = 0;

/// One MAC operation in flight.
#[derive(Clone, Copy)]
struct MacOp {
    active: bool,
    /// The caller's operation struct address, used as the key into this table.
    owner: usize,
    /// Slot index, or `usize::MAX` when the key id was unknown.
    slot: usize,
    input_len: usize,
    input: [u8; MAX_MAC_INPUT],
}

const EMPTY_OP: MacOp = MacOp {
    active: false,
    owner: 0,
    slot: usize::MAX,
    input_len: 0,
    input: [0; MAX_MAC_INPUT],
};

static mut MAC_TABLE: [MacOp; MAC_OPS] = [EMPTY_OP; MAC_OPS];

// ---- keeping the key table across a reset --------------------------------------------------------------
//
// Written up at `SLOTS_TABLE` below; this is the machinery. One NVM3 object per key, in a range of our own, holding
// the type, the length and the bytes. The flash is written whenever the table changes, so the two never disagree.

/// Key range for persisted keys: clear of the stack's own keys and of the bonding records at `0xE0000`.
const PERSIST_BASE: u32 = 0xD_0000;
/// A persisted key: `id` (4) + `key_type` (4) + `key_len` (2) + bytes.
///
/// The id is stored in the payload, not encoded in the NVM3 key. NVM3 keys are 20 bits and the stack names keys by
/// vendor identifiers such as `0x48003`, so `PERSIST_BASE | (id % 0x10000)` silently turned `0x48003` into `0x8003`
/// and every restore handed the key back under a name nobody asked for (measured 2026-11-17: `rst=9` and still
/// `miss=0x48003` after a reset).
const PERSIST_HEADER: usize = 10;

/// The first eight 32-bit words of the last attributes blob seen by `psa_import_key`. The stack puts the key id in
/// there; reading what arrives beats inventing an offset from a header that is not in the tree.
static mut ATTR_WORDS: [u32; 8] = [0; 8];

/// Words of the attributes of the last import whose payload was 16 bytes, i.e. a long-term key.
static mut LTK_ATTR_WORDS: [u32; 8] = [0; 8];
static mut LTK_ATTR_SEEN: u32 = 0;

/// The attributes words of the last 16-byte import, and how many such imports were seen.
pub fn ltk_attributes() -> ([u32; 8], u32) {
    // SAFETY: single-core; written on the import path, read from the main loop.
    unsafe {
        (
            *ptr::addr_of!(LTK_ATTR_WORDS),
            *ptr::addr_of!(LTK_ATTR_SEEN),
        )
    }
}

/// The captured words, for a test that needs to locate the id field.
pub fn attributes_words() -> [u32; 8] {
    // SAFETY: single-core; written from the import path, read from the main loop.
    unsafe { *ptr::addr_of!(ATTR_WORDS) }
}

/// Copy the attributes of an import whose payload is 16 bytes: that is the long-term key, and it is the one the
/// security manager later asks for by an id derived from the bond handle.
///
/// # Safety
/// As for `capture_attributes`.
unsafe fn capture_ltk_attributes(attributes: *const u8, data_length: usize) {
    if attributes.is_null() || data_length != 16 {
        return;
    }
    for (i, slot) in (*ptr::addr_of_mut!(LTK_ATTR_WORDS)).iter_mut().enumerate() {
        *slot = core::ptr::read_unaligned(attributes.add(i * 4).cast::<u32>());
    }
    *ptr::addr_of_mut!(LTK_ATTR_SEEN) += 1;
}

/// Copy the first words of the attributes blob.
///
/// # Safety
/// `attributes` must be a readable blob of at least 32 bytes, which is what the PSA API guarantees for
/// `psa_key_attributes_t`.
unsafe fn capture_attributes(attributes: *const u8) {
    if attributes.is_null() {
        return;
    }
    for (i, slot) in (*ptr::addr_of_mut!(ATTR_WORDS)).iter_mut().enumerate() {
        *slot = core::ptr::read_unaligned(attributes.add(i * 4).cast::<u32>());
    }
}

extern "C" {
    static mut nvm3_defaultHandle: *mut u8;
    fn nvm3_writeData(handle: *mut u8, key: u32, data: *const u8, length: usize) -> u32;
    fn nvm3_readData(handle: *mut u8, key: u32, data: *mut u8, length: usize) -> u32;
    fn nvm3_deleteObject(handle: *mut u8, key: u32) -> u32;
    fn nvm3_enumObjects(handle: *mut u8, key_list: *mut u32, key_list_size: usize) -> usize;
    fn nvm3_getObjectInfo(handle: *mut u8, key: u32, obj_type: *mut u32, length: *mut usize)
        -> u32;
}

fn persist_key_for(id: u32) -> u32 {
    PERSIST_BASE | (id % 0x1_0000)
}

/// Write one slot to flash. Called on every change, so a key that exists in the table always exists on the device.
fn persist_slot(slot: &Slot) {
    if !slot.used || slot.id == 0 || slot.key_len == 0 {
        return;
    }
    let mut buf = [0u8; PERSIST_HEADER + MAX_KEY];
    buf[..4].copy_from_slice(&slot.id.to_le_bytes());
    buf[4..8].copy_from_slice(&slot.key_type.to_le_bytes());
    buf[8..10].copy_from_slice(&(slot.key_len as u16).to_le_bytes());
    buf[PERSIST_HEADER..PERSIST_HEADER + slot.key_len].copy_from_slice(&slot.key[..slot.key_len]);
    // SAFETY: the buffer outlives the call and the length is its own.
    unsafe {
        nvm3_writeData(
            nvm3_defaultHandle,
            persist_key_for(slot.id),
            buf.as_ptr(),
            PERSIST_HEADER + slot.key_len,
        );
    }
}

/// Remove one slot from flash.
fn forget_slot(id: u32) {
    // SAFETY: deleting a key that is not there is not an error in NVM3.
    unsafe {
        nvm3_deleteObject(nvm3_defaultHandle, persist_key_for(id));
    }
}

/// Load every persisted key back into the table, once per boot, and restore the id counter above them.
///
/// The counter matters as much as the keys: ids are handed out from `NEXT_ID`, which starts at 1 on every boot, so
/// without this a new import would take the id of a key that survived and overwrite it.
fn restore_slots_once() {
    // SAFETY: single-core, called from the PSA entry points before they touch the table, and guarded by RESTORE_DONE.
    unsafe {
        if *ptr::addr_of!(RESTORE_DONE) {
            return;
        }
        // The stack opens the NVM3 default instance itself, and the first PSA call can arrive before it has. Marking
        // the restore as done in that state made the whole persistence silent: measured 2026-11-17, `rst=0/0` after a
        // reset while the keys were in the flash, and the stack then missed key 0x40037 and the link dropped with
        // 0x1006. So: only take `DONE` when the handle is usable, and retry on the next call otherwise.
        RESTORE_CALLS.fetch_add(1, Ordering::Relaxed);
        if nvm3_defaultHandle.is_null() {
            RESTORE_NULL_HANDLE.fetch_add(1, Ordering::Relaxed);
            return;
        }
        *ptr::addr_of_mut!(RESTORE_DONE) = true;
        RESTORE_RUNS.fetch_add(1, Ordering::Relaxed);

        let mut keys = [0u32; 24];
        let found = nvm3_enumObjects(nvm3_defaultHandle, keys.as_mut_ptr(), keys.len());
        RESTORE_ENUMERATED.store(found as u32, Ordering::Relaxed);
        let mut highest = 0u32;
        for key in keys.iter().take(found.min(keys.len())) {
            // Same range as `PERSIST_BASE`, compared by the high half so this does not depend on a bit mask.
            if *key / 0x1_0000 != PERSIST_BASE / 0x1_0000 {
                continue;
            }
            RESTORE_IN_RANGE.fetch_add(1, Ordering::Relaxed);
            // Ask the object how long it is before reading it: `nvm3_readData` refuses a length larger than the
            // object, and asking for the full buffer (header + MAX_KEY) against a shorter object failed every read.
            // Measured 2026-11-17: four keys in range, zero restored, because of exactly this.
            let mut obj_type = 0u32;
            let mut obj_len = 0usize;
            if nvm3_getObjectInfo(nvm3_defaultHandle, *key, &mut obj_type, &mut obj_len) != 0 {
                continue;
            }
            if !(PERSIST_HEADER + 1..=PERSIST_HEADER + MAX_KEY).contains(&obj_len) {
                continue;
            }
            let mut buf = [0u8; PERSIST_HEADER + MAX_KEY];
            let rc = nvm3_readData(nvm3_defaultHandle, *key, buf.as_mut_ptr(), obj_len);
            if rc != 0 {
                continue;
            }
            let id = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
            let key_type = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
            let key_len = u16::from_le_bytes([buf[8], buf[9]]) as usize;
            if key_len == 0 || key_len > MAX_KEY {
                continue;
            }
            // `id` comes from the payload above: the NVM3 key only identifies the object.
            let table = slots();
            if let Some(free) = table.iter_mut().find(|s| !s.used) {
                let mut bytes = [0u8; MAX_KEY];
                bytes[..key_len].copy_from_slice(&buf[PERSIST_HEADER..PERSIST_HEADER + key_len]);
                *free = Slot {
                    used: true,
                    id,
                    key_len,
                    key: bytes,
                    key_type,
                };
                highest = highest.max(id);
                SLOTS_RESTORED.fetch_add(1, Ordering::Relaxed);
                LAST_RESTORED_ID.store(id, Ordering::Relaxed);
            }
        }
        if highest >= *ptr::addr_of!(NEXT_ID) {
            *ptr::addr_of_mut!(NEXT_ID) = highest + 1;
        }
    }
}

fn slots() -> &'static mut [Slot; SLOTS] {
    // SAFETY: single-core, no interrupts touch this table (the timer is polled and the stack runs in the main
    // loop), and every mutation happens inside a function that is not re-entered.
    unsafe { &mut *ptr::addr_of_mut!(SLOTS_TABLE) }
}

fn ops() -> &'static mut [MacOp; MAC_OPS] {
    // SAFETY: as `slots`.
    unsafe { &mut *ptr::addr_of_mut!(MAC_TABLE) }
}

/// Initialise PSA. Nothing to do: the slot table is already in `.bss` and empty.
#[no_mangle]
pub extern "C" fn psa_crypto_init() -> i32 {
    bump(0);
    PSA_SUCCESS
}

/// Hardware entropy, from the Secure Engine's TRNG.
///
/// **This function used to refuse.** Its doc claimed no TRNG was declared for this device; the premise was wrong, and
/// a read-only review of this tree found it: the device header defines `SEMAILBOX_PRESENT`, and the vendor's own
/// `sl_se_get_random` on this exact part issues `SLI_SE_COMMAND_TRNG_GET_RANDOM` through the mailbox. That refusal is
/// what made `sli_bt_system_start_bluetooth` return `0x040C` (`SL_STATUS_BT_RADIO`): `ll_randomInit` asks for 32
/// bytes, `bg_random_get_data` maps any failure to `0x0407`, and the radio wrapper masks it with `0xFF`.
///
/// The rule that produced the refusal still holds, and is now honoured rather than dodged: never invent randomness.
/// On failure this returns `PSA_ERROR_HARDWARE_FAILURE` with the buffer zeroed, and there is no fallback to a
/// counter or a timer. `se_trng` documents the protocol and the provenance of every constant in it.
///
/// # Safety
///
/// `output` must point to `output_size` writable bytes in SRAM.
#[no_mangle]
pub unsafe extern "C" fn psa_generate_random(output: *mut u8, output_size: usize) -> i32 {
    bump(1);
    let p = ptr::addr_of_mut!(RANDOM_REQUESTS);
    ptr::write_volatile(p, ptr::read_volatile(p) + 1);
    crate::se_trng::count_request();

    if output.is_null() {
        return PSA_ERROR_INVALID_ARGUMENT;
    }
    if output_size == 0 {
        return 0;
    }

    let buf = core::slice::from_raw_parts_mut(output, output_size);
    match crate::se_trng::fill(buf) {
        Ok(()) => 0,
        // `fill` has already zeroed the buffer. Counts of requests/retries/failures live in `se_trng::stats`; the
        // bytes themselves are never recorded.
        Err(_) => PSA_ERROR_HARDWARE_FAILURE,
    }
}

/// Reset an attributes structure to the all-zero state PSA defines for it.
///
/// The zeroing is done through the caller's pointer for the size this project measured (232 bytes), because
/// leaving an attributes struct half-filled is how a later `psa_import_key` acquires a usage flag nobody set.
/// The size is a constant here rather than a guess at a field: zeroing a buffer the caller owns, to a size this
/// project measured from the same headers the stack compiled against.
///
/// # Safety
///
/// `attributes` must point to a `psa_key_attributes_t` (232 bytes for this configuration).
#[no_mangle]
pub unsafe extern "C" fn psa_reset_key_attributes(attributes: *mut u8) {
    bump(8);
    if attributes.is_null() {
        return;
    }
    ptr::write_bytes(attributes, 0, PSA_KEY_ATTRIBUTES_BYTES);
}

/// The identifier the caller asked for, if it asked for one.
///
/// Measured 2026-11-17 by disassembling the stack's own bonding code: `bondingdb_psa_add_ecb_key` builds
/// `0x48000 | (handle << 8) | type`, writes it into the attributes blob at **offset 8** (`str r6, [r4, #8]`) and then
/// calls `psa_import_key`; the security manager later asks PSA for exactly that identifier. Ignoring it -- handing out
/// our own small id instead -- made every post-reset lookup miss and the link drop with HCI 0x1006.
///
/// # Safety
///
/// `attributes` is either null or a readable attributes blob; only offset 8 is read.
unsafe fn requested_id(attributes: *const u8) -> u32 {
    if attributes.is_null() {
        return 0;
    }
    core::ptr::read_unaligned(attributes.add(8).cast::<u32>())
}

/// The id a new key takes: the caller's if it named one, otherwise the next counter value.
///
/// # Safety
///
/// As for `requested_id`.
unsafe fn take_id(attributes: *const u8) -> u32 {
    let asked = requested_id(attributes);
    if asked != 0 {
        return asked;
    }
    let counter = ptr::addr_of_mut!(NEXT_ID);
    let value = ptr::read_volatile(counter);
    ptr::write_volatile(counter, value.wrapping_add(1).max(1));
    value.max(1)
}

/// `sizeof(psa_key_attributes_t)` **as this implementation reports it to the allocator**.
///
/// This must equal `sli_psa_context_get_size(6)` in `support.rs`: that call is what sizes the heap block the stack
/// hands us, and everything written into it has to fit. It was 232 while the allocator was told 64, so every reset
/// wrote 168 bytes past the block -- heap corruption in code nobody looks at, which is the shape of the unspecified
/// SMP failures seen with the external bonding database (measured and named by the review of 2026-11-17).
const PSA_KEY_ATTRIBUTES_BYTES: usize = 64;

/// Record a key type on an attributes structure. The stack passes `PSA_KEY_TYPE_AES` and no domain data.
///
/// # Safety
///
/// `attributes` must be a valid attributes pointer; the remaining arguments are not dereferenced.
#[no_mangle]
pub unsafe extern "C" fn psa_set_key_domain_parameters(
    attributes: *mut u8,
    key_type: u32,
    _data: *const u8,
    _data_length: usize,
) -> i32 {
    bump(9);
    if attributes.is_null() {
        return PSA_ERROR_INVALID_ARGUMENT;
    }
    // The type is remembered where the *slot* will read it: a small side table keyed by the attributes pointer
    // would be wrong if the caller reuses the struct, so the type is re-read at import time instead. Storing it
    // in the first four bytes of the struct is also wrong (it is an opaque type). What matters for correctness
    // is that the type must be one this implementation can honour.
    if key_type != PSA_KEY_TYPE_AES {
        return PSA_ERROR_NOT_SUPPORTED;
    }
    let _ = key_type;
    PSA_SUCCESS
}

/// Import a key into a slot and return its id in `*key`.
///
/// # Safety
///
/// `data` must point to `data_length` readable bytes and `key` to four writable bytes.
/// The last refused import, packed as `(code << 16) | length`.
///
/// A count of failures ("imp=15/14") says that one import was refused and nothing about which, and a refusal in the
/// middle of a pairing leaves a bonding record the stack discards at the next boot -- measured: the bond reaches
/// flash, survives a reset, and is deleted while the stack initialises. This records the code and the length of the
/// last refusal so the next run can name the key that was missing.
static mut IMPORT_FAILED: u32 = 0;

pub fn import_failed() -> u32 {
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!(IMPORT_FAILED)) }
}

/// Import a key, recording the code and length of the last refusal.
///
/// # Safety
///
/// `data` must point to `data_length` readable bytes and `key` to four writable bytes, and both must stay valid for
/// the duration of the call. This is the C entry point the stack's blob calls; it is not reachable from safe Rust.
#[no_mangle]
pub unsafe extern "C" fn psa_import_key(
    attributes: *const u8,
    data: *const u8,
    data_length: usize,
    key: *mut u32,
) -> i32 {
    capture_attributes(attributes);
    capture_ltk_attributes(attributes, data_length);
    let rc = psa_import_key_inner(attributes, data, data_length, key);
    if rc != 0 {
        core::ptr::write_volatile(
            core::ptr::addr_of_mut!(IMPORT_FAILED),
            (((rc as u32) & 0xFFFF) << 16) | ((data_length as u32) & 0xFFFF),
        );
    }
    rc
}

unsafe fn psa_import_key_inner(
    attributes: *const u8,
    data: *const u8,
    data_length: usize,
    key: *mut u32,
) -> i32 {
    bump(3);
    // Record what the stack is asking us to store: the words of its attributes blob, and separately the ones that
    // accompany a 16-byte payload (a long-term key). This is how the identifier seam was located -- see
    // docs/04-reference/BOND-ACROSS-RESET-2026-11-17.md.
    capture_attributes(attributes);
    capture_ltk_attributes(attributes, data_length);
    restore_slots_once();
    if data.is_null() || key.is_null() {
        return PSA_ERROR_INVALID_ARGUMENT;
    }
    if data_length == 0 || data_length > MAX_KEY {
        unsafe {
            *core::ptr::addr_of_mut!(IMPORT_REFUSED) =
                ((data_length as u32) << 8) | (PSA_ERROR_INVALID_ARGUMENT as u32 & 0xFF)
        };
        return PSA_ERROR_INVALID_ARGUMENT;
    }
    // The peer's public key arrives here before any key agreement: type 0x4112, 65 bytes, `0x04 || X || Y`.
    // Rejecting it costs the whole pairing. The blob calls `smp_verify_remote_public_key`, reads -135 as "the peer
    // sent an invalid point" and answers SMP reason 0x0B — which the spec requires for an invalid key, and which
    // is why the failure looked like a DHKey check with zero elliptic-curve calls (measured 2026-11-17, a phone as
    // the peer, ec=0,0,0 and no PSA failure recorded).
    if data_length == 65 {
        let bytes = core::slice::from_raw_parts(data, 65);
        if bytes[0] != 0x04 || p256::PublicKey::from_sec1_bytes(bytes).is_err() {
            crate::psa_ec::note_failure(3, PSA_KEY_TYPE_ECC_PUBLIC_KEY_P256);
            return PSA_ERROR_INVALID_ARGUMENT;
        }
        let table = slots();
        let Some(index) = table.iter().position(|s| !s.used) else {
            return PSA_ERROR_INSUFFICIENT_MEMORY;
        };
        let mut k = [0u8; MAX_KEY];
        k[..65].copy_from_slice(bytes);
        table[index] = Slot {
            used: true,
            id: 0,
            key_len: 65,
            key: k,
            key_type: PSA_KEY_TYPE_ECC_PUBLIC_KEY_P256,
        };
        let id = take_id(attributes);
        ptr::write(key, id);
        for slot in slots().iter_mut() {
            if slot.used && slot.id == 0 {
                slot.id = id;
                persist_slot(slot);
                break;
            }
        }
        return 0;
    }
    if !data_length.is_multiple_of(8) {
        return PSA_ERROR_INVALID_ARGUMENT;
    }
    // Only AES key sizes PSA knows: 128, 192, 256 bits.
    if !matches!(data_length, 16 | 24 | 32) {
        return PSA_ERROR_NOT_SUPPORTED;
    }
    // This AES core is 128-bit. Accepting a 24- or 32-byte key would store something the MAC below cannot use,
    // so it is refused here rather than producing a wrong tag later.
    if data_length != 16 {
        unsafe {
            *core::ptr::addr_of_mut!(IMPORT_REFUSED) =
                ((data_length as u32) << 8) | (PSA_ERROR_NOT_SUPPORTED as u32 & 0xFF)
        };
        return PSA_ERROR_NOT_SUPPORTED;
    }

    let id = take_id(attributes);

    let mut k = [0u8; MAX_KEY];
    ptr::copy_nonoverlapping(data, k.as_mut_ptr(), data_length);

    // An identifier names one key: re-importing under an identifier that is already in the table must REPLACE that
    // key, not add a second one beside it. Measured 2026-11-17: the table ended up holding the same id twice (an old
    // long-term key in front of the new one), the lookup returned the stale key, and the host cancelled the pairing
    // with AuthenticationCanceled while the device recorded a bond.
    let table = slots();
    if let Some(existing) = table.iter_mut().find(|s| s.used && s.id == id) {
        *existing = Slot {
            used: true,
            id,
            key_len: data_length,
            key: k,
            key_type: PSA_KEY_TYPE_AES,
        };
        persist_slot(existing);
        ptr::write(key, id);
        return 0;
    }

    let free = table.iter().position(|s| !s.used);
    let Some(index) = free else {
        return PSA_ERROR_INSUFFICIENT_MEMORY;
    };

    table[index] = Slot {
        used: true,
        id, // stamped now that the handle exists
        key_len: data_length,
        key: k,
        key_type: PSA_KEY_TYPE_AES,
    };

    ptr::write(key, id);
    for slot in slots().iter_mut() {
        if slot.used && slot.id == 0 {
            slot.id = id;
            persist_slot(slot);
            break;
        }
    }

    let c = ptr::addr_of_mut!(IMPORTS);
    ptr::write_volatile(c, ptr::read_volatile(c) + 1);
    PSA_SUCCESS
}

/// Destroy a key. Unknown ids are reported, not ignored: a stack that destroys a key it never imported has a
/// bookkeeping bug worth seeing.
///
/// # Safety
///
/// The id is a value, not a pointer; declaring it `u64` reads the low half whether the caller passed one register
/// or two, and only the low 32 bits are used.
#[no_mangle]
pub unsafe extern "C" fn psa_destroy_key(key: u32) -> i32 {
    DESTROYED.fetch_add(1, Ordering::Relaxed);
    let slot = (DESTROYED_NEXT.fetch_add(1, Ordering::Relaxed) % 4) as usize;
    DESTROYED_IDS[slot].store(key, Ordering::Relaxed);
    bump(6);
    restore_slots_once();
    let id = key;
    if id == 0 {
        return PSA_ERROR_INVALID_HANDLE;
    }
    let table = slots();
    for slot in table.iter_mut() {
        if slot.used && slot.id == id {
            slot.used = false;
            slot.id = 0;
            slot.key_len = 0;
            slot.key = [0; MAX_KEY];
            forget_slot(id);
            return PSA_SUCCESS;
        }
    }
    PSA_ERROR_DOES_NOT_EXIST
}

/// Begin a CMAC over the key `key`. The algorithm must be a CMAC one; anything else is refused rather than
/// silently computed as CMAC.
///
/// # Safety
///
/// `operation` must be the caller's `psa_mac_operation_t` address, and `key` is a value.
#[no_mangle]
pub unsafe extern "C" fn psa_mac_sign_setup(operation: *mut u8, key: u32, alg: u32) -> i32 {
    bump(12);
    if operation.is_null() {
        return PSA_ERROR_INVALID_ARGUMENT;
    }
    // PSA_ALG_CMAC = 0x03C00000, and the truncated variants are (PSA_ALG_CMAC | (length << 16) | 0x001F0000).
    // Only full-length CMAC is supported; a truncated request would need a different output length.
    const PSA_ALG_CMAC: u32 = 0x03C0_0000;
    if alg != PSA_ALG_CMAC {
        return PSA_ERROR_NOT_SUPPORTED;
    }

    let id = key;
    let table = slots();
    let mut seen = 0u32;
    let mut found: Option<usize> = None;
    for (i, slot) in table.iter().enumerate() {
        if slot.used {
            seen += 1;
            if seen == id {
                found = Some(i);
                break;
            }
        }
    }
    let Some(index) = found else {
        return PSA_ERROR_INVALID_HANDLE;
    };

    let owner = operation as usize;
    let ops_table = ops();
    let free = ops_table.iter().position(|o| !o.active || o.owner == owner);
    let Some(op_index) = free else {
        return PSA_ERROR_INSUFFICIENT_MEMORY;
    };
    ops_table[op_index] = MacOp {
        active: true,
        owner,
        slot: index,
        input_len: 0,
        input: [0; MAX_MAC_INPUT],
    };
    PSA_SUCCESS
}

/// Feed data to the CMAC in progress.
///
/// # Safety
///
/// `operation` must be the address passed to `psa_mac_sign_setup`; `input` must point to `input_length` bytes.
#[no_mangle]
pub unsafe extern "C" fn psa_mac_update(
    operation: *mut u8,
    input: *const u8,
    input_length: usize,
) -> i32 {
    bump(13);
    if operation.is_null() {
        return PSA_ERROR_INVALID_ARGUMENT;
    }
    if input.is_null() && input_length > 0 {
        return PSA_ERROR_INVALID_ARGUMENT;
    }
    let owner = operation as usize;
    let ops_table = ops();
    let Some(op) = ops_table.iter_mut().find(|o| o.active && o.owner == owner) else {
        return PSA_ERROR_BAD_STATE;
    };
    if op.input_len + input_length > MAX_MAC_INPUT {
        return PSA_ERROR_INSUFFICIENT_MEMORY;
    }
    ptr::copy_nonoverlapping(input, op.input[op.input_len..].as_mut_ptr(), input_length);
    op.input_len += input_length;
    PSA_SUCCESS
}

/// Finish the CMAC and write it to `mac`.
///
/// # Safety
///
/// `operation`, `mac` and `mac_length` are the caller's; `mac` must have `mac_size` writable bytes.
#[no_mangle]
pub unsafe extern "C" fn psa_mac_sign_finish(
    operation: *mut u8,
    mac: *mut u8,
    mac_size: usize,
    mac_length: *mut usize,
) -> i32 {
    bump(14);
    if operation.is_null() || mac.is_null() || mac_length.is_null() {
        return PSA_ERROR_INVALID_ARGUMENT;
    }
    let owner = operation as usize;
    let ops_table = ops();
    let Some(op_index) = ops_table.iter().position(|o| o.active && o.owner == owner) else {
        return PSA_ERROR_BAD_STATE;
    };
    if mac_size < 16 {
        return PSA_ERROR_BUFFER_TOO_SMALL;
    }

    let op = ops_table[op_index];
    let table = slots();
    let Some(slot) = table.get(op.slot) else {
        ops_table[op_index].active = false;
        return PSA_ERROR_BAD_STATE;
    };
    if !slot.used || slot.key_len != 16 {
        ops_table[op_index].active = false;
        return PSA_ERROR_INVALID_HANDLE;
    }

    let mut k = [0u8; 16];
    k.copy_from_slice(&slot.key[..16]);
    let tag = cmac(&k, &op.input[..op.input_len]);

    ptr::copy_nonoverlapping(tag.as_ptr(), mac, 16);
    ptr::write(mac_length, 16);
    ops_table[op_index].active = false;
    // The accumulated input may contain key material; clear it rather than leave it in `.bss`.
    ops_table[op_index].input = [0; MAX_MAC_INPUT];
    ops_table[op_index].input_len = 0;
    PSA_SUCCESS
}

/// Abandon a MAC operation.
///
/// # Safety
///
/// `operation` is the caller's address; nothing is dereferenced.
#[no_mangle]
pub unsafe extern "C" fn psa_mac_abort(operation: *mut u8) {
    bump(15);
    if operation.is_null() {
        return;
    }
    let owner = operation as usize;
    for op in ops().iter_mut() {
        if op.owner == owner {
            op.active = false;
            op.input = [0; MAX_MAC_INPUT];
            op.input_len = 0;
        }
    }
}

/// `PSA_ERROR_INSUFFICIENT_MEMORY`, declared here because only this file uses it.
const PSA_ERROR_INSUFFICIENT_MEMORY: i32 = -141;

/// Diagnostics: how many keys are in the table, how many imports happened, and whether anything asked for
/// randomness. The boot example prints the first and last of these, because "the stack asked for randomness and
/// this firmware cannot provide it" is something a reader should see, not infer.
pub fn stats() -> (usize, u32, u32) {
    let in_use = unsafe {
        (*ptr::addr_of!(SLOTS_TABLE))
            .iter()
            .filter(|s| s.used)
            .count()
    };
    unsafe {
        (
            in_use,
            ptr::read_volatile(ptr::addr_of!(IMPORTS)),
            ptr::read_volatile(ptr::addr_of!(RANDOM_REQUESTS)),
        )
    }
}

// ---------------------------------------------------------------------------------------------------------------
// The functions the security manager needs, beyond the MAC streaming API above.
//
// With `SL_CATALOG_BLUETOOTH_FEATURE_SM_PRESENT` enabled the linker named exactly seven of them:
// `psa_generate_key`, `psa_export_public_key`, `psa_raw_key_agreement`, `psa_mac_compute`, `psa_cipher_encrypt`,
// `psa_export_key`, `psa_get_key_attributes`. Three take plain buffers and reuse the AES the project already has and
// tests; four need P-256 and the attribute structure, and are the next increment.
//
// Anything this does not support is **recorded**, not silently answered: a security exchange that fails because an
// algorithm returned a generic error is the hardest kind of bug to find on a board with no debugger.
// ---------------------------------------------------------------------------------------------------------------

/// `PSA_ALG_CMAC`
const PSA_ALG_CMAC: u32 = 0x03c0_0200;
/// `PSA_ALG_CTR`
const PSA_ALG_CTR: u32 = 0x04c0_1000;
/// `PSA_ALG_ECB_NO_PADDING`. The bonding database wraps the long-term key with this, through
/// `bondingdb_psa_add_ecb_key`, so a security stack that cannot do it cannot store a bond.
const PSA_ALG_ECB_NO_PADDING: u32 = 0x0440_4400;
/// The last algorithm this module was asked for and could not serve, and how many times. Read by `stats()`'s caller
/// through [`unsupported_algorithm`]; a firmware that pairs badly can be asked what it wanted.
static mut UNSUPPORTED_ALG: u32 = 0;
static mut UNSUPPORTED_COUNT: u32 = 0;

/// The last unsupported algorithm and the number of times one was requested.
pub fn unsupported_algorithm() -> (u32, u32) {
    unsafe { (UNSUPPORTED_ALG, UNSUPPORTED_COUNT) }
}

fn record_unsupported(alg: u32) -> i32 {
    unsafe {
        UNSUPPORTED_ALG = alg;
        UNSUPPORTED_COUNT += 1;
    }
    PSA_ERROR_NOT_SUPPORTED
}

/// The slot a handle refers to. Handles are the indices this module hands out, offset by one, so that zero is never
/// a valid key -- `NEXT_ID` starts at 1 for exactly that reason.
fn slot_for(key: u32) -> Option<&'static Slot> {
    let id = key;
    if id == 0 {
        return None;
    }
    if let Some(slot) = slots().iter().find(|s| s.used && s.id == id) {
        return Some(slot);
    }
    // A miss is the moment the keys are actually needed, and the only moment we know the stack has finished opening
    // its NVM3 instance. Try one more restore here before reporting the key as absent: without this the table stays
    // empty for the whole boot if the start-up restore ran too early (measured 2026-11-17: the link dropped with
    // 0x1006 on a key that was in the flash).
    SLOT_MISSES.fetch_add(1, Ordering::Relaxed);
    LAST_MISS_ID.store(id, Ordering::Relaxed);
    allow_another_restore();
    restore_slots_once();
    slots().iter().find(|s| s.used && s.id == id)
}

/// `psa_export_key`: copy a stored key out. Used by the security manager to fetch the long-term key it just produced.
///
/// # Safety
///
/// `data` must point at `data_size` writable bytes, and `data_length` at a writable `size_t`, as the PSA contract
/// requires of its callers.
#[no_mangle]
pub unsafe extern "C" fn psa_export_key(
    key: u32,
    data: *mut u8,
    data_size: usize,
    data_length: *mut usize,
) -> i32 {
    bump(4);
    let Some(slot) = slot_for(key) else {
        return PSA_ERROR_INVALID_HANDLE;
    };
    if slot.key_len > data_size {
        if !data_length.is_null() {
            *data_length = slot.key_len;
        }
        return PSA_ERROR_BUFFER_TOO_SMALL;
    }
    if data.is_null() {
        return PSA_ERROR_INVALID_ARGUMENT;
    }
    core::ptr::copy_nonoverlapping(slot.key.as_ptr(), data, slot.key_len);
    if !data_length.is_null() {
        *data_length = slot.key_len;
    }
    0
}

/// `psa_mac_compute`: AES-CMAC over `input`, into `mac`.
///
/// PSA's one-shot form is what the security manager uses for the confirm and key-derivation functions of pairing
/// (f4, f5, f6, g2), and all of them are CMAC over a handful of bytes.
///
/// # Safety
///
/// `input` must point at `input_length` readable bytes and `mac` at `mac_size` writable ones; `mac_length` at a
/// writable `size_t`.
#[no_mangle]
pub unsafe extern "C" fn psa_mac_compute(
    key: u32,
    alg: u32,
    input: *const u8,
    input_length: usize,
    mac: *mut u8,
    mac_size: usize,
    mac_length: *mut usize,
) -> i32 {
    bump(11);
    if alg != PSA_ALG_CMAC {
        return record_unsupported(alg);
    }
    let Some(slot) = slot_for(key) else {
        return PSA_ERROR_INVALID_HANDLE;
    };
    if slot.key_len != 16 {
        return PSA_ERROR_INVALID_ARGUMENT;
    }
    if input.is_null() || mac.is_null() || mac_size < 16 {
        return PSA_ERROR_INVALID_ARGUMENT;
    }
    let mut block = [0u8; 16];
    block.copy_from_slice(&slot.key[..16]);
    let mut msg = [0u8; MAX_MAC_INPUT];
    let n = input_length.min(MAX_MAC_INPUT);
    core::ptr::copy_nonoverlapping(input, msg.as_mut_ptr(), n);
    let tag = aes_modes::cmac(&block, &msg[..n]);
    core::ptr::copy_nonoverlapping(tag.as_ptr(), mac, 16);
    if !mac_length.is_null() {
        *mac_length = 16;
    }
    0
}

/// `psa_cipher_encrypt`: AES-CTR, with PSA's convention that the output begins with the counter block.
///
/// PSA defines this one-shot form as "input = IV || plaintext, output = IV || ciphertext", and the security manager
/// relies on that layout when it encrypts the confirm value of legacy pairing.
///
/// # Safety
///
/// `input` must point at `input_length` readable bytes and `output` at `output_size` writable ones; `output_length`
/// at a writable `size_t`.
#[no_mangle]
pub unsafe extern "C" fn psa_cipher_encrypt(
    key: u32,
    alg: u32,
    input: *const u8,
    input_length: usize,
    output: *mut u8,
    output_size: usize,
    output_length: *mut usize,
) -> i32 {
    bump(16);
    if alg == PSA_ALG_ECB_NO_PADDING {
        let Some(slot) = slot_for(key) else {
            return PSA_ERROR_INVALID_HANDLE;
        };
        if slot.key_len != 16 {
            return PSA_ERROR_INVALID_ARGUMENT;
        }
        // No IV in PSA's ECB one-shot, and the length must be a whole number of blocks.
        if input.is_null() || output.is_null() || !input_length.is_multiple_of(16) {
            return PSA_ERROR_INVALID_ARGUMENT;
        }
        if output_size < input_length {
            if !output_length.is_null() {
                *output_length = input_length;
            }
            return PSA_ERROR_BUFFER_TOO_SMALL;
        }
        let mut block = [0u8; 16];
        block.copy_from_slice(&slot.key[..16]);
        let mut offset = 0usize;
        while offset < input_length {
            let mut plain = [0u8; 16];
            core::ptr::copy_nonoverlapping(input.add(offset), plain.as_mut_ptr(), 16);
            let cipher = aes_ctr_radio::encrypt_block(&block, &plain);
            core::ptr::copy_nonoverlapping(cipher.as_ptr(), output.add(offset), 16);
            offset += 16;
        }
        if !output_length.is_null() {
            *output_length = input_length;
        }
        return 0;
    }
    if alg != PSA_ALG_CTR {
        return record_unsupported(alg);
    }
    let Some(slot) = slot_for(key) else {
        return PSA_ERROR_INVALID_HANDLE;
    };
    if slot.key_len != 16 {
        return PSA_ERROR_INVALID_ARGUMENT;
    }
    if input.is_null() || output.is_null() || input_length < 16 {
        return PSA_ERROR_INVALID_ARGUMENT;
    }
    let payload = input_length - 16;
    if output_size < input_length {
        if !output_length.is_null() {
            *output_length = input_length;
        }
        return PSA_ERROR_BUFFER_TOO_SMALL;
    }

    let mut block = [0u8; 16];
    block.copy_from_slice(&slot.key[..16]);

    // The IV leads both the input and the output.
    core::ptr::copy_nonoverlapping(input, output, 16);
    if payload == 0 {
        if !output_length.is_null() {
            *output_length = 16;
        }
        return 0;
    }

    let mut iv = [0u8; 16];
    core::ptr::copy_nonoverlapping(input, iv.as_mut_ptr(), 16);
    let mut counter = u128::from_be_bytes(iv);
    let mut offset = 0usize;
    while offset < payload {
        let ks = aes_ctr_radio::encrypt_block(&block, &counter.to_be_bytes());
        let take = core::cmp::min(16, payload - offset);
        for (j, k) in ks.iter().take(take).enumerate() {
            let byte = core::ptr::read(input.add(16 + offset + j));
            core::ptr::write(output.add(16 + offset + j), byte ^ k);
        }
        offset += take;
        counter = counter.wrapping_add(1);
    }

    if !output_length.is_null() {
        *output_length = input_length;
    }
    0
}

// ---------------------------------------------------------------------------------------------------------------
// Storage, and the four entry points the security manager's elliptic-curve path needs. The maths lives in
// `psa_ec.rs`; these are the ABI shapes the vendor blob expects.
// ---------------------------------------------------------------------------------------------------------------

/// The slot behind a handle, for the elliptic-curve module.
pub(crate) fn slot(key: u32) -> Option<&'static Slot> {
    slot_for(key)
}

/// Store `material` as a key of `key_type` and return its handle, or `None` when the table is full.
pub(crate) fn store_key(material: &[u8], key_type: u32) -> Option<u32> {
    if material.is_empty() || material.len() > MAX_KEY {
        return None;
    }
    let table = slots();
    let index = table.iter().position(|s| !s.used)?;
    let id = unsafe {
        let n = ptr::addr_of_mut!(NEXT_ID);
        let v = ptr::read_volatile(n);
        ptr::write_volatile(n, v.wrapping_add(1).max(1));
        v.max(1)
    };
    table[index] = Slot {
        used: true,
        id,
        key_len: material.len(),
        key: [0; MAX_KEY],
        key_type,
    };
    table[index].key[..material.len()].copy_from_slice(material);
    persist_slot(&table[index]);
    Some(id)
}

/// `psa_generate_key`
///
/// # Safety
///
/// `attributes` must point at a `psa_key_attributes_t`; `key` at a `mbedtls_svc_key_id_t`.
#[no_mangle]
pub unsafe extern "C" fn psa_generate_key(attributes: *const u8, key: *mut u32) -> i32 {
    bump(2);
    if attributes.is_null() {
        return PSA_ERROR_INVALID_ARGUMENT;
    }
    // The type is the first field of the attribute structure, whatever the rest of it looks like.
    let key_type = ptr::read_unaligned(attributes as *const u16);
    if key_type == crate::psa_ec::PSA_KEY_TYPE_ECC_KEY_PAIR_P256 {
        return crate::psa_ec::generate_ecc_key(
            attributes as *const crate::psa_ec::KeyAttributes,
            key,
        );
    }
    // An AES key: the size is in bits, second field.
    //
    // The type is checked, not assumed. Treating "not ECC" as "AES" is what let a 0x7112 key-pair request become a
    // 32-byte AES key without a word of complaint, because 256 bits is a legal AES size.
    if key_type != crate::psa_ec::PSA_KEY_TYPE_AES {
        record_unsupported(key_type.into());
        return PSA_ERROR_NOT_SUPPORTED;
    }
    let bits = ptr::read_unaligned(attributes.add(2) as *const u16);
    let len = (bits / 8) as usize;
    if len != 16 && len != 32 {
        record_unsupported(key_type.into());
        return PSA_ERROR_NOT_SUPPORTED;
    }
    let mut material = [0u8; 32];
    let status = psa_generate_random(material.as_mut_ptr(), len);
    if status != 0 {
        return status;
    }
    match store_key(&material[..len], PSA_KEY_TYPE_AES) {
        Some(id) => {
            if !key.is_null() {
                *key = id;
            }
            0
        }
        None => -143, // PSA_ERROR_INSUFFICIENT_MEMORY
    }
}

/// `psa_get_key_attributes`
///
/// # Safety
///
/// `attributes` must point at a `psa_key_attributes_t`.
#[no_mangle]
pub unsafe extern "C" fn psa_get_key_attributes(key: u32, attributes: *mut u8) -> i32 {
    bump(7);
    crate::psa_ec::key_attributes(key, attributes as *mut crate::psa_ec::KeyAttributes)
}

/// `psa_export_public_key`
///
/// # Safety
///
/// `data` must point at `data_size` writable bytes.
#[no_mangle]
pub unsafe extern "C" fn psa_export_public_key(
    key: u32,
    data: *mut u8,
    data_size: usize,
    data_length: *mut usize,
) -> i32 {
    bump(5);
    crate::psa_ec::export_public_key(key, data, data_size, data_length)
}

/// `psa_raw_key_agreement`
///
/// # Safety
///
/// `peer_key` must point at `peer_key_length` readable bytes; `output` at `output_size` writable ones.
#[no_mangle]
pub unsafe extern "C" fn psa_raw_key_agreement(
    alg: u32,
    private_key: u32,
    peer_key: *const u8,
    peer_key_length: usize,
    output: *mut u8,
    output_size: usize,
    output_length: *mut usize,
) -> i32 {
    bump(10);
    crate::psa_ec::raw_key_agreement(
        alg,
        private_key,
        peer_key,
        peer_key_length,
        output,
        output_size,
        output_length,
    )
}

// ---------------------------------------------------------------------------------------------------------------
// Which PSA entry points the stack actually calls, in what order — instrumentation, not logic.
//
// The pairing failure of 2026-11-14 said "no PSA entry point was called" from three counters. Three is not the
// whole interface: the blobs import seventeen PSA symbols (`nm -u` on the three archives), and the security
// manager may stop at any one of them. These counters are here so the next pairing attempt answers "where did it
// stop" with a number per symbol instead of a guess.
pub const PSA_CALL_NAMES: [&str; 17] = [
    "psa_crypto_init",
    "psa_generate_random",
    "psa_generate_key",
    "psa_import_key",
    "psa_export_key",
    "psa_export_public_key",
    "psa_destroy_key",
    "psa_get_key_attributes",
    "psa_reset_key_attributes",
    "psa_set_key_domain_parameters",
    "psa_raw_key_agreement",
    "psa_mac_compute",
    "psa_mac_sign_setup",
    "psa_mac_update",
    "psa_mac_sign_finish",
    "psa_mac_abort",
    "psa_cipher_encrypt",
];
static mut PSA_CALLS: [u32; 17] = [0; 17];
/// Index of the most recent PSA entry point, kept as a breadcrumb.
///
/// A fault inside the stack's pairing path kills the firmware, so a counter can only report where the code had been.
/// This one is written by `bump`, which every entry point calls first, and it survives the fault because it is a
/// static: the last live record before the crash names the function the code was in.
static mut LAST_CALL: u32 = 0;
/// Why the last `psa_import_key` was refused: `(length << 8) | (code & 0xFF)`. A refusal that leaves no trace is how
/// "imp=14/13" stayed unexplained: one import in fourteen failed and nothing said which one or why.
static mut IMPORT_REFUSED: u32 = 0;

#[inline(always)]
fn bump(i: usize) {
    // SAFETY: single-core, and a counter that races with itself only misreports a count.
    unsafe {
        PSA_CALLS[i] = PSA_CALLS[i].wrapping_add(1);
        *core::ptr::addr_of_mut!(LAST_CALL) = i as u32;
    }
}

/// The per-symbol call counts, in `PSA_CALL_NAMES` order.
pub fn psa_calls() -> [u32; 17] {
    unsafe { PSA_CALLS }
}

/// AES-128-CMAC against a published vector (NIST SP 800-38B, 16-byte message).
///
/// The DHKey check of LE Secure Connections is an AES-CMAC over the ECDH secret (f5/f6), so a wrong MAC produces
/// exactly the `SL_STATUS_BT_SMP_DHKEY_CHECK_FAILED` this board reports. This separates "the MAC is wrong" from
/// "the MAC is right and its inputs are wrong" with no phone in the loop.
pub fn cmac_kat() -> bool {
    const KEY: [u8; 16] = [
        0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f,
        0x3c,
    ];
    const MSG: [u8; 16] = [
        0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93, 0x17,
        0x2a,
    ];
    const EXPECTED: [u8; 16] = [
        0x07, 0x0a, 0x16, 0xb4, 0x6b, 0x4d, 0x41, 0x44, 0xf7, 0x9b, 0xdd, 0x9d, 0xd0, 0x4a, 0x28,
        0x7c,
    ];
    let got = unsafe { crate::crypto::cmac_of(KEY.as_ptr(), MSG.as_ptr(), MSG.len()) };
    got == EXPECTED
}

/// How many times the stack asked to import a key, and how many of those succeeded.
///
/// The pairing dies on an import that returns -135; these two numbers say whether that path is even reached, which
/// is the difference between "our import is wrong" and "the exchange never gets there".
pub fn import_stats() -> (u32, u32) {
    unsafe {
        let calls = *core::ptr::addr_of!(PSA_CALLS[3]);
        let ok = *core::ptr::addr_of!(IMPORTS);
        (calls, ok)
    }
}

/// Whether the stack asked for a key to be generated, and whether we refused a key type instead.
///
/// `ec=0,0,0` says the elliptic-curve code never ran; these two numbers say why: either nothing asked for a key, or
/// something asked for a type this file declines and the caller did not notice.
pub fn generate_stats() -> (u32, u32) {
    unsafe {
        let calls = *core::ptr::addr_of!(PSA_CALLS[2]);
        let unsupported = *core::ptr::addr_of!(UNSUPPORTED_ALG);
        (calls, unsupported)
    }
}

/// The index of the last PSA entry point entered, for a fault that leaves no room for a crash handler.
pub fn last_call() -> u32 {
    unsafe { *core::ptr::addr_of!(LAST_CALL) }
}

/// `(length << 8) | code` of the last refused `psa_import_key`, zero if none since boot.
pub fn import_refused() -> u32 {
    unsafe { *core::ptr::addr_of!(IMPORT_REFUSED) }
}
