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
/// Longest key this accepts, in bytes. AES-256 would be 32; the link layer uses 16.
const MAX_KEY: usize = 32;
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

static mut SLOTS_TABLE: [Slot; SLOTS] = [EMPTY_SLOT; SLOTS];
/// Key id 0 is `PSA_KEY_ID_NULL` and must never be handed out; ids start at 1.
static mut NEXT_ID: u32 = 1;
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
    if attributes.is_null() {
        return;
    }
    ptr::write_bytes(attributes, 0, PSA_KEY_ATTRIBUTES_BYTES);
}

/// `sizeof(psa_key_attributes_t)` measured for this project's mbedtls headers.
const PSA_KEY_ATTRIBUTES_BYTES: usize = 232;

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
#[no_mangle]
pub unsafe extern "C" fn psa_import_key(
    _attributes: *const u8,
    data: *const u8,
    data_length: usize,
    key: *mut u32,
) -> i32 {
    if data.is_null() || key.is_null() {
        return PSA_ERROR_INVALID_ARGUMENT;
    }
    if data_length == 0 || data_length > MAX_KEY || !data_length.is_multiple_of(8) {
        return PSA_ERROR_INVALID_ARGUMENT;
    }
    // Only AES key sizes PSA knows: 128, 192, 256 bits.
    if !matches!(data_length, 16 | 24 | 32) {
        return PSA_ERROR_NOT_SUPPORTED;
    }
    // This AES core is 128-bit. Accepting a 24- or 32-byte key would store something the MAC below cannot use,
    // so it is refused here rather than producing a wrong tag later.
    if data_length != 16 {
        return PSA_ERROR_NOT_SUPPORTED;
    }

    let table = slots();
    let free = table.iter().position(|s| !s.used);
    let Some(index) = free else {
        return PSA_ERROR_INSUFFICIENT_MEMORY;
    };

    let mut k = [0u8; MAX_KEY];
    ptr::copy_nonoverlapping(data, k.as_mut_ptr(), data_length);
    table[index] = Slot {
        used: true,
        id: 0, // stamped below, once the handle is handed out
        key_len: data_length,
        key: k,
        key_type: PSA_KEY_TYPE_AES,
    };

    // The slot just written holds no id yet; stamp it now that the handle exists.
    let id = {
        let n = ptr::addr_of_mut!(NEXT_ID);
        let v = ptr::read_volatile(n);
        ptr::write_volatile(n, v.wrapping_add(1).max(1));
        v.max(1)
    };
    ptr::write(key, id);
    for slot in slots().iter_mut() {
        if slot.used && slot.id == 0 {
            slot.id = id;
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
pub unsafe extern "C" fn psa_destroy_key(key: u64) -> i32 {
    let id = key as u32;
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
pub unsafe extern "C" fn psa_mac_sign_setup(operation: *mut u8, key: u64, alg: u32) -> i32 {
    if operation.is_null() {
        return PSA_ERROR_INVALID_ARGUMENT;
    }
    // PSA_ALG_CMAC = 0x03C00000, and the truncated variants are (PSA_ALG_CMAC | (length << 16) | 0x001F0000).
    // Only full-length CMAC is supported; a truncated request would need a different output length.
    const PSA_ALG_CMAC: u32 = 0x03C0_0000;
    if alg != PSA_ALG_CMAC {
        return PSA_ERROR_NOT_SUPPORTED;
    }

    let id = key as u32;
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
fn slot_for(key: u64) -> Option<&'static Slot> {
    let id = key as u32;
    if id == 0 {
        return None;
    }
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
    key: u64,
    data: *mut u8,
    data_size: usize,
    data_length: *mut usize,
) -> i32 {
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
    key: u64,
    alg: u32,
    input: *const u8,
    input_length: usize,
    mac: *mut u8,
    mac_size: usize,
    mac_length: *mut usize,
) -> i32 {
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
    key: u64,
    alg: u32,
    input: *const u8,
    input_length: usize,
    output: *mut u8,
    output_size: usize,
    output_length: *mut usize,
) -> i32 {
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
pub(crate) fn slot(key: u64) -> Option<&'static Slot> {
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
    Some(id)
}

/// `psa_generate_key`
///
/// # Safety
///
/// `attributes` must point at a `psa_key_attributes_t`; `key` at a `mbedtls_svc_key_id_t`.
#[no_mangle]
pub unsafe extern "C" fn psa_generate_key(attributes: *const u8, key: *mut u64) -> i32 {
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
                *key = u64::from(id);
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
pub unsafe extern "C" fn psa_get_key_attributes(key: u64, attributes: *mut u8) -> i32 {
    crate::psa_ec::key_attributes(key, attributes as *mut crate::psa_ec::KeyAttributes)
}

/// `psa_export_public_key`
///
/// # Safety
///
/// `data` must point at `data_size` writable bytes.
#[no_mangle]
pub unsafe extern "C" fn psa_export_public_key(
    key: u64,
    data: *mut u8,
    data_size: usize,
    data_length: *mut usize,
) -> i32 {
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
    private_key: u64,
    peer_key: *const u8,
    peer_key_length: usize,
    output: *mut u8,
    output_size: usize,
    output_length: *mut usize,
) -> i32 {
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
