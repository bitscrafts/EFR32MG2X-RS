//! The two modes the Bluetooth stack and its security manager need, and nothing else.
//!
//! - **CCM\*** with the parameters Bluetooth actually uses: a 4-byte tag and a 2-byte length field, so a 13-byte
//!   nonce. This is what encrypts link-layer packets (`sli_ccm_encrypt_and_tag_ble` answers it).
//! - **CMAC** (RFC 4493), which is what PSA's `psa_mac_sign_*` computes when the stack imports a key and asks
//!   for a signature.
//!
//! Both sit on `aes_ctr_radio::encrypt_block`, so there is exactly one AES core in this project.
//!
//! # Testing, and the trap it avoids
//!
//! A round-trip test passes for an implementation that is self-consistent and wrong: if both sides mask the tag
//! with the wrong keystream block, or count the counter from the wrong place, the plaintext still comes back.
//! That is exactly how a link-layer crypto bug reaches the air and passes every test in the tree.
//!
//! So the general CCM here is checked against **RFC 3610 packet vector #1** (M = 8, L = 2), comparing
//! ciphertext *and* the full tag, and CMAC against the four **RFC 4493** vectors. The Bluetooth constants are
//! then just a parameter choice on top of the code the vectors validated.
//!
//! This module had that bug before the vectors existed: the tag was masked with the block after the last data
//! block instead of block zero. The round trip could not see it.

#![no_std]
#![forbid(unsafe_code)]

use aes_ctr_radio::encrypt_block;

/// Tag length in bytes, fixed by the Bluetooth core specification.
pub const BLE_TAG_LEN: usize = 4;
/// Length-field size in bytes, fixed by Bluetooth: a 13-byte nonce and a 2-byte counter.
pub const BLE_L: usize = 2;
/// Nonce length implied by `BLE_L`.
pub const BLE_NONCE_LEN: usize = 15 - BLE_L;

/// A packet failed authentication. The caller must drop it; there is no plaintext to salvage.
#[derive(Debug, PartialEq, Eq)]
pub struct TagMismatch;

/// The flags byte of `B0`: Adata, M', L'.
fn b0_flags(tag_len: usize, l: usize, has_aad: bool) -> u8 {
    let adata = if has_aad { 0x40 } else { 0x00 };
    adata | ((((tag_len - 2) / 2) as u8) << 3) | ((l - 1) as u8)
}

/// `B0`: flags, nonce, payload length — the block the CBC-MAC starts from.
fn b0(nonce: &[u8], payload_len: usize, aad_len: usize, tag_len: usize, l: usize) -> [u8; 16] {
    let mut b = [0u8; 16];
    b[0] = b0_flags(tag_len, l, aad_len > 0);
    let nonce_len = 15 - l;
    b[1..1 + nonce_len].copy_from_slice(&nonce[..nonce_len]);
    // The length field is the low `l` bytes, big endian.
    for (i, byte) in b[16 - l..].iter_mut().enumerate() {
        *byte = (payload_len >> (8 * (l - 1 - i))) as u8;
    }
    b
}

/// `A_i`: flags, nonce, counter — the block whose encryption gives keystream block `S_i`.
fn counter_block(nonce: &[u8], i: usize, l: usize) -> [u8; 16] {
    let mut a = [0u8; 16];
    a[0] = (l - 1) as u8;
    let nonce_len = 15 - l;
    a[1..1 + nonce_len].copy_from_slice(&nonce[..nonce_len]);
    for (k, byte) in a[16 - l..].iter_mut().enumerate() {
        *byte = (i >> (8 * (l - 1 - k))) as u8;
    }
    a
}

fn xor_then_encrypt(key: &[u8; 16], state: &[u8; 16], block: &[u8; 16]) -> [u8; 16] {
    let mut mixed = [0u8; 16];
    for i in 0..16 {
        mixed[i] = state[i] ^ block[i];
    }
    encrypt_block(key, &mixed)
}

