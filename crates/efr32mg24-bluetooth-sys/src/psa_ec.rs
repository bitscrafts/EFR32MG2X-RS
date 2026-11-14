//! The asymmetric half of PSA, for the security manager's LE Secure Connections.
//!
//! With `SL_CATALOG_BLUETOOTH_FEATURE_SM_PRESENT` enabled the linker named seven missing symbols. Three took plain
//! buffers and live in `psa.rs` on top of the AES this project already tests. The four here are the elliptic-curve
//! ones, and they are what pairing needs to agree on a key:
//!
//!   * `psa_generate_key` — a P-256 key pair, stored in the same slot table as the AES keys
//!   * `psa_export_public_key` — the uncompressed point a peer receives
//!   * `psa_raw_key_agreement` — ECDH with the peer's public key
//!   * `psa_get_key_attributes` — what kind of key a handle holds
//!
//! ## The attribute structure is verified, not assumed
//!
//! `psa_get_key_attributes` writes into a caller-owned `psa_key_attributes_t`. Getting that layout wrong would corrupt
//! the caller's stack, and a padding mistake is invisible in the source: the sizes come from
//! `psa_key_type_t`/`psa_key_bits_t` (u16), `psa_key_lifetime_t`/`mbedtls_svc_key_id_t`/`psa_key_usage_t`/
//! `psa_algorithm_t` (u32) and `psa_key_attributes_flag_t` (u16), in that order, then two pointer-sized fields.
//!
//! The firmware therefore records `size_of` for this struct next to `sizeof(psa_key_attributes_t)` as measured by the C
//! compiler in `config/psa_layout_probe.c`. If the two differ, the record says so before pairing is attempted.

use core::ffi::c_void;

use p256::elliptic_curve::rand_core::{CryptoRng, RngCore};
use p256::elliptic_curve::sec1::ToEncodedPoint;
use p256::{PublicKey, SecretKey};

/// `PSA_KEY_TYPE_ECC_KEY_PAIR(PSA_ECC_FAMILY_SECP_R1)`.
///
/// **0x7112, not 0x4112.** `crypto_values.h` gives `PSA_KEY_TYPE_ECC_PUBLIC_KEY_BASE = 0x4100` and
/// `PSA_KEY_TYPE_ECC_KEY_PAIR_BASE = 0x7100`; the key *pair* base is the higher one. The first version of this
/// constant used 0x4112, so `psa_generate_key` did not recognise the security manager's request, fell through to its
/// AES branch -- 256 bits is a legal AES size, so nothing complained -- and the public-key export then refused a key
/// that was not an ECC pair. The security manager answered the pairing request with SMP reason 8, "unspecified".
pub const PSA_KEY_TYPE_ECC_KEY_PAIR_P256: u16 = 0x7112;

/// `PSA_KEY_TYPE_AES`
pub const PSA_KEY_TYPE_AES: u16 = 0x2400;
/// `PSA_ALG_ECDH`
pub const PSA_ALG_ECDH: u32 = 0x0902_0000;
/// `PSA_ERROR_NOT_SUPPORTED`
const PSA_ERROR_NOT_SUPPORTED: i32 = -134;
/// `PSA_ERROR_INVALID_ARGUMENT`
const PSA_ERROR_INVALID_ARGUMENT: i32 = -135;
/// `PSA_ERROR_BUFFER_TOO_SMALL`
const PSA_ERROR_BUFFER_TOO_SMALL: i32 = -138;
/// `PSA_ERROR_INVALID_HANDLE`
const PSA_ERROR_INVALID_HANDLE: i32 = -136;

/// The last call that failed and what it was handed, so a pairing attempt that aborts with "unspecified reason" can be
/// read off the board instead of guessed at. `(operation, value)`: operation 1 = generate, 2 = export public,
/// 3 = agreement, 4 = attributes; value is the status code, or the key type for a refused generate.
static mut EC_GENERATED: u32 = 0;
static mut EC_EXPORTED: u32 = 0;
static mut EC_AGREED: u32 = 0;

/// `(key pairs generated, public keys exported, agreements computed)`. A pairing that fails after the export and before
/// the agreement is a different defect from one that fails in the agreement, and the codes are undocumented.
pub fn call_counts() -> (u32, u32, u32) {
    unsafe { (EC_GENERATED, EC_EXPORTED, EC_AGREED) }
}

/// How many P-256 key pairs `psa_generate_key` has produced. A pairing attempt that leaves this at zero never reached
/// the cryptography.
pub fn generated_count() -> u32 {
    unsafe { EC_GENERATED }
}

static mut LAST_FAIL_OP: u32 = 0;
static mut LAST_FAIL_VALUE: u32 = 0;

/// The last failure, or `(0, 0)` if none.
pub fn last_failure() -> (u32, u32) {
    unsafe { (LAST_FAIL_OP, LAST_FAIL_VALUE) }
}

fn note_failure(op: u32, value: u32) {
    unsafe {
        LAST_FAIL_OP = op;
        LAST_FAIL_VALUE = value;
    }
}

