//! The two Silicon Labs support symbols the Bluetooth stack expects.
//!
//! The implementations are Rust; the `extern "C"` on them is only the ABI the vendor's blob calls through,
//! not a dependency on vendor C. Both were vendor sources in the SDK (`sli_psa_crypto.c`,
//! `sli_protocol_crypto_radioaes.c`, Gecko SDK v4.4.5, Zlib) and the logic now lives in this repository:
//! the first is a table of measured sizes, the second is `libraries/aes-ctr-radio`.

/// Size of a PSA context, by context type.
///
/// The vendor implementation is a `switch` over `sizeof(psa_*_operation_t)`. Those sizes depend on the
/// mbedtls configuration the stack was built with, and the SDK ships that configuration only as a
/// per-project template, so the exact numbers the blob expects are not published.
///
/// Measured with the vendor's own mbedtls headers (`psa/crypto.h` from the Gecko SDK v4.4.5 package):
///
/// ```text
/// psa_hash_operation_t            232
/// psa_cipher_operation_t          120
/// psa_pake_operation_t            912
/// psa_mac_operation_t             400
/// psa_aead_operation_t            480
/// psa_key_derivation_operation_t  568
/// psa_key_attributes_t             48
/// ```
///
/// The values returned are those measurements **rounded up**, and the direction matters: the result is
/// passed straight to an allocator, so over-allocating wastes a few bytes while under-allocating corrupts
/// the heap in code nobody is looking at. Rounding up is safe by construction; a guessed exact number
/// would not be.
///
/// The enum order is the vendor's (`sli_psa_crypto.h`), and it agrees with the call sites in the blobs:
/// the stack asks for type 6 (`psa_key_attributes_t`), 3 (`psa_mac_operation_t`) and 0
/// (`psa_hash_operation_t`), which is what the disassembly shows in `r0`.
#[no_mangle]
pub extern "C" fn sli_psa_context_get_size(ctx_type: u32) -> usize {
    match ctx_type {
        0 => 256,  // psa_hash_operation_t            232 measured
        1 => 160,  // psa_cipher_operation_t          120
        2 => 1024, // psa_pake_operation_t            912
        3 => 448,  // psa_mac_operation_t             400
        4 => 512,  // psa_aead_operation_t            480
        5 => 640,  // psa_key_derivation_operation_t  568
        6 => 64,   // psa_key_attributes_t             48
        _ => 0,    // the vendor returns 0 for anything it does not know
    }
}

/// AES-CTR in software, 16 bytes at a time, for the controller's access addresses.
///
/// The signature is the vendor's, from `sli_protocol_crypto.h`, and it matches the call site in
/// `libbluetooth_controller...a` (six arguments, confirmed by disassembly):
///
/// ```c
/// sl_status_t sli_aes_crypt_ctr_radio(const unsigned char *key, unsigned int keybits,
///                                     const unsigned char input[16], const unsigned char iv_in[16],
///                                     volatile unsigned char iv_out[16],
///                                     volatile unsigned char output[16]);
/// ```
///
/// The vendor drives the RADIOAES peripheral here. Software was chosen instead because that block is
/// shared with the link layer: a software stream cannot contend with the radio for it, and a handful of
/// blocks per connection is not a performance question.
///
/// Only 128-bit keys are accepted; anything else returns `SL_STATUS_NOT_SUPPORTED` rather than
/// truncating a key silently.
///
/// # Safety
///
/// All six pointers must be valid. `output` and `iv_out` are written; the rest are read.
#[no_mangle]
pub unsafe extern "C" fn sli_aes_crypt_ctr_radio(
    key: *const u8,
    keybits: u32,
    input: *const u8,
    iv_in: *const u8,
    iv_out: *mut u8,
    output: *mut u8,
) -> u32 {
    const SL_STATUS_OK: u32 = 0;
    const SL_STATUS_INVALID_PARAMETER: u32 = 0x21;
    const SL_STATUS_NOT_SUPPORTED: u32 = 0x0E;

    if key.is_null() || input.is_null() || iv_in.is_null() || iv_out.is_null() || output.is_null() {
        return SL_STATUS_INVALID_PARAMETER;
    }
    if keybits != 128 {
        return SL_STATUS_NOT_SUPPORTED;
    }

    let mut k = [0u8; 16];
    let mut counter = [0u8; 16];
    let mut block = [0u8; 16];
    for i in 0..16 {
        k[i] = core::ptr::read_volatile(key.add(i));
        counter[i] = core::ptr::read_volatile(iv_in.add(i));
        block[i] = core::ptr::read_volatile(input.add(i));
    }

    let (cipher, next) = aes_ctr_radio::ctr_step(&k, &counter, &block);

    for i in 0..16 {
        core::ptr::write_volatile(output.add(i), cipher[i]);
        core::ptr::write_volatile(iv_out.add(i), next[i]);
    }

    SL_STATUS_OK
}