/// CBC-MAC over the AAD and the plaintext, starting at `B0`.
fn cbc_mac(key: &[u8; 16], first: [u8; 16], data: &[u8], aad: &[u8]) -> [u8; 16] {
    let mut x = encrypt_block(key, &first);

    if !aad.is_empty() {
        // The AAD length is encoded in two bytes here, then the AAD, zero padded. RFC 3610 fixes this to a
        // two-byte length for AAD shorter than 0xFF00 bytes, which is every Bluetooth header.
        let mut block = [0u8; 16];
        block[0] = (aad.len() >> 8) as u8;
        block[1] = aad.len() as u8;
        let mut filled = 2;
        for &byte in aad {
            block[filled] = byte;
            filled += 1;
            if filled == 16 {
                x = xor_then_encrypt(key, &x, &block);
                block = [0u8; 16];
                filled = 0;
            }
        }
        if filled > 0 {
            x = xor_then_encrypt(key, &x, &block);
        }
    }

    let mut offset = 0;
    while offset < data.len() {
        let mut block = [0u8; 16];
        let take = core::cmp::min(16, data.len() - offset);
        block[..take].copy_from_slice(&data[offset..offset + take]);
        x = xor_then_encrypt(key, &x, &block);
        offset += take;
    }
    x
}

/// Apply the CTR keystream to `data`, starting at counter 1.
fn apply_keystream(key: &[u8; 16], nonce: &[u8], data: &mut [u8], l: usize) {
    let mut i = 1usize;
    let mut offset = 0;
    while offset < data.len() {
        let ks = encrypt_block(key, &counter_block(nonce, i, l));
        let take = core::cmp::min(16, data.len() - offset);
        for j in 0..take {
            data[offset + j] ^= ks[j];
        }
        offset += take;
        i += 1;
    }
}

/// General CCM encryption, returning the tag. `tag_len` and `l` are parameters so the RFC 3610 vectors can be
/// reproduced exactly; Bluetooth callers use [`encrypt`], which fixes them.
pub fn encrypt_with(
    key: &[u8; 16],
    nonce: &[u8],
    aad: &[u8],
    data: &mut [u8],
    tag_len: usize,
    l: usize,
) -> [u8; 16] {
    let mac = cbc_mac(key, b0(nonce, data.len(), aad.len(), tag_len, l), data, aad);
    apply_keystream(key, nonce, data, l);

    // **S0, not the block after the data.** The tag mask is the encryption of counter zero. Masking with the
    // next counter is the bug this module shipped before the RFC vectors arrived: round trips pass either way.
    let s0 = encrypt_block(key, &counter_block(nonce, 0, l));
    let mut tag = [0u8; 16];
    for i in 0..tag_len {
        tag[i] = mac[i] ^ s0[i];
    }
    tag
}

/// General CCM decryption. On `Err`, `data` is untouched.
pub fn decrypt_with(
    key: &[u8; 16],
    nonce: &[u8],
    aad: &[u8],
    data: &mut [u8],
    tag: &[u8],
    l: usize,
) -> Result<(), TagMismatch> {
    let tag_len = tag.len();

    // CCM's CBC-MAC covers the **plaintext**, so the data has to be decrypted before the tag can be checked.
    // The contract "on error, the caller's buffer is untouched" is kept by re-applying the keystream, which is
    // its own inverse: the buffer ends byte-identical to how it arrived.
    apply_keystream(key, nonce, data, l);
    let mac = cbc_mac(key, b0(nonce, data.len(), aad.len(), tag_len, l), data, aad);
    let s0 = encrypt_block(key, &counter_block(nonce, 0, l));

    let mut diff = 0u8;
    for i in 0..tag_len {
        diff |= (mac[i] ^ s0[i]) ^ tag[i];
    }
    if diff != 0 {
        apply_keystream(key, nonce, data, l);
        return Err(TagMismatch);
    }
    Ok(())
}

/// Bluetooth's CCM\*: encrypt in place and return the 4-byte tag.
pub fn encrypt(
    key: &[u8; 16],
    nonce: &[u8; BLE_NONCE_LEN],
    aad: &[u8],
    data: &mut [u8],
) -> [u8; BLE_TAG_LEN] {
    let full = encrypt_with(key, nonce, aad, data, BLE_TAG_LEN, BLE_L);
    let mut tag = [0u8; BLE_TAG_LEN];
    tag.copy_from_slice(&full[..BLE_TAG_LEN]);
    tag
}