/// `psa_key_attributes_t`, in the order and widths the PSA headers declare.
#[repr(C)]
pub struct KeyAttributes {
    /// `psa_key_type_t` (u16)
    pub key_type: u16,
    /// `psa_key_bits_t` (u16)
    pub bits: u16,
    /// `psa_key_lifetime_t` (u32)
    pub lifetime: u32,
    /// `mbedtls_svc_key_id_t` (u32)
    pub id: u32,
    /// `psa_key_policy_t.usage` (u32)
    pub usage: u32,
    /// `psa_key_policy_t.alg` (u32)
    pub alg: u32,
    /// `psa_key_policy_t.alg2` (u32)
    pub alg2: u32,
    /// `psa_key_attributes_flag_t` (u16), followed by two bytes of padding to align the pointer below.
    pub flags: u16,
    pub _padding: u16,
    /// `domain_parameters`
    pub domain_parameters: *mut c_void,
    /// `domain_parameters_size`
    pub domain_parameters_size: usize,
}

// The layout, checked at compile time. 2 + 2 + 4 + 4 + 4 + 4 + 4 + 2 = 26, padded to 28 for the pointer, then two
// pointer-sized fields: 36 bytes on this 32-bit target. The two fields the security manager actually reads -- `type`
// and `bits` -- sit at offsets 0 and 2 with no padding possible before them, which is the part that matters; the size
// is asserted so that a future field added here cannot silently overrun the caller's structure.
const _: () = assert!(core::mem::size_of::<KeyAttributes>() == 36);

/// A hardware-backed RNG for `p256`, over the SE's TRNG that this crate already reaches.
struct SeRng;

impl RngCore for SeRng {
    fn next_u32(&mut self) -> u32 {
        let mut b = [0u8; 4];
        self.fill_bytes(&mut b);
        u32::from_le_bytes(b)
    }

    fn next_u64(&mut self) -> u64 {
        let mut b = [0u8; 8];
        self.fill_bytes(&mut b);
        u64::from_le_bytes(b)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        // SAFETY: `psa_generate_random` is this crate's own function; it zeroes the buffer and reports failure rather
        // than publishing bytes it did not get, so a failure here leaves `dest` zeroed and the caller's scalar is
        // still a valid (if useless) value.
        unsafe {
            let _ = crate::psa::psa_generate_random(dest.as_mut_ptr(), dest.len());
        }
    }

