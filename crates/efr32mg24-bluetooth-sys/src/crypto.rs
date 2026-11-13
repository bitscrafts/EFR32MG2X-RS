//! The two link-layer crypto entry points, on top of `libraries/aes-modes`.
//!
//! Both signatures come from `sli_protocol_crypto.h` in Gecko SDK v4.4.5 — not inferred from a disassembly:
//!
//! ```c
//! sl_status_t sli_aes_crypt_ecb_radio(bool encrypt, const unsigned char *key, unsigned int keybits,
//!                                     const unsigned char input[16], volatile unsigned char output[16]);
//! sl_status_t sli_ccm_encrypt_and_tag_ble(unsigned char *data, size_t length, const unsigned char *key,
//!                                         const unsigned char *iv, unsigned char header, unsigned char *tag);
//! ```
//!
//! The AES core and CCM\* are in `aes-modes`, validated against RFC 4493, RFC 3610 (ciphertext *and* tag) and
//! the tamper cases. This file is the ABI seam and nothing more: pointer checks, key-size check, status codes.
//!
//! # Why the key size is checked rather than assumed
//!
//! `keybits` is a parameter, so a caller could pass 192 or 256. This AES is 128-bit only; returning
//! `SL_STATUS_NOT_SUPPORTED` for anything else makes that a visible refusal instead of 24 silently ignored bytes
//! and a packet the peer cannot decrypt.

use core::ffi::c_void;
use core::ptr;

use aes_ctr_radio::encrypt_block;
use aes_modes::{cmac, decrypt as ccm_decrypt, encrypt as ccm_encrypt, BLE_NONCE_LEN, BLE_TAG_LEN};

/// `SL_STATUS_OK`
const SL_STATUS_OK: u32 = 0;
/// `SL_STATUS_NOT_SUPPORTED`, which is `0x000F` -- checked against `sl_status.h`, not guessed. It was `0x0004`
/// here for a while, and `0x0004` is no status at all.
const SL_STATUS_NOT_SUPPORTED: u32 = 0x000F;
/// `SL_STATUS_INVALID_PARAMETER`
const SL_STATUS_INVALID_PARAMETER: u32 = 0x0000_0021;
/// `SL_STATUS_INVALID_SIGNATURE`
const SL_STATUS_INVALID_SIGNATURE: u32 = 0x002C;

/// AES-ECB, one block. The stack uses it for single-block operations; CCM\* below does its own blocks.
///
/// # Safety
///
/// `key` must point to at least `keybits / 8` readable bytes, `input` to 16 readable bytes, and `output` to 16
/// writable bytes. This is the C ABI the stack calls with those guarantees.
#[no_mangle]
pub unsafe extern "C" fn sli_aes_crypt_ecb_radio(
    encrypt: bool,
    key: *const u8,
    keybits: u32,
    input: *const u8,
    output: *mut u8,
) -> u32 {
    if key.is_null() || input.is_null() || output.is_null() {
        return SL_STATUS_INVALID_PARAMETER;
    }
    if keybits != 128 {
        return SL_STATUS_NOT_SUPPORTED;
    }

    let mut k = [0u8; 16];
    let mut block = [0u8; 16];
    ptr::copy_nonoverlapping(key, k.as_mut_ptr(), 16);
    ptr::copy_nonoverlapping(input, block.as_mut_ptr(), 16);

    // AES decryption of a single block is not the same as encryption with a reversed round; `aes-modes` does not
    // expose a decrypt core because nothing in this firmware uses it. Refusing is honest and loud.
    if !encrypt {
        return SL_STATUS_NOT_SUPPORTED;
    }

    let out = encrypt_block(&k, &block);
    ptr::copy_nonoverlapping(out.as_ptr(), output, 16);
    SL_STATUS_OK
}

/// AES-CMAC over one block of data, exposed because PSA's MAC path needs it and the stack asks through PSA.
///
/// # Safety
///
/// `key` must point to 16 readable bytes and `msg` to `len` readable bytes.
pub unsafe fn cmac_of(key: *const u8, msg: *const u8, len: usize) -> [u8; 16] {
    let mut k = [0u8; 16];
    ptr::copy_nonoverlapping(key, k.as_mut_ptr(), 16);
    let slice = core::slice::from_raw_parts(msg, len);
    cmac(&k, slice)
}