/// Bluetooth's CCM\*: verify the 4-byte tag, then decrypt in place.
pub fn decrypt(
    key: &[u8; 16],
    nonce: &[u8; BLE_NONCE_LEN],
    aad: &[u8],
    data: &mut [u8],
    tag: &[u8; BLE_TAG_LEN],
) -> Result<(), TagMismatch> {
    decrypt_with(key, nonce, aad, data, tag, BLE_L)
}

/// AES-CMAC (RFC 4493). The tag is 16 bytes; truncate at the caller if less is needed.
pub fn cmac(key: &[u8; 16], msg: &[u8]) -> [u8; 16] {
    let (k1, k2) = subkeys(key);
    let n = msg.len().div_ceil(16);

    let mut x = [0u8; 16];
    if n == 0 {
        let mut last = [0u8; 16];
        last[0] = 0x80;
        xor_into(&mut last, &k2);
        return encrypt_block(key, &xor_arrays(&x, &last));
    }

    for i in 0..n - 1 {
        let mut block = [0u8; 16];
        block.copy_from_slice(&msg[i * 16..i * 16 + 16]);
        x = encrypt_block(key, &xor_arrays(&x, &block));
    }

    let rest = &msg[(n - 1) * 16..];
    let mut last = [0u8; 16];
    if rest.len() == 16 {
        last.copy_from_slice(rest);
        xor_into(&mut last, &k1);
    } else {
        last[..rest.len()].copy_from_slice(rest);
        last[rest.len()] = 0x80;
        xor_into(&mut last, &k2);
    }
    encrypt_block(key, &xor_arrays(&x, &last))
}

fn xor_arrays(a: &[u8; 16], b: &[u8; 16]) -> [u8; 16] {
    let mut out = [0u8; 16];
    for i in 0..16 {
        out[i] = a[i] ^ b[i];
    }
    out
}

fn xor_into(target: &mut [u8; 16], other: &[u8; 16]) {
    for i in 0..16 {
        target[i] ^= other[i];
    }
}

/// RFC 4493 subkeys: K1 = dbl(AES(0)), K2 = dbl(K1).
fn subkeys(key: &[u8; 16]) -> ([u8; 16], [u8; 16]) {
    let l = encrypt_block(key, &[0u8; 16]);
    let k1 = dbl(&l);
    let k2 = dbl(&k1);
    (k1, k2)
}