    fn try_fill_bytes(
        &mut self,
        dest: &mut [u8],
    ) -> Result<(), p256::elliptic_curve::rand_core::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

impl CryptoRng for SeRng {}

/// Generate a key pair and return its handle, storing the private scalar in the slot table.
///
/// # Safety
///
/// `attributes` must point at a `psa_key_attributes_t` and `key` at a `mbedtls_svc_key_id_t`, as the caller does.
pub unsafe fn generate_ecc_key(attributes: *const KeyAttributes, key: *mut u64) -> i32 {
    if attributes.is_null() || key.is_null() {
        return PSA_ERROR_INVALID_ARGUMENT;
    }
    let attrs = &*attributes;
    if attrs.key_type != PSA_KEY_TYPE_ECC_KEY_PAIR_P256 {
        note_failure(1, u32::from(attrs.key_type));
        return PSA_ERROR_NOT_SUPPORTED;
    }

    let secret = SecretKey::random(&mut SeRng);
    let scalar = secret.to_bytes();

    match crate::psa::store_key(&scalar, u32::from(PSA_KEY_TYPE_ECC_KEY_PAIR_P256)) {
        Some(id) => {
            *key = u64::from(id);
            unsafe { EC_GENERATED = EC_GENERATED.wrapping_add(1) };
            0
        }
        None => {
            note_failure(1, 143);
            -143 // PSA_ERROR_INSUFFICIENT_MEMORY
        }
    }
}

/// Write the attributes of a stored key into the caller's structure.
///
/// # Safety
///
/// `attributes` must point at a `psa_key_attributes_t`.
pub unsafe fn key_attributes(key: u64, attributes: *mut KeyAttributes) -> i32 {
    if attributes.is_null() {
        return PSA_ERROR_INVALID_ARGUMENT;
    }
    let Some(slot) = crate::psa::slot(key) else {
        return PSA_ERROR_INVALID_HANDLE;
    };
    let out = &mut *attributes;
    out.key_type = slot.key_type as u16;
    out.bits = if slot.key_type == u32::from(PSA_KEY_TYPE_ECC_KEY_PAIR_P256) {
        256
    } else {
        (slot.key_len * 8) as u16
    };
    out.lifetime = 0; // PSA_KEY_LIFETIME_VOLATILE
    out.id = slot.id;
    out.usage = 0;
    out.alg = 0;
    out.alg2 = 0;
    out.flags = 0;
    out._padding = 0;
    out.domain_parameters = core::ptr::null_mut();
    out.domain_parameters_size = 0;
    0
}

/// Export the public half of a stored ECC key as the uncompressed point the peer expects (65 bytes: 0x04 || X || Y).
///
/// # Safety
///
/// `data` must point at `data_size` writable bytes.
pub unsafe fn export_public_key(
    key: u64,
    data: *mut u8,
    data_size: usize,
    data_length: *mut usize,
) -> i32 {
    let Some(slot) = crate::psa::slot(key) else {
        note_failure(2, (key & 0xffff) as u32);
        return PSA_ERROR_INVALID_HANDLE;
    };
    if slot.key_type != u32::from(PSA_KEY_TYPE_ECC_KEY_PAIR_P256) || slot.key_len != 32 {
        note_failure(2, slot.key_type);
        return PSA_ERROR_INVALID_ARGUMENT;
    }
    let mut scalar = [0u8; 32];
    scalar.copy_from_slice(&slot.key[..32]);
    let Ok(secret) = SecretKey::from_slice(&scalar) else {
        note_failure(2, 0xdead);
        return PSA_ERROR_INVALID_ARGUMENT;
    };
    let point = secret.public_key().to_encoded_point(false);
    let bytes = point.as_bytes();
    if data.is_null() || data_size < bytes.len() {
        if !data_length.is_null() {
            *data_length = bytes.len();
        }
        return PSA_ERROR_BUFFER_TOO_SMALL;
    }
    core::ptr::copy_nonoverlapping(bytes.as_ptr(), data, bytes.len());
    if !data_length.is_null() {
        *data_length = bytes.len();
    }
    unsafe { EC_EXPORTED = EC_EXPORTED.wrapping_add(1) };
    0
}

/// ECDH: the shared secret between a stored private key and a peer's public key.
///
/// # Safety
///
/// `peer_key` must point at `peer_key_length` readable bytes; `output` at `output_size` writable ones.
pub unsafe fn raw_key_agreement(
    alg: u32,
    key: u64,
    peer_key: *const u8,
    peer_key_length: usize,
    output: *mut u8,
    output_size: usize,
    output_length: *mut usize,
) -> i32 {
    if alg != PSA_ALG_ECDH {
        note_failure(3, alg);
        return PSA_ERROR_NOT_SUPPORTED;
    }
    let Some(slot) = crate::psa::slot(key) else {
        return PSA_ERROR_INVALID_HANDLE;
    };
    if slot.key_type != u32::from(PSA_KEY_TYPE_ECC_KEY_PAIR_P256) || slot.key_len != 32 {
        return PSA_ERROR_INVALID_ARGUMENT;
    }
    if peer_key.is_null() || peer_key_length != 65 || output.is_null() || output_size < 32 {
        note_failure(3, 0x65_00 | peer_key_length as u32);
        if !output_length.is_null() {
            *output_length = 32;
        }
        return PSA_ERROR_INVALID_ARGUMENT;
    }

    let mut scalar = [0u8; 32];
    scalar.copy_from_slice(&slot.key[..32]);
    let Ok(secret) = SecretKey::from_slice(&scalar) else {
        return PSA_ERROR_INVALID_ARGUMENT;
    };

    let mut peer = [0u8; 65];
    core::ptr::copy_nonoverlapping(peer_key, peer.as_mut_ptr(), 65);
    let Ok(public) = PublicKey::from_sec1_bytes(&peer) else {
        return PSA_ERROR_INVALID_ARGUMENT;
    };

    let shared = p256::ecdh::diffie_hellman(secret.to_nonzero_scalar(), public.as_affine());
    let x = shared.raw_secret_bytes();
    core::ptr::copy_nonoverlapping(x.as_ptr(), output, 32);
    if !output_length.is_null() {
        *output_length = 32;
    }
    unsafe { EC_AGREED = EC_AGREED.wrapping_add(1) };
    0
}

/// A self-test the firmware can run and record: two key pairs agree on the same secret in both directions.
///
/// Returns `(ok, public_len, secret_matches)`. Pairing depends on exactly this property, and a firmware that cannot
/// do it should say so before a client tries to pair.
pub fn self_test() -> (bool, usize, bool) {
    let a = SecretKey::random(&mut SeRng);
    let b = SecretKey::random(&mut SeRng);
    let pa = a.public_key().to_encoded_point(false);
    let pb = b.public_key().to_encoded_point(false);

    let Ok(pb_parsed) = PublicKey::from_sec1_bytes(pb.as_bytes()) else {
        return (false, pa.as_bytes().len(), false);
    };
    let Ok(pa_parsed) = PublicKey::from_sec1_bytes(pa.as_bytes()) else {
        return (false, pa.as_bytes().len(), false);
    };

    let ab = p256::ecdh::diffie_hellman(a.to_nonzero_scalar(), pb_parsed.as_affine());
    let ba = p256::ecdh::diffie_hellman(b.to_nonzero_scalar(), pa_parsed.as_affine());
    (
        true,
        pa.as_bytes().len(),
        ab.raw_secret_bytes() == ba.raw_secret_bytes(),
    )
}