/// BLE CCM\*: encrypt `data` in place and write the 4-byte tag. `header` is the one-byte PDU header that
/// Bluetooth authenticates as associated data.
///
/// # Safety
///
/// `data` must point to `length` writable bytes, `key` to 16 readable bytes, `iv` to 13 readable bytes, and
/// `tag` to 4 writable bytes.
#[no_mangle]
pub unsafe extern "C" fn sli_ccm_encrypt_and_tag_ble(
    data: *mut u8,
    length: usize,
    key: *const u8,
    iv: *const u8,
    header: u8,
    tag: *mut u8,
) -> u32 {
    if data.is_null() || key.is_null() || iv.is_null() || tag.is_null() {
        return SL_STATUS_INVALID_PARAMETER;
    }

    let mut k = [0u8; 16];
    let mut nonce = [0u8; BLE_NONCE_LEN];
    ptr::copy_nonoverlapping(key, k.as_mut_ptr(), 16);
    ptr::copy_nonoverlapping(iv, nonce.as_mut_ptr(), BLE_NONCE_LEN);

    let payload = core::slice::from_raw_parts_mut(data, length);
    let aad = [header];
    let out = ccm_encrypt(&k, &nonce, &aad, payload);
    ptr::copy_nonoverlapping(out.as_ptr(), tag, BLE_TAG_LEN);
    SL_STATUS_OK
}

/// The decrypting direction, under the name `sli_protocol_crypto.h` actually declares.
///
/// The link failed on this symbol the moment the stack was really started (`sli_bt_system_start_bluetooth`), which
/// is how the name was found: the header had it, and this file only had the invented one below.
///
/// # Safety
///
/// As `sli_ccm_encrypt_and_tag_ble`: `data` must point to `length` writable bytes, `key` to 16 readable bytes, `iv`
/// to 13, and `tag` to 4.
#[no_mangle]
pub unsafe extern "C" fn sli_ccm_auth_decrypt_ble(
    data: *mut u8,
    length: usize,
    key: *const u8,
    iv: *const u8,
    header: u8,
    tag: *const u8,
) -> u32 {
    sli_ccm_decrypt_and_verify_ble(data, length, key, iv, header, tag)
}

/// The decrypting direction, kept under the name this file invented before the header was read.
///
/// The vendor's name is `sli_ccm_auth_decrypt_ble`, which is now the entry point above and delegates here.
///
/// # Safety
///
/// As `sli_ccm_encrypt_and_tag_ble`; the returned status distinguishes a forged packet from a bad argument.
#[no_mangle]
pub unsafe extern "C" fn sli_ccm_decrypt_and_verify_ble(
    data: *mut u8,
    length: usize,
    key: *const u8,
    iv: *const u8,
    header: u8,
    tag: *const u8,
) -> u32 {
    if data.is_null() || key.is_null() || iv.is_null() || tag.is_null() {
        return SL_STATUS_INVALID_PARAMETER;
    }

    let mut k = [0u8; 16];
    let mut nonce = [0u8; BLE_NONCE_LEN];
    let mut expected = [0u8; BLE_TAG_LEN];
    ptr::copy_nonoverlapping(key, k.as_mut_ptr(), 16);
    ptr::copy_nonoverlapping(iv, nonce.as_mut_ptr(), BLE_NONCE_LEN);
    ptr::copy_nonoverlapping(tag, expected.as_mut_ptr(), BLE_TAG_LEN);

    let payload = core::slice::from_raw_parts_mut(data, length);
    let aad = [header];
    match ccm_decrypt(&k, &nonce, &aad, payload, &expected) {
        Ok(()) => SL_STATUS_OK,
        // Authentication failure is its own status, and it is `SL_STATUS_INVALID_SIGNATURE` = `0x002C`: the caller
        // must drop the packet, not retry. `0x0F01` was invented here and means nothing to the stack.
        Err(_) => SL_STATUS_INVALID_SIGNATURE,
    }
}

/// Kept for the ABI's sake: the stack may compare against a `void *` handle.
pub type Opaque = c_void;