fn dbl(input: &[u8; 16]) -> [u8; 16] {
    let mut out = [0u8; 16];
    let mut carry = 0u8;
    for i in (0..16).rev() {
        out[i] = (input[i] << 1) | carry;
        carry = input[i] >> 7;
    }
    if carry != 0 {
        out[15] ^= 0x87;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RFC 4493 section 4: the empty message, 16 bytes, 40 bytes and 64 bytes over one key.
    #[test]
    fn rfc4493_cmac_vectors() {
        let key: [u8; 16] = [
            0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf,
            0x4f, 0x3c,
        ];
        let msg = [
            0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93,
            0x17, 0x2a, 0xae, 0x2d, 0x8a, 0x57, 0x1e, 0x03, 0xac, 0x9c, 0x9e, 0xb7, 0x6f, 0xac,
            0x45, 0xaf, 0x8e, 0x51, 0x30, 0xc8, 0x1c, 0x46, 0xa3, 0x5c, 0xe4, 0x11, 0xe5, 0xfb,
            0xc1, 0x19, 0x1a, 0x0a, 0x52, 0xef, 0xf6, 0x9f, 0x24, 0x45, 0xdf, 0x4f, 0x9b, 0x17,
            0xad, 0x2b, 0x41, 0x7b, 0xe6, 0x6c, 0x37, 0x10,
        ];

        let cases: [(usize, [u8; 16]); 4] = [
            (
                0,
                [
                    0xbb, 0x1d, 0x69, 0x29, 0xe9, 0x59, 0x37, 0x28, 0x7f, 0xa3, 0x7d, 0x12, 0x9b,
                    0x75, 0x67, 0x46,
                ],
            ),
            (
                16,
                [
                    0x07, 0x0a, 0x16, 0xb4, 0x6b, 0x4d, 0x41, 0x44, 0xf7, 0x9b, 0xdd, 0x9d, 0xd0,
                    0x4a, 0x28, 0x7c,
                ],
            ),
            (
                40,
                [
                    0xdf, 0xa6, 0x67, 0x47, 0xde, 0x9a, 0xe6, 0x30, 0x30, 0xca, 0x32, 0x61, 0x14,
                    0x97, 0xc8, 0x27,
                ],
            ),
            (
                64,
                [
                    0x51, 0xf0, 0xbe, 0xbf, 0x7e, 0x3b, 0x9d, 0x92, 0xfc, 0x49, 0x74, 0x17, 0x79,
                    0x36, 0x3c, 0xfe,
                ],
            ),
        ];
        for (len, want) in cases {
            assert_eq!(
                cmac(&key, &msg[..len]),
                want,
                "CMAC mismatch at length {len}"
            );
        }
    }

    /// RFC 3610 packet vector #1, in full: M = 8, L = 2, 8 bytes of AAD and 23 bytes of payload. This checks the
    /// ciphertext **and the tag**, which is what catches a wrong tag mask.
    #[test]
    fn rfc3610_packet_vector_one() {
        let key = [
            0xc0, 0xc1, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7, 0xc8, 0xc9, 0xca, 0xcb, 0xcc, 0xcd,
            0xce, 0xcf,
        ];
        let nonce = [
            0x00, 0x00, 0x00, 0x03, 0x02, 0x01, 0x00, 0xa0, 0xa1, 0xa2, 0xa3, 0xa4, 0xa5,
        ];
        let aad = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07];
        let mut data = [
            0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15,
            0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e,
        ];
        let expected_cipher = [
            0x58, 0x8c, 0x97, 0x9a, 0x61, 0xc6, 0x63, 0xd2, 0xf0, 0x66, 0xd0, 0xc2, 0xc0, 0xf9,
            0x89, 0x80, 0x6d, 0x5f, 0x6b, 0x61, 0xda, 0xc3, 0x84,
        ];
        let expected_tag = [0x17, 0xe8, 0xd1, 0x2c, 0xfd, 0xf9, 0x26, 0xe0];

        let tag = encrypt_with(&key, &nonce, &aad, &mut data, 8, 2);
        assert_eq!(data, expected_cipher, "RFC 3610 vector 1 ciphertext");
        assert_eq!(tag[..8], expected_tag[..], "RFC 3610 vector 1 tag");

        // And the same vector decrypts, which proves the tag comparison path, not only the tag computation.
        let mut back = expected_cipher;
        assert_eq!(
            decrypt_with(&key, &nonce, &aad, &mut back, &expected_tag, 2),
            Ok(())
        );
        assert_eq!(
            &back[..],
            &[
                0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15,
                0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e
            ][..]
        );
    }

    /// Bluetooth's parameters: what one side encrypts, the other decrypts, and a single flipped bit anywhere is
    /// rejected without modifying the packet.
    #[test]
    fn ble_round_trip_and_tamper_rejection() {
        let key = [0x11u8; 16];
        let nonce = [0x22u8; BLE_NONCE_LEN];
        let original: [u8; 20] = core::array::from_fn(|i| i as u8);

        let mut data = original;
        let tag = encrypt(&key, &nonce, &[], &mut data);
        assert_ne!(data, original, "encryption must change the packet");

        let mut back = data;
        assert_eq!(decrypt(&key, &nonce, &[], &mut back, &tag), Ok(()));
        assert_eq!(back, original);

        let mut tampered = data;
        tampered[3] ^= 0x01;
        let mut scratch = tampered;
        assert_eq!(
            decrypt(&key, &nonce, &[], &mut scratch, &tag),
            Err(TagMismatch)
        );
        assert_eq!(scratch, tampered, "a rejected packet must not be modified");

        let mut bad_tag = tag;
        bad_tag[0] ^= 0x80;
        let mut scratch2 = data;
        assert_eq!(
            decrypt(&key, &nonce, &[], &mut scratch2, &bad_tag),
            Err(TagMismatch)
        );

        let mut other_nonce = nonce;
        other_nonce[0] ^= 0x01;
        let mut scratch3 = data;
        assert_eq!(
            decrypt(&key, &other_nonce, &[], &mut scratch3, &tag),
            Err(TagMismatch)
        );
    }

    /// Associated data is authenticated and not encrypted: it changes the tag, and changing it invalidates the
    /// packet even though the payload bytes are identical.
    #[test]
    fn associated_data_is_authenticated_not_encrypted() {
        let key = [0x33u8; 16];
        let nonce = [0x44u8; BLE_NONCE_LEN];
        let payload = [0xAAu8; 8];

        let mut with_aad = payload;
        let tag_a = encrypt(&key, &nonce, &[0x01, 0x02], &mut with_aad);
        let mut without = payload;
        let _ = encrypt(&key, &nonce, &[], &mut without);

        // The keystream is the same either way, so identical payloads give identical ciphertext, and only the tag
        // differs. Asserting the *tag* difference is the point: CCM authenticates AAD without encrypting it.
        assert_eq!(with_aad, without, "AAD must not change the ciphertext");
        let mut wrong_aad = with_aad;
        assert_eq!(
            decrypt(&key, &nonce, &[0x01, 0x03], &mut wrong_aad, &tag_a),
            Err(TagMismatch),
            "AAD must be authenticated"
        );
    }

    /// A zero-length payload still authenticates: the tag covers B0.
    #[test]
    fn empty_payload_still_authenticates() {
        let key = [0x55u8; 16];
        let nonce = [0x66u8; BLE_NONCE_LEN];
        let mut nothing: [u8; 0] = [];
        let tag = encrypt(&key, &nonce, &[], &mut nothing);
        assert_eq!(decrypt(&key, &nonce, &[], &mut nothing, &tag), Ok(()));

        let mut bad_tag = tag;
        bad_tag[1] ^= 1;
        let mut bad = [0u8; 0];
        assert_eq!(
            decrypt(&key, &nonce, &[], &mut bad, &bad_tag),
            Err(TagMismatch)
        );
    }

    /// 33 bytes forces three blocks and a partial fourth, so the counter has to carry across the boundary.
    #[test]
    fn multi_block_payload_round_trips() {
        let key = [0x77u8; 16];
        let nonce = [0x88u8; BLE_NONCE_LEN];
        let original: [u8; 33] = core::array::from_fn(|i| (i * 7) as u8);
        let mut data = original;
        let tag = encrypt(&key, &nonce, &[], &mut data);
        let mut back = data;
        assert_eq!(decrypt(&key, &nonce, &[], &mut back, &tag), Ok(()));
        assert_eq!(back, original);
    }

    /// The tag mask is `S0`, so two payloads of different length but the same key and nonce must not share a tag
    /// relationship that a one-block-at-a-time implementation would produce. This is a guard against the exact
    /// bug this module shipped: masking with the block after the data.
    #[test]
    fn tag_uses_counter_zero_not_the_block_after_the_data() {
        let key = [0x99u8; 16];
        let nonce = [0xAAu8; BLE_NONCE_LEN];
        let mut short = [0u8; 16];
        let mut long = [0u8; 32];
        let tag_short = encrypt(&key, &nonce, &[], &mut short);
        let tag_long = encrypt(&key, &nonce, &[], &mut long);
        assert_ne!(tag_short, tag_long);

        // Independent check: the tag must equal CBC-MAC xor E(K, A0), computed here by hand from the primitives.
        let mac = cbc_mac(&key, b0(&nonce, 16, 0, BLE_TAG_LEN, BLE_L), &[0u8; 16], &[]);
        let s0 = encrypt_block(&key, &counter_block(&nonce, 0, BLE_L));
        let expected: [u8; BLE_TAG_LEN] = core::array::from_fn(|i| mac[i] ^ s0[i]);
        assert_eq!(tag_short, expected);
    }
}
